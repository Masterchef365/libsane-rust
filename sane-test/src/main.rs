#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libsane_sys::*;
use std::{
    ffi::CStr,
    fmt::{self, Display},
    marker::PhantomData,
};

unsafe extern "C" fn auth_callback(
    resource: SANE_String_Const,
    username: *mut SANE_Char,
    password: *mut SANE_Char,
) {
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

struct LibSane;

impl LibSane {
    fn init() -> Result<Self, SaneError> {
        let mut version: i32 = 0;
        unsafe {
            //TODO: Implement callback
            SaneError::from_retcode(sane_init(&mut version as *mut i32, None)).map(|_| LibSane)
        }
    }
}

impl Drop for LibSane {
    fn drop(&mut self) {
        println!("DROP CALLED");
        unsafe { sane_exit() }
    }
}

#[derive(Debug)]
pub struct SaneDevice<'a> {
    pub name: &'a CStr,
    pub vendor: &'a CStr,
    pub model: &'a CStr,
    pub type_: &'a CStr,
}

impl<'a> SaneDevice<'a> {
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
}

struct SaneDeviceList<'sane> {
    devices: *mut *const SANE_Device,
    _phantomdata: PhantomData<&'sane ()>,
}

impl<'sane> SaneDeviceList<'sane> {
    pub fn get_devices(libsane: &'sane LibSane, local_only: bool) -> Result<Self, SaneError> {
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
    device_list: &'devlist SaneDeviceList<'sane>,
    position: isize,
}

impl<'devlist, 'sane> Iterator for SaneDeviceListIter<'devlist, 'sane> {
    type Item = SaneDevice<'devlist>;
    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.device_list.devices.wrapping_offset(self.position);
        unsafe {
            if (*ptr).is_null() {
                None
            } else {
                self.position += 1;
                Some(SaneDevice::from_ptr(*ptr))
            }
        }
    }
}

fn main() -> Result<(), SaneError> {
    let sane = LibSane::init()?;
    let devices = SaneDeviceList::get_devices(&sane, true)?;
    //TODO: Test device lifetime relative to sane!!
    let dev = devices.iter().next().unwrap();
    //drop(devices);
    println!("{:?}", dev);

    /*
    let mut i = 0;
    loop {
    let ptr = devices.wrapping_offset(i);
    if (*ptr).is_null() {
    break;
    }
    i += 1;
    let device = *ptr;
    let name = CStr::from_ptr((*device).name);
    let vendor = CStr::from_ptr((*device).vendor);
    let model = CStr::from_ptr((*device).model);
    let type_ = CStr::from_ptr((*device).type_);
    println!("{:?} {:?} {:?} {:?}", name, vendor, model, type_);
    }
    std::alloc::dealloc(devices);
    */
    Ok(())
}
