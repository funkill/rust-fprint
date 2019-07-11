use crate::finger::Finger;
use std::os::raw::c_uchar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintData(pub(crate) *mut crate::bindings::fp_print_data);

impl PrintData {
    /// Created PrintData without data
    pub fn new() -> Self {
        let data = std::ptr::null_mut();

        Self::with_data(data)
    }

    pub fn with_data(data: *mut crate::bindings::fp_print_data) -> Self {
        PrintData(data)
    }

    /// Saves a stored print to disk, assigned to a specific finger. Even though you are limited
    /// to storing only the 10 human fingers, this is a per-device-type limit.
    /// For example, you can store the users right index finger from a DigitalPersona scanner,
    /// and you can also save the right index finger from a UPEK scanner. When you later
    /// come to load the print, the right one will be automatically selected.
    ///
    /// This function will unconditionally overwrite a fingerprint previously saved for the same
    /// finger and device type. The print is saved in a hidden directory beneath the current
    /// user's home directory.
    pub fn save_to_disk(&self, finger: Finger) -> crate::Result<()> {
        let result = unsafe { crate::bindings::fp_print_data_save(self.0, finger as u32) };

        if result == 0 {
            Ok(())
        } else {
            Err(crate::FPrintError::SavePrint(result))
        }
    }

    /// Convert a stored print into a unified representation inside a data buffer.
    /// You can then store this data buffer in any way that suits you, and load it back at
    /// some later time using `PrintData::from_data()` (or `PrintData::try_from(Location)`).
    pub fn get_data(&self) -> crate::Result<&[u8]> {
        self.as_bytes()
    }

    pub fn as_bytes(&self) -> crate::Result<&[u8]> {
        let mut buf: *mut c_uchar = std::ptr::null_mut();
        let length = unsafe { crate::bindings::fp_print_data_get_data(self.0, &mut buf) };

        if length == 0 {
            Err(crate::FPrintError::ConvertationFailed)
        } else {
            let data = unsafe { std::slice::from_raw_parts(buf, length) };

            Ok(data)
        }
    }

    /// Load a stored print from a data buffer. The contents of said buffer must be the untouched
    /// contents of a buffer previously supplied to you by the `PrintData::get_data()`.
    pub fn from_data(data: &[u8]) -> crate::Result<Self> {
        Self::from_bytes(data)
    }

    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> crate::Result<Self> {
        Self::from_bytes_raw(bytes).map(Self::with_data)
    }

    pub(crate) fn from_bytes_raw(
        bytes: impl AsRef<[u8]>,
    ) -> crate::Result<*mut crate::bindings::fp_print_data> {
        let bytes = bytes.as_ref();
        let len = bytes.len();
        let value = bytes.as_ptr() as *mut c_uchar;
        let print = unsafe { crate::bindings::fp_print_data_from_data(value, len) };

        if print.is_null() {
            // TODO: refactor it!
            Err(crate::FPrintError::NeedError)
        } else {
            Ok(print)
        }
    }

    /// Gets the driver ID for a stored print. The driver ID indicates which driver the print
    /// originally came from. The print is only usable with a device controlled by that driver.
    pub fn get_driver_id(&self) -> u16 {
        unsafe { crate::bindings::fp_print_data_get_driver_id(self.0) }
    }

    /// Gets the devtype for a stored print. The [devtype](https://fprint.freedesktop.org/libfprint-stable/advanced-topics.html#device-types)
    /// represents which type of device under the parent driver is compatible with the print.
    pub fn get_devtype(&self) -> u32 {
        unsafe { crate::bindings::fp_print_data_get_devtype(self.0) }
    }
}

impl Default for PrintData {
    fn default() -> Self {
        PrintData::new()
    }
}

impl Drop for PrintData {
    fn drop(&mut self) {
        unsafe { crate::bindings::fp_print_data_free(self.0) }
    }
}
