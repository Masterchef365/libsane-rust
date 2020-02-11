#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
mod error;
mod device_list;
mod device;
mod option_descriptor;
pub use error::{SaneError, Result};
use device_list::DeviceList;
use device::Device;

use libsane_sys::*;

/// Libsane C library representation
pub struct LibSane;

impl LibSane {
    pub fn init(callback: SANE_Auth_Callback) -> Result<Self> {
        let mut version: i32 = 0;
        unsafe {
            //TODO: Implement callback
            SaneError::from_retcode(sane_init(&mut version as *mut i32, callback)).map(|_| LibSane)
        }
    }

    pub fn device_names<'a>(&'a self, local_only: bool) -> Result<DeviceList<'a>> {
        DeviceList::get_devices(self, local_only)
    }

    pub fn open<'a>(&'a self, name: &str) -> Result<Device<'a>> {
        let name = std::ffi::CString::new(name).expect("Invalid C String");
        Device::open_device(&name)
    }
}

impl Drop for LibSane {
    fn drop(&mut self) {
        unsafe { sane_exit() }
    }
}
