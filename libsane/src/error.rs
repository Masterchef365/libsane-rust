use libsane_sys::*;
use std::fmt;

pub type Result<T> = std::result::Result<T, SaneError>;

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
    pub fn from_retcode(code: SANE_Status) -> std::result::Result<(), Self> {
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
