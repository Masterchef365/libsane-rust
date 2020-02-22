use crate::{device::Device, error::SaneError};
use libsane_sys::*;
use std::ffi::CStr;
use std::num::NonZeroI32;

//TODO: Add descriptions for all items here
#[derive(Debug)]
pub struct OptionDescriptor<'a> {
    /// This option's position in a description list
    pub number: SANE_Int,
    pub name: Option<&'a CStr>,
    pub title: Option<&'a CStr>,
    pub description: Option<&'a CStr>,
    pub value_type: ValueType,
    pub capabilities: Capabilities,
    pub unit: Unit,
    pub size: SANE_Int,
    pub constraint: Constraint<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Bool,
    Int,
    Fixed,
    String,
    Button,
    Group,
}

#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    pub settable: Settable,
    /// If set, this capability is not directly supported by the device and is instead emulated in the backend
    pub emulated: bool,
    /// If set, this capability indicates that the backend (or the device) is capable to picking a reasonable option value automatically.
    pub automatic: bool,
    /// If set, this capability indicates that the option is not currently active (e.g., because it's meaningful only if another option is set to some other value).
    pub inactive: bool,
    ///  If set, this capability indicates that the option should be considered an "advanced user option".
    pub advanced: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Settable {
    /// The option value can only be set in software
    Software,
    /// The option value can only be by physical hardware (e.g. a switch)
    Hardware {
        /// The option's value is visible to software
        software_visible: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Unit {
    /// Value is unit-less (e.g., page count).
    None,
    /// Value is in number of pixels.
    Pixel,
    /// Value is in number of bits.
    Bit,
    /// Value is in millimeters.
    MM,
    /// Value is a resolution in dots/inch.
    DPI,
    /// Value is a percentage.
    Percent,
    /// Value is time in µ-seconds.
    Microsecond,
}
#[derive(Debug)]
pub enum Constraint<'a> {
    None,
    Range {
        min: i32,
        max: i32,
        quant: Option<NonZeroI32>,
    },
    List(&'a [SANE_Word]),
    StringList(Vec<&'a CStr>),
}

/// Traverses a null terminated list of C strings
fn null_term_cstring_list<'a>(list: *const SANE_String_Const) -> Vec<&'a CStr> {
    let mut i = 0;
    let mut strings = Vec::new();
    unsafe {
        loop {
            let ptr = list.wrapping_offset(i);
            if (*ptr).is_null() {
                break strings;
            } else {
                strings.push(CStr::from_ptr(*ptr));
            }
            i += 1;
        }
    }
}

impl<'a> Constraint<'a> {
    pub(crate) fn new(
        constraint_type: SANE_Constraint_Type,
        value_ptr: &SANE_Option_Descriptor__bindgen_ty_1,
    ) -> Self {
        unsafe {
            match constraint_type {
                SANE_Constraint_Type_SANE_CONSTRAINT_NONE => Constraint::None,
                SANE_Constraint_Type_SANE_CONSTRAINT_RANGE => {
                    let range = *value_ptr.range;
                    Constraint::Range {
                        min: range.min,
                        max: range.max,
                        quant: NonZeroI32::new(range.max),
                    }
                }
                SANE_Constraint_Type_SANE_CONSTRAINT_STRING_LIST => {
                    Constraint::StringList(null_term_cstring_list(value_ptr.string_list))
                }
                SANE_Constraint_Type_SANE_CONSTRAINT_WORD_LIST => {
                    let length = *value_ptr.word_list.wrapping_offset(0);
                    let contents = value_ptr.word_list.wrapping_offset(1);
                    let list = std::slice::from_raw_parts(contents, length as usize);
                    Constraint::List(list)
                }
                _ => panic!("Unrecognized constraint type"),
            }
        }
    }
}

unsafe fn optional_cstr<'a>(ptr: *const i8) -> Option<&'a CStr> {
    if ptr.is_null() {
        None
    } else {
        Some(CStr::from_ptr(ptr))
    }
}

// TODO: TryFrom?
impl From<SANE_Unit> for Unit {
    fn from(unit: SANE_Unit) -> Self {
        match unit {
            SANE_Unit_SANE_UNIT_NONE => Unit::None,
            SANE_Unit_SANE_UNIT_PIXEL => Unit::Pixel,
            SANE_Unit_SANE_UNIT_BIT => Unit::Bit,
            SANE_Unit_SANE_UNIT_MM => Unit::MM,
            SANE_Unit_SANE_UNIT_DPI => Unit::DPI,
            SANE_Unit_SANE_UNIT_PERCENT => Unit::Percent,
            SANE_Unit_SANE_UNIT_MICROSECOND => Unit::Microsecond,
            _ => panic!("Unrecognized or unsupported unit specifier"),
        }
    }
}

impl Into<SANE_Unit> for Unit {
    fn into(self) -> SANE_Unit {
        match self {
            Unit::None => SANE_Unit_SANE_UNIT_NONE,
            Unit::Pixel => SANE_Unit_SANE_UNIT_PIXEL,
            Unit::Bit => SANE_Unit_SANE_UNIT_BIT,
            Unit::MM => SANE_Unit_SANE_UNIT_MM,
            Unit::DPI => SANE_Unit_SANE_UNIT_DPI,
            Unit::Percent => SANE_Unit_SANE_UNIT_PERCENT,
            Unit::Microsecond => SANE_Unit_SANE_UNIT_MICROSECOND,
        }
    }
}

impl From<SANE_Int> for Capabilities {
    fn from(cap: SANE_Int) -> Self {
        // TODO: Use a bitfield crate to do this?
        let software_settable = cap >> 0 & 1 == 1;
        let hardware_settable = cap >> 1 & 1 == 1;
        let software_visible = cap >> 2 & 1 == 1;
        let emulated = cap >> 3 & 1 == 1;
        let automatic = cap >> 4 & 1 == 1;
        let inactive = cap >> 5 & 1 == 1;
        let advanced = cap >> 6 & 1 == 1;

        let settable = match (software_settable, hardware_settable) {
            // TODO: These should be mutually exclusive??
            (true, _) => Settable::Software,
            (false, _) => Settable::Hardware { software_visible },
            //_ => panic!("Invalid capability bitfield"),
        };

        Self {
            settable,
            emulated,
            automatic,
            inactive,
            advanced,
        }
    }
}

impl From<SANE_Value_Type> for ValueType {
    fn from(vt: SANE_Value_Type) -> Self {
        match vt {
            SANE_Value_Type_SANE_TYPE_BOOL => ValueType::Bool,
            SANE_Value_Type_SANE_TYPE_BUTTON => ValueType::Button,
            SANE_Value_Type_SANE_TYPE_FIXED => ValueType::Fixed,
            SANE_Value_Type_SANE_TYPE_GROUP => ValueType::Group,
            SANE_Value_Type_SANE_TYPE_INT => ValueType::Int,
            SANE_Value_Type_SANE_TYPE_STRING => ValueType::String,
            _ => panic!("Invalid value type"),
        }
    }
}

impl<'a> OptionDescriptor<'a> {
    pub(crate) fn from_descriptor(descriptor: &'a SANE_Option_Descriptor, number: SANE_Int) -> Self {
        unsafe {
            Self {
                name: optional_cstr(descriptor.name),
                title: optional_cstr(descriptor.title),
                description: optional_cstr(descriptor.desc),
                capabilities: Capabilities::from(descriptor.cap),
                constraint: Constraint::new(descriptor.constraint_type, &descriptor.constraint),
                unit: descriptor.unit.into(),
                size: descriptor.size,
                value_type: descriptor.type_.into(),
                number,
            }
        }
    }
}

pub struct OptionDescriptorIterator<'device, 'sane> {
    device: &'device Device<'sane>,
    length: SANE_Int,
    position: SANE_Int,
}

impl<'device, 'sane> OptionDescriptorIterator<'device, 'sane> {
    pub(crate) fn new(device: &'device Device<'sane>) -> Self {
        let mut length: SANE_Int = 0;
        // TODO: Clean up this getter (Abstract elsewhere eventually)
        unsafe {
            let mut info: SANE_Int = 0;
            SaneError::from_retcode(sane_control_option(
                device.get_handle(),
                0,
                SANE_Action_SANE_ACTION_GET_VALUE,
                &mut length as *mut i32 as *mut std::ffi::c_void,
                &mut info as *mut i32,
            ))
            .unwrap()
        }

        Self {
            device,
            length,
            position: 0,
        }
    }
}

impl<'device, 'sane> Iterator for OptionDescriptorIterator<'device, 'sane> {
    type Item = OptionDescriptor<'device>;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let ptr = sane_get_option_descriptor(self.device.get_handle(), self.position);
            if self.position >= self.length || ptr.is_null() {
                None
            } else {
                let desc = OptionDescriptor::from_descriptor(&*ptr, self.position);
                self.position += 1;
                Some(desc)
            }
        }
    }
}
