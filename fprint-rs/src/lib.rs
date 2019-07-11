#![warn(clippy::all)]

mod device;
mod discovered_device;
mod driver;
mod errors;
mod finger;
mod print_data;

pub use crate::{device::*, discovered_device::*, driver::*, errors::*, finger::*, print_data::*};

pub type Result<T> = std::result::Result<T, FPrintError>;

#[derive(Debug, Clone)]
pub struct FPrint;

impl FPrint {
    /// Initialise libfprint.
    ///
    /// To enable debug output of libfprint specifically, use GLib's `G_MESSAGES_DEBUG` environment
    /// variable as explained in Running and debugging GLib Applications.
    ///
    /// The log domains used in `libfprint` are either `libfprint` or `libfprint-FP_COMPONENT` where
    /// `FP_COMPONENT` is defined in the source code for each driver, or component of the library.
    /// Starting with all and trimming down is advised.
    ///
    /// To enable debugging of `libusb`, for USB-based fingerprint reader drivers, use
    /// libusb's `LIBUSB_DEBUG` environment variable as explained in the libusb-1.0 API Reference.
    ///
    /// Example:
    /// ```bash
    /// # LIBUSB_DEBUG=4 G_MESSAGES_DEBUG=all my-libfprint-application
    /// ```
    pub fn new() -> crate::Result<FPrint> {
        let res = unsafe { fprint_sys::fp_init() } as i32;

        if res == 0 {
            Ok(FPrint)
        } else {
            Err(crate::FPrintError::InitError(res))
        }
    }

    /// Scans the system and returns a list of discovered devices. This is your entry point
    /// into finding a fingerprint reader to operate.
    pub fn discover(&self) -> DiscoveredDevices {
        let devices_list = unsafe { fprint_sys::fp_discover_devs() };

        DiscoveredDevices::with_devices(devices_list)
    }
}

impl Drop for FPrint {
    fn drop(&mut self) {
        unsafe {
            fprint_sys::fp_exit();
        }
    }
}
