#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
mod device;
mod device_list;
mod error;
mod option_descriptor;
//pub use device::{Device, Value};
pub use device::*;
pub use device_list::{DeviceDescription, DeviceListIter};
pub use error::{Result, SaneError};
pub use option_descriptor::*;

use libsane_sys::*;

/// LibSANE C library representation
pub struct LibSane;

impl LibSane {
    /// Initialize the LibSANE library
    pub fn init(callback: SANE_Auth_Callback) -> Result<Self> {
        let mut version: i32 = 0;
        unsafe {
            SaneError::from_retcode(sane_init(&mut version as *mut i32, callback)).map(|_| LibSane)
        }
    }

    /// Return an iterator over available device descriptions
    pub fn list_devices<'a>(&'a self, local_only: bool) -> Result<DeviceListIter<'a>> {
        DeviceListIter::new(self, local_only)
    }

    /// Open the device with name `name`
    pub fn open_device<'a>(&'a self, name: &str) -> Result<Device<'a>> {
        let name = std::ffi::CString::new(name).expect("Invalid C String");
        Device::open_device(&name)
    }
}

impl Drop for LibSane {
    fn drop(&mut self) {
        unsafe { sane_exit() }
    }
}
