use libsane_sys::*;
use std::{ffi::CStr, marker::PhantomData};
use crate::error::{Result, SaneError};
use crate::option_descriptor::OptionDescriptorIterator;

pub struct Device<'sane> {
    handle: SANE_Handle,
    _phantomdata: PhantomData<&'sane ()>,
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
}

impl Drop for Device<'_> {
    fn drop(&mut self) {
        unsafe {
            sane_close(self.handle);
        }
    }
}
