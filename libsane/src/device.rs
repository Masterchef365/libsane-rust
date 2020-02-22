use crate::error::{Result, SaneError};
use crate::option_descriptor::{OptionDescriptor, OptionDescriptorIterator, Settable, ValueType};
use libsane_sys::*;
use std::{
    ffi::{c_void, CStr},
    marker::PhantomData,
};

pub struct Device<'sane> {
    handle: SANE_Handle,
    _phantomdata: PhantomData<&'sane ()>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(Box<[bool]>),
    Int(Box<[i32]>),
    Fixed(Box<[i32]>),
    String(Box<CStr>),
}

impl Value {
    pub(crate) unsafe fn as_ptr(&self) -> *mut c_void {
        match self {
            Value::Bool(b) => b.as_ptr() as *mut c_void,
            Value::Int(i) => i.as_ptr() as *mut c_void,
            Value::Fixed(i) => i.as_ptr() as *mut c_void,
            Value::String(s) => s.as_ptr() as *mut c_void,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrameType {
    ///Band covering human visual range.
    Gray,
    /// Pixel-interleaved red/green/blue bands.
    RGB,
    /// Red band of a red/green/blue image.
    Red,
    /// Green band of a red/green/blue image.
    Green,
    /// Blue band of a red/green/blue image.
    Blue,
}

#[derive(Debug, Clone, Copy)]
pub struct ScanParameters {
    /// Specifies the format of the next frame to be returned.
    format: FrameType,
    /// Set to `true` if and only if the frame that is currently being acquired is the last frame of a multi frame image.
    last_frame: bool,
    /// How many scan lines the frame is comprised of. None if the number of lines is not known a priori.
    lines: Option<SANE_Int>,
    /// Number of bytes per scan line.
    bytes_per_line: SANE_Int,
    /// Number of pixels per scan line.
    pixels_per_line: SANE_Int,
    /// Number of bits per sample.
    depth: SANE_Int,
}

impl From<SANE_Parameters> for ScanParameters {
    fn from(params: SANE_Parameters) -> ScanParameters {
        let format = match params.format {
            SANE_Frame_SANE_FRAME_GRAY => FrameType::Gray,
            SANE_Frame_SANE_FRAME_RGB => FrameType::RGB,
            SANE_Frame_SANE_FRAME_RED => FrameType::Red,
            SANE_Frame_SANE_FRAME_GREEN => FrameType::Green,
            SANE_Frame_SANE_FRAME_BLUE => FrameType::Blue,
            _ => panic!("Unrecognized frame type"),
        };

        let last_frame = match params.last_frame {
            0 => false,
            1 => true,
            _ => panic!("Invalid boolean"),
        };

        ScanParameters {
            format,
            last_frame,
            lines: if params.lines != -1 {
                Some(params.lines)
            } else {
                None
            },
            bytes_per_line: params.bytes_per_line,
            pixels_per_line: params.pixels_per_line,
            depth: params.depth,
        }
    }
}

impl<'sane> Device<'sane> {
    pub(crate) fn open_device(name: &CStr) -> Result<Self> {
        let mut handle: SANE_Handle = std::ptr::null_mut();

        unsafe {
            SaneError::from_retcode(sane_open(name.as_ptr(), &mut handle as *mut SANE_Handle))?
        };

        Ok(Self {
            handle,
            _phantomdata: PhantomData,
        })
    }

    pub(crate) fn get_handle(&self) -> SANE_Handle {
        self.handle
    }

    pub fn options<'device>(&'device self) -> OptionDescriptorIterator<'device, 'sane> {
        OptionDescriptorIterator::new(self)
    }

    pub fn set_option(&self, descriptor: &OptionDescriptor, value: &Value) -> Result<()> {
        match (&value, descriptor.value_type) {
            (Value::Bool(_), ValueType::Bool) => (),
            (Value::Int(_), ValueType::Int) => (),
            (Value::Fixed(_), ValueType::Fixed) => (),
            (Value::String(_), ValueType::String) => (),
            _ => panic!("Invalid value type"),
        }

        // TODO: Emit warning?
        if let Settable::Hardware { .. } = descriptor.capabilities.settable {
            return Ok(());
        }

        unsafe {
            SaneError::from_retcode(sane_control_option(
                self.handle,
                descriptor.number,
                SANE_Action_SANE_ACTION_SET_VALUE,
                value.as_ptr() as *mut c_void,
                std::ptr::null_mut(),
            ))?
        }
        Ok(())
    }

    pub fn get_option(&self, descriptor: &OptionDescriptor) -> Result<Option<Value>> {
        let mut buffer = vec![0u8; descriptor.size as usize];

        if let Settable::Hardware {
            software_visible: false,
        } = descriptor.capabilities.settable
        {
            return Ok(None);
        }

        match descriptor.value_type {
            ValueType::Group | ValueType::Button => return Ok(None),
            _ => (),
        }

        // TODO: This sucks fuck
        unsafe {
            SaneError::from_retcode(sane_control_option(
                self.handle,
                descriptor.number,
                SANE_Action_SANE_ACTION_GET_VALUE,
                buffer.as_mut_ptr() as *mut c_void,
                std::ptr::null_mut(),
            ))?;

            let buffer = buffer.into_boxed_slice();
            let len = buffer.len();
            let ptr = buffer.as_ptr();
            std::mem::forget(buffer);

            // TODO: Check size before passing the pointer?
            return Ok(Some(match descriptor.value_type {
                ValueType::Bool => {
                    Value::Bool({ Box::from(std::slice::from_raw_parts(ptr as *const bool, len)) })
                }
                ValueType::Int => Value::Int({
                    Box::from(std::slice::from_raw_parts(ptr as *const i32, len / 4))
                }),
                ValueType::Fixed => Value::Fixed({
                    Box::from(std::slice::from_raw_parts(ptr as *const i32, len / 4))
                }),
                ValueType::String => Value::String(Box::from(CStr::from_ptr(ptr as *const i8))),
                _ => unreachable!(),
            }));
        }
    }

    pub fn get_params(&self) -> Result<ScanParameters> {
        let mut parameters = SANE_Parameters {
            format: Default::default(),
            last_frame: Default::default(),
            bytes_per_line: Default::default(),
            pixels_per_line: Default::default(),
            lines: Default::default(),
            depth: Default::default(),
        };
        unsafe {
            SaneError::from_retcode(sane_get_parameters(
                self.handle,
                &mut parameters as *mut SANE_Parameters,
            ))?;
        }
        Ok(parameters.into())
    }
}

impl Drop for Device<'_> {
    fn drop(&mut self) {
        unsafe {
            sane_close(self.handle);
        }
    }
}
