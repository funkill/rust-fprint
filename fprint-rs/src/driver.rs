use std::convert::TryFrom;
use std::ffi::CStr;

/// Internally, libfprint is abstracted into various drivers to communicate with the different types
/// of supported fingerprint readers. libfprint works hard so that you don't have to care about
/// these internal abstractions, however there are some situations where you may be interested
/// in a little behind-the-scenes driver info.
#[derive(Debug)]
pub struct Driver(*mut fprint_sys::fp_driver);

impl Driver {
    pub fn new(driver: *mut fprint_sys::fp_driver) -> Self {
        Driver(driver)
    }

    /// Retrieves the name of the driver. For example: "upekts"
    pub fn get_name(&self) -> String {
        unsafe {
            let name = fprint_sys::fp_driver_get_name(self.0);

            CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    }

    /// Retrieves a descriptive name of the driver. For example: "UPEK TouchStrip"
    pub fn get_full_name(&self) -> String {
        unsafe {
            let full_name = fprint_sys::fp_driver_get_full_name(self.0);

            CStr::from_ptr(full_name).to_string_lossy().into_owned()
        }
    }

    /// Retrieves the driver ID code for a driver.
    pub fn get_driver_id(&self) -> u16 {
        unsafe { fprint_sys::fp_driver_get_driver_id(self.0) }
    }

    /// Retrieves the scan type for the devices associated with the driver.
    pub fn get_scan_type(&self) -> crate::Result<ScanType> {
        let scan_type = unsafe { fprint_sys::fp_driver_get_scan_type(self.0) };

        ScanType::try_from(scan_type)
    }
}

/// Devices require either swiping or pressing the finger on the device. This is useful for front-ends.
#[derive(Debug, Eq, PartialEq)]
pub enum ScanType {
    /// the reader has a surface area that covers the whole finger
    Press,
    /// the reader requires swiping the finger on a smaller area
    Swipe,
}

impl TryFrom<u32> for ScanType {
    type Error = crate::FPrintError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            fprint_sys::fp_scan_type_FP_SCAN_TYPE_PRESS => Ok(ScanType::Press),
            fprint_sys::fp_scan_type_FP_SCAN_TYPE_SWIPE => Ok(ScanType::Swipe),
            n => Err(crate::FPrintError::TryFromError(n)),
        }
    }
}
