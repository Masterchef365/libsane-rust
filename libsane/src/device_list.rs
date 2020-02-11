use std::{ffi::CStr, marker::PhantomData};
use libsane_sys::*;
use crate::{device::Device, LibSane, error::{Result, SaneError}};

#[derive(Debug)]
pub struct DeviceDescription<'sane> {
    pub name: &'sane CStr,
    pub vendor: &'sane CStr,
    pub model: &'sane CStr,
    pub type_: &'sane CStr,
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

    pub fn open(&self, _: &LibSane) -> Result<Device> {
        Device::open_device(self.name)
    }
}

pub struct DeviceListIter<'sane> {
    devices: *mut *const SANE_Device,
    position: isize,
    _phantomdata: PhantomData<&'sane ()>,
}

impl<'sane> DeviceListIter<'sane> {
    pub(crate) fn new(_libsane: &'sane LibSane, local_only: bool) -> Result<Self> {
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
            position: 0,
            _phantomdata: PhantomData,
        })
    }
}

impl<'sane> Iterator for DeviceListIter<'sane> {
    type Item = DeviceDescription<'sane>;
    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.devices.wrapping_offset(self.position);

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
