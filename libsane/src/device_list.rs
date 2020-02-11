use std::{ffi::CStr, marker::PhantomData};
use libsane_sys::*;
use crate::{device::Device, LibSane, error::{Result, SaneError}};

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

    pub fn open(&self) -> Result<Device> {
        Device::open_device(self.name)
    }
}

pub struct DeviceList<'sane> {
    devices: *mut *const SANE_Device,
    _phantomdata: PhantomData<&'sane ()>,
}

impl<'sane> DeviceList<'sane> {
    pub fn get_devices(_libsane: &'sane LibSane, local_only: bool) -> Result<Self> {
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

    pub fn iter<'devlist>(&'devlist self) -> DeviceListIter<'devlist, 'sane> {
        DeviceListIter {
            device_list: self,
            position: 0,
        }
    }
}

pub struct DeviceListIter<'devlist, 'sane> {
    device_list: &'devlist DeviceList<'sane>,
    position: isize,
}

impl<'devlist, 'sane> Iterator for DeviceListIter<'devlist, 'sane> {
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
