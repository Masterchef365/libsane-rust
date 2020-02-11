#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libsane_sys::*;
use std::{ffi::CStr, fmt, marker::PhantomData};
pub struct LibSane;

impl LibSane {
    pub fn init() -> Result<Self, SaneError> {
        let mut version: i32 = 0;
        unsafe {
            //TODO: Implement callback
            SaneError::from_retcode(sane_init(&mut version as *mut i32, None)).map(|_| LibSane)
        }
    }

    pub fn get_devices<'a>(&'a self, local_only: bool) -> Result<DeviceList<'a>, SaneError> {
        DeviceList::get_devices(self, local_only)
    }

    pub fn open<'a>(&self, name: &str) -> Result<Device<'a>, SaneError> {
        let name = std::ffi::CString::new(name).expect("Invalid C String");
        Device::open_device(&name)
    }
}

impl Drop for LibSane {
    fn drop(&mut self) {
        unsafe { sane_exit() }
    }
}

#[derive(Debug)]
pub struct DeviceDescription<'devlist> {
    pub name: &'devlist CStr,
    pub vendor: &'devlist CStr,
    pub model: &'devlist CStr,
    pub type_: &'devlist CStr,
}

impl DeviceDescription<'_> {
    fn from_ptr(ptr: *const SANE_Device) -> Self {
        unsafe {
            Self {
                name: CStr::from_ptr((*ptr).name),
                vendor: CStr::from_ptr((*ptr).vendor),
                model: CStr::from_ptr((*ptr).model),
                type_: CStr::from_ptr((*ptr).type_),
            }
        }
    }

    pub fn open(&self) -> Result<Device, SaneError> {
        Device::open_device(self.name)
    }
}

pub struct DeviceList<'sane> {
    devices: *mut *const SANE_Device,
    _phantomdata: PhantomData<&'sane ()>,
}

impl<'sane> DeviceList<'sane> {
    pub fn get_devices(_libsane: &'sane LibSane, local_only: bool) -> Result<Self, SaneError> {
        let local_only = if local_only { 1 } else { 0 };
        let mut devices: *mut *const SANE_Device = std::ptr::null_mut();
        unsafe {
            SaneError::from_retcode(sane_get_devices(
                &mut devices as *mut *mut *const SANE_Device,
                local_only,
            ))?;
        }

        Ok(Self {
            devices,
            _phantomdata: PhantomData,
        })
    }

    pub fn iter<'devlist>(&'devlist self) -> SaneDeviceListIter<'devlist, 'sane> {
        SaneDeviceListIter {
            device_list: self,
            position: 0,
        }
    }
}

pub struct SaneDeviceListIter<'devlist, 'sane> {
    device_list: &'devlist DeviceList<'sane>,
    position: isize,
}

impl<'devlist, 'sane> Iterator for SaneDeviceListIter<'devlist, 'sane> {
    type Item = DeviceDescription<'devlist>;
    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.device_list.devices.wrapping_offset(self.position);

        unsafe {
            if (*ptr).is_null() {
                None
            } else {
                self.position += 1;
                Some(DeviceDescription::from_ptr(*ptr))
            }
        }
    }
}

pub struct Device<'sane> {
    handle: SANE_Handle,
    _phantomdata: PhantomData<&'sane ()>,
}

impl Device<'_> {
    pub(crate) fn open_device(name: &CStr) -> Result<Self, SaneError> {
        let mut handle: SANE_Handle = std::ptr::null_mut();

        unsafe {
            SaneError::from_retcode(sane_open(name.as_ptr(), &mut handle as *mut SANE_Handle))?
        };

        Ok(Self {
            handle,
            _phantomdata: PhantomData,
        })
    }
}

impl Drop for Device<'_> {
    fn drop(&mut self) {
        unsafe {
            sane_close(self.handle);
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum SaneError {
    Unsupported,
    Cancelled,
    DeviceBusy,
    Invalid,
    EOF,
    Jammed,
    NoDocs,
    CoverOpen,
    Io,
    Memory,
    AccessDenied,
}

impl fmt::Display for SaneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SaneError::Unsupported => "Operation is not supported.",
                SaneError::Cancelled => "Operation was cancelled.",
                SaneError::DeviceBusy => "Device is busy, retry later.",
                SaneError::Invalid => "Data or argument is invalid.",
                SaneError::EOF => "No more data available (end-of-file).",
                SaneError::Jammed => "Document feeder jammed.",
                SaneError::NoDocs => "Document feeder out of documents.",
                SaneError::CoverOpen => "Scanner cover is open.",
                SaneError::Io => "Error during device I/O.",
                SaneError::Memory => "Out of memory.",
                SaneError::AccessDenied => "Access to resource has been denied.  ,",
            }
        )
    }
}

impl SaneError {
    pub fn from_retcode(code: SANE_Status) -> Result<(), Self> {
        match code {
            SANE_Status_SANE_STATUS_GOOD => Ok(()),
            SANE_Status_SANE_STATUS_UNSUPPORTED => Err(SaneError::Unsupported),
            SANE_Status_SANE_STATUS_CANCELLED => Err(SaneError::Cancelled),
            SANE_Status_SANE_STATUS_DEVICE_BUSY => Err(SaneError::DeviceBusy),
            SANE_Status_SANE_STATUS_INVAL => Err(SaneError::Invalid),
            SANE_Status_SANE_STATUS_EOF => Err(SaneError::EOF),
            SANE_Status_SANE_STATUS_JAMMED => Err(SaneError::Jammed),
            SANE_Status_SANE_STATUS_NO_DOCS => Err(SaneError::NoDocs),
            SANE_Status_SANE_STATUS_COVER_OPEN => Err(SaneError::CoverOpen),
            SANE_Status_SANE_STATUS_IO_ERROR => Err(SaneError::Io),
            SANE_Status_SANE_STATUS_NO_MEM => Err(SaneError::Memory),
            SANE_Status_SANE_STATUS_ACCESS_DENIED => Err(SaneError::AccessDenied),
            _ => panic!("Unrecognized or unsupported SANE return code."),
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Settable {
    /// The option value can only be set in software
    Software,
    /// The option value can only be by physical hardware (e.g. a switch)
    Hardware {
        /// The option's value is visible to software
        software_visible: bool,
    },
}

#[derive(Debug)]
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

// TODO: TryFrom?
impl From<SANE_Int> for Capabilities {
    fn from(cap: SANE_Int) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum ValueType {
    Bool,
    Int,
    Fixed,
    String,
    Button,
    Group,
}

impl From<SANE_Value_Type> for ValueType {
    fn from(vt: SANE_Value_Type) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum Constraint<'a> {
    None,
    Range {
        min: u32,
        max: u32,
        quant: Option<std::num::NonZeroU32>,
    },
    List(&'a [SANE_Word]),
    StringList(Vec<&'a CStr>),
}

impl<'a> Constraint<'a> {
    pub(crate) fn new(constraint_type: SANE_Constraint_Type, value_ptr: &SANE_Option_Descriptor__bindgen_ty_1) -> Self {
        match constraint_type {
            SANE_Constraint_Type_SANE_CONSTRAINT_NONE => Constraint::None,
            SANE_Constraint_Type_SANE_CONSTRAINT_RANGE => todo!(),
            SANE_Constraint_Type_SANE_CONSTRAINT_STRING_LIST => todo!(),
            SANE_Constraint_Type_SANE_CONSTRAINT_WORD_LIST => todo!(),
            _ => panic!("Unrecognized constraint type"),
        }
    }
}

#[derive(Debug)]
pub struct OptionDescriptor<'a> {
    pub name: &'a CStr,
    pub title: &'a CStr,
    pub description: &'a CStr,
    pub value_type: ValueType,
    pub unit: Unit,
    pub size: SANE_Int,
    pub constraint: Constraint<'a>,
}

impl<'a> OptionDescriptor<'a> {
    pub(crate) fn from_descriptor(descriptor: &'a SANE_Option_Descriptor) -> Self {
        unsafe {
            Self {
                name: CStr::from_ptr(descriptor.name),
                title: CStr::from_ptr(descriptor.title),
                description: CStr::from_ptr(descriptor.desc),
                constraint: Constraint::new(descriptor.constraint_type, &descriptor.constraint),
                unit: descriptor.unit.into(),
                size: descriptor.size,
                value_type: descriptor.type_.into(),
            }
        }
    }
}
