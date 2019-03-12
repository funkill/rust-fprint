use crate::Driver;
use std::convert::{TryFrom, TryInto};
use std::os::raw::c_int;
use std::os::raw::c_uchar;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;

///
#[derive(Debug, Clone)]
pub struct Device(*mut fprint_sys::fp_dev);

impl Device {
    pub fn new(device: *mut fprint_sys::fp_dev) -> Self {
        Device(device)
    }

    /// Get the `Driver` for a fingerprint device.
    pub fn get_driver(&self) -> Driver {
        let driver = unsafe { fprint_sys::fp_dev_get_driver(self.0) };

        Driver::new(driver)
    }

    /// Gets the number of enroll stages required to enroll a fingerprint with the device.
    pub fn get_nr_enroll_stages(&self) -> i32 {
        unsafe { fprint_sys::fp_dev_get_nr_enroll_stages(self.0) as i32 }
    }

    /// Alias for `get_nr_enroll_stages`
    pub fn get_enroll_stages_count(&self) -> i32 {
        self.get_nr_enroll_stages()
    }

    /// Gets the devtype for a device.
    pub fn get_dev_type(&self) -> u32 {
        unsafe { fprint_sys::fp_dev_get_devtype(self.0) }
    }

    /// Determines if a stored print is compatible with a certain device.
    pub fn supports_print_data(&self, data: &PrintData) -> bool {
        let result = unsafe { fprint_sys::fp_dev_supports_print_data(self.0, data.0) };

        result != 0
    }

    pub fn is_support_print_data(&self, data: &PrintData) -> bool {
        self.supports_print_data(data)
    }

    /// Determines if a device has imaging capabilities. If a device has imaging capabilities
    /// you are able to perform imaging operations such as retrieving scan images using
    /// `img_capture`. However, not all devices are imaging devices – some do all processing
    /// in hardware. This function will indicate which class a device in question falls into.
    pub fn supports_imaging(&self) -> bool {
        let result = unsafe { fprint_sys::fp_dev_supports_imaging(self.0) };

        result == 0
    }

    /// Determines if a device is capable of identification through `identify_finger` and similar.
    /// Not all devices support this functionality.
    pub fn supports_identification(&self) -> bool {
        let result = unsafe { fprint_sys::fp_dev_supports_identification(self.0) };

        result == 0
    }

    /// Gets the expected width of images that will be captured from the device.
    /// If the width of images from this device can vary, 0 will be returned.
    pub fn get_img_width(&self) -> SizeVariant {
        unsafe { fprint_sys::fp_dev_get_img_width(self.0) }.into()
    }

    /// Gets the expected height of images that will be captured from the device.
    /// If the height of images from this device can vary, 0 will be returned.
    pub fn get_img_height(&self) -> SizeVariant {
        unsafe { fprint_sys::fp_dev_get_img_height(self.0) }.into()
    }

    /// Loads a previously stored print from disk. The print must have been saved earlier
    /// using the `PrintData::save_to_disk()` function
    pub fn load_data(&self, finger: Finger) -> crate::Result<PrintData> {
        let mut data: *mut fprint_sys::fp_print_data = std::ptr::null_mut();
        let result = unsafe { fprint_sys::fp_print_data_load(self.0, finger as u32, &mut data) };
        if data.is_null() {
            return Err(crate::FPrintError::NullPtr(crate::NullPtrContext::LoadPrintData));
        }

        if result == -libc::ENOENT {
            return Err(crate::FPrintError::FingerprintNotFound(finger));
        }

        if result != 0 {
            return Err(crate::FPrintError::Obscure(result));
        }

        Ok(PrintData::with_data(data))
    }

    /// Removes a stored print from disk previously saved with `PrintData::save_to_disk()`.
    pub fn delete_data(&self, finger: Finger) -> crate::Result<()> {
        let result = unsafe { fprint_sys::fp_print_data_delete(self.0, finger as u32) };
        assert_eq!({result <= 0}, true);

        if result == 0 {
            Ok(())
        } else {
            Err(crate::FPrintError::RemoveFingerprint(finger))
        }
    }

    /// Captures a fp_img from a device. The returned image is the raw image provided
    /// by the device, you may wish to standardize it.
    ///
    /// If set, the `unconditional` flag indicates that the device should capture an image
    /// unconditionally, regardless of whether a finger is there or not. If unset, this function
    /// will block until a finger is detected on the sensor.
    pub fn capture_image(&self, unconditional: bool) -> crate::Result<Image> {
        let mut image: *mut fprint_sys::fp_img = std::ptr::null_mut();
        let result =
            unsafe { fprint_sys::fp_dev_img_capture(self.0, unconditional as i32, &mut image) };

        match result {
            0 => Ok(Image::new(image)),
            _ if result == -libc::ENOTSUP => Err(crate::FPrintError::NotSupported(crate::NotSupportContext::CapturingImage)),
            res @ _ => Err(crate::FPrintError::Other(res)),
        }
    }

    /// Performs an enroll stage. See [Enrolling](https://fprint.freedesktop.org/libfprint-stable/libfprint-Devices-operations.html#enrolling)
    /// for an explanation of enroll stages.
    ///
    /// If no enrollment is in process, this kicks of the process and runs the first stage.
    /// If an enrollment is already in progress, calling this function runs the next stage,
    /// which may well be the last.
    ///
    /// A negative error code may be returned from any stage. When this occurs, further calls to
    /// the enroll function will start a new enrollment process, i.e. a negative error code
    /// indicates that the enrollment process has been aborted. These error codes only ever
    /// indicate unexpected internal errors or I/O problems.
    ///
    /// The `EnrollResult::Retry` codes may be returned from any enroll stage. These codes
    /// indicate that the scan was not succesful in that the user did not position their finger
    /// correctly or similar. When a `EnrollResult::Retry` code is returned, the enrollment stage
    /// is not advanced, so the next call into this function will retry the current stage again.
    /// The current stage may need to be retried several times.
    ///
    /// The `EnrollResult::Fail` code may be returned from any enroll stage. This code
    /// indicates that even though the scans themselves have been acceptable, data processing
    /// applied to these scans produces incomprehensible results. In other words, the user may
    /// have been scanning a different finger for each stage or something like that. Like
    /// negative error codes, this return code indicates that the enrollment process has been aborted.
    ///
    /// The `EnrollResult::Pass` code will only ever be returned for non-final stages.
    /// This return code indicates that the scan was acceptable and the next call into this
    /// function will advance onto the next enroll stage.
    ///
    /// The `EnrollResult::Complete` code will only ever be returned from the final enroll stage.
    /// It indicates that enrollment completed successfully, and that print_data has been assigned
    /// to point to the resultant enrollment data. The print_data parameter will not be modified
    /// during any other enrollment stages, hence it is actually legal to pass NULL as this
    /// argument for all but the final stage.
    ///
    /// If the device is an imaging device, it can also return the image from the scan, even
    /// when the enroll fails with a `Retry` or `Fail` code. It is legal to call this function
    /// even on non-imaging devices, just don't expect them to provide images.
    pub fn enroll_finger_image(&self, print: &mut PrintData) -> crate::Result<EnrollResult> {
        let mut image: *mut fprint_sys::fp_img = std::ptr::null_mut();
        let result = unsafe { fprint_sys::fp_enroll_finger_img(self.0, &mut print.0, &mut image) };

        if result < 0 {
            Err(crate::FPrintError::UnexpectedAbort(result))
        } else {
            EnrollResult::try_from(result as u32)
        }
    }

    /// Like an `enroll_finger_image` but returns generator what yielded enroll result and
    /// returns print data.
    pub fn enroll<'a>(
        &'a self,
    ) -> impl Generator<Yield = EnrollResult, Return = crate::Result<PrintData>> + 'a {
        move || {
            let mut data = PrintData::new();
            loop {
                let result = self.enroll_finger_image(&mut data);
                match result {
                    Ok(enroll_result) => {
                        if enroll_result == EnrollResult::Complete {
                            if data.0.is_null() {
                                // @todo: need error
                                return Err(crate::FPrintError::Obscure(0));
                            } else {
                                return Ok(data);
                            }
                        } else {
                            yield enroll_result;
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }

    /// Performs a new scan and verify it against a previously enrolled print.
    /// If the device is an imaging device, it can also return the image from the scan, even
    /// when the verify fails with a RETRY code. It is legal to call this function even on
    /// non-imaging devices, just don't expect them to provide images.
    pub fn verify_finger_image(&self, print: &mut PrintData) -> crate::Result<VerifyResult> {
        let mut image: *mut fprint_sys::fp_img = std::ptr::null_mut();
        let result = unsafe { fprint_sys::fp_verify_finger_img(self.0, print.0, &mut image) };

        if result < 0 {
            Err(crate::FPrintError::VerifyFailed(result))
        } else {
            let result = VerifyResult::try_from(result as u32)?;

            match result {
                VerifyResult::Match => Ok(result),
                VerifyResult::NoMatch => Ok(result),
                _ => Err(crate::FPrintError::RetryVerification(result)),
            }
        }
    }

    /// Performs a new scan and attempts to identify the scanned finger against a collection
    /// of previously enrolled fingerprints. If the device is an imaging device, it can also
    /// return the image from the scan, even when identification fails with a RETRY code.
    /// It is legal to call this function even on non-imaging devices, just don't expect
    /// them to provide images.
    ///
    /// This function returns codes from `VerifyResult`. The return code `VerifyResult::Match`
    /// indicates that the scanned fingerprint does appear in the print gallery, and the
    /// match_offset output parameter will indicate the index into the print gallery array of
    /// the matched print.
    ///
    /// This function will not necessarily examine the whole print gallery, it will return
    /// as soon as it finds a matching print.
    ///
    /// Not all devices support identification. -ENOTSUP will be returned when this is the case.
    pub fn identify_finger_image(
        &self,
        gallery: &mut PrintData,
        offset: usize,
    ) -> crate::Result<VerifyResult> {
        let mut image: *mut fprint_sys::fp_img = std::ptr::null_mut();
        let mut offset = offset;
        let result = unsafe {
            fprint_sys::fp_identify_finger_img(self.0, &mut gallery.0, &mut offset, &mut image)
        };

        if result == -libc::ENOTSUP {
            Err(crate::FPrintError::NotSupported(crate::NotSupportContext::Identify))
        } else if result < 0 {
            Err(crate::FPrintError::IdentifyFailed(result))
        } else {
            let result = VerifyResult::try_from(result as u32)?;

            match result {
                VerifyResult::Match => Ok(result),
                VerifyResult::NoMatch => Ok(result),
                _ => Err(crate::FPrintError::RetryVerification(result)),
            }
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { fprint_sys::fp_dev_close(self.0) }
    }
}

pub enum SizeVariant {
    NonImagingDevice,
    Variable,
    Static(u32),
}

impl From<c_int> for SizeVariant {
    fn from(value: c_int) -> Self {
        match value {
            -1 => SizeVariant::NonImagingDevice,
            0 => SizeVariant::Variable,
            s @ _ => SizeVariant::Static(s as u32),
        }
    }
}

#[derive(Debug)]
pub struct PrintData(pub(crate) *mut fprint_sys::fp_print_data);

impl Default for PrintData {
    fn default() -> Self {
        PrintData::new()
    }
}

impl PrintData {
    pub fn new() -> Self {
        let data = std::ptr::null_mut();

        Self::with_data(data)
    }

    pub fn with_data(data: *mut fprint_sys::fp_print_data) -> Self {
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
        let result = unsafe { fprint_sys::fp_print_data_save(self.0, finger as u32) };

        if result == 0 {
            Ok(())
        } else {
            Err(crate::FPrintError::SavePrint(result))
        }
    }

    /// Convert a stored print into a unified representation inside a data buffer.
    /// You can then store this data buffer in any way that suits you, and load it back at
    /// some later time using `PrintData::from_data()` (or `PrintData::try_from(Location)`).
    pub fn get_data(&self) -> crate::Result<Location> {
        let mut buf: *mut c_uchar = std::ptr::null_mut();
        let length = unsafe { fprint_sys::fp_print_data_get_data(self.0, &mut buf) };

        if length == 0 {
            Err(crate::FPrintError::ConvertationFailed)
        } else {
            Ok(Location::new(buf, length))
        }
    }

    /// Load a stored print from a data buffer. The contents of said buffer must be the untouched
    /// contents of a buffer previously supplied to you by the `PrintData::get_data()`.
    pub fn from_data(data: Location) -> crate::Result<Self> {
        data.try_into()
    }

    /// Gets the driver ID for a stored print. The driver ID indicates which driver the print
    /// originally came from. The print is only usable with a device controlled by that driver.
    pub fn get_driver_id(&self) -> u16 {
        unsafe { fprint_sys::fp_print_data_get_driver_id(self.0) }
    }

    /// Gets the devtype for a stored print. The [devtype](https://fprint.freedesktop.org/libfprint-stable/advanced-topics.html#device-types)
    /// represents which type of device under the parent driver is compatible with the print.
    pub fn get_devtype(&self) -> u32 {
        unsafe { fprint_sys::fp_print_data_get_devtype(self.0) }
    }
}

impl TryFrom<Location> for PrintData {
    type Error = crate::FPrintError;

    fn try_from(value: Location) -> Result<Self, Self::Error> {
        let print = unsafe { fprint_sys::fp_print_data_from_data(value.inner, value.length) };

        if print.is_null() {
            // TODO: refactor it!
            Err(crate::FPrintError::TryFromError(0))
        } else {
            Ok(PrintData::with_data(print))
        }
    }
}

impl Drop for PrintData {
    fn drop(&mut self) {
        unsafe { fprint_sys::fp_print_data_free(self.0) }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Finger {
    LeftThumb = 1,
    LeftIndex = 2,
    LeftMiddle = 3,
    LeftRing = 4,
    LeftLittle = 5,
    RightThumb = 6,
    RightIndex = 7,
    RightMiddle = 8,
    RightRing = 9,
    RightLittle = 10,
}

impl Display for Finger {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let string = match self {
            Finger::LeftThumb => "LeftThumb",
            Finger::LeftIndex => "LeftIndex",
            Finger::LeftMiddle => "LeftMiddle",
            Finger::LeftRing => "LeftRing",
            Finger::LeftLittle => "LeftLittle",
            Finger::RightThumb => "RightThumb",
            Finger::RightIndex => "RightIndex",
            Finger::RightMiddle => "RightMiddle",
            Finger::RightRing => "RightRing",
            Finger::RightLittle => "RightLittle",
        };

        write!(f, "{}", string)
    }
}

impl TryFrom<fprint_sys::fp_finger> for Finger {
    type Error = crate::FPrintError;

    fn try_from(value: fprint_sys::fp_finger) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Finger::LeftThumb),
            2 => Ok(Finger::LeftIndex),
            3 => Ok(Finger::LeftMiddle),
            4 => Ok(Finger::LeftRing),
            5 => Ok(Finger::LeftLittle),
            6 => Ok(Finger::RightThumb),
            7 => Ok(Finger::RightIndex),
            8 => Ok(Finger::RightMiddle),
            9 => Ok(Finger::RightRing),
            10 => Ok(Finger::RightLittle),
            n @ _ => Err(crate::FPrintError::TryFromError(n)),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CaptureResult {
    Complete = 0,
    Fail = 1,
}

impl TryFrom<u32> for CaptureResult {
    type Error = crate::FPrintError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CaptureResult::Complete),
            1 => Ok(CaptureResult::Fail),
            n @ _ => Err(crate::FPrintError::TryFromError(n)),
        }
    }
}

pub struct Image(*mut fprint_sys::fp_img);

impl Image {
    pub fn new(image: *mut fprint_sys::fp_img) -> Self {
        Image(image)
    }

    /// Gets the pixel height of an image.
    pub fn get_height(&self) -> i32 {
        unsafe { fprint_sys::fp_img_get_height(self.0) }
    }

    /// Gets the pixel width of an image.
    pub fn get_width(&self) -> i32 {
        unsafe { fprint_sys::fp_img_get_width(self.0) }
    }

    /// Gets the greyscale data for an image. This data must not be modified or freed,
    /// and must not be used after dropping `Image`.
    /// Returns a pointer to libfprint's internal data for the image
    pub fn get_data(&self) -> *const c_uchar {
        unsafe { fprint_sys::fp_img_get_data(self.0) }
    }

    /// A quick convenience function to save an image to a file in [PGM format](http://netpbm.sourceforge.net/doc/pgm.html).
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(crate::FPrintError::PathNotExists);
        }

        let path = path.as_os_str().as_bytes().as_ptr();

        let result = unsafe { fprint_sys::fp_img_save_to_file(self.0, path as *mut i8) };
        if result == 0 {
            Ok(())
        } else {
            Err(crate::FPrintError::SavePrint(result))
        }
    }

    /// [Standardizes](https://fprint.freedesktop.org/libfprint-stable/libfprint-Image-operations.html#img_std)
    /// an image by normalizing its orientation, colors, etc. It is safe to call this multiple
    /// times on an image, `libfprint` keeps track of the work it needs to do to make an image
    /// standard and will not perform these operations more than once for a given image.
    pub fn standardize(&self) {
        unsafe { fprint_sys::fp_img_standardize(self.0) };
    }

    /// Get a binarized form of a standardized scanned image. This is where the fingerprint image
    /// has been "enhanced" and is a set of pure black ridges on a pure white background.
    /// Internally, image processing happens on top of the binarized image.
    ///
    /// The image must have been standardized otherwise this function will fail (this version of
    /// `binarized` standardized image before binarizing).
    ///
    /// It is safe to binarize an image and free the original while continuing to use
    /// the binarized version.
    ///
    /// You cannot binarize an image twice.
    pub fn binarize(&self) -> crate::Result<Self> {
        self.standardize();
        let result = unsafe { fprint_sys::fp_img_binarize(self.0) };

        if result.is_null() {
            Err(crate::FPrintError::NullPtr(crate::NullPtrContext::Binarize))
        } else {
            Ok(Image::new(result))
        }
    }

    //    /// Get a list of minutiae detected in an image. A minutia point is a feature detected on a
    //    /// fingerprint, typically where ridges end or split. libfprint's image processing code relies
    //    /// upon comparing sets of minutiae, so accurate placement of minutia points is critical
    //    /// for good imaging performance.
    //    ///
    //    /// The image must have been standardized otherwise this function will fail.
    //    ///
    //    /// You cannot pass a binarized image to this function. Instead, pass the original image.
    //    ///
    //    /// Returns a list of pointers to minutiae, where the list is of length indicated in the
    //    /// nr_minutiae output parameter. The returned list is only valid while the parent image
    //    /// has not been freed, and the minutiae data must not be modified or freed.
    //    pub fn get_minutiae(&mut self, nr_minutiae: *int) -> Vec<Minutiae> {
    //        self.standardize();
    //
    //        let minutiaes = unsafe { fprint_sys::fp_img_get_minutiae(self.inner, nr_minutiae) };
    //        if minutiaes.is_null() {
    //            Err(())
    //        } else {
    //            let minutiae: *mut fprint_sys::fp_dscv_dev = unsafe { (*minutiaes).offset(0) };
    //            let minutiae: fprint_sys::fp_dscv_dev = unsafe { minutiae.read() };
    //            let minutiae = Minutiae::new(minutiae);
    //
    //            Ok(vec![minutiae])
    //        }
    //    }
}

//struct Minutiae {
//
//}
//
//impl Minutiae {
//    pub fn new(minutiae: *mut fprint_sys::fp_minutia) -> Self {
//
//    }
//}

/// Enrollment result codes returned from `Device::enroll_finger`. Result codes with `RETRY`
/// in the name suggest that the scan failed due to user error. Applications will generally
/// want to inform the user of the problem and then retry the enrollment stage.
///
/// For more info on the semantics of interpreting these result codes and tracking
/// enrollment process, see [Enrolling](https://fprint.freedesktop.org/libfprint-stable/libfprint-Devices-operations.html#enrolling)
#[repr(u32)]
#[derive(Debug)]
pub enum EnrollResult {
    Complete = 1,
    /// Enrollment failed due to incomprehensible data; this may occur when
    /// the user scans a different finger on each enroll stage.
    Fail = 2,
    /// Enroll stage passed; more stages are need to complete the process.
    Pass = 3,
    /// The enrollment scan did not succeed due to poor scan quality or
    /// other general user scanning problem.
    Retry = 100,
    /// The enrollment scan did not succeed because the finger swipe was
    /// too short.
    RetryTooShort = 101,
    /// The enrollment scan did not succeed because the finger was not
    /// centered on the scanner.
    RetryCenterFinger = 102,
    /// The verification scan did not succeed due to quality or pressure
    /// problems; the user should remove their finger from the scanner before
    /// retrying.
    RetryRemoveFinger = 103,
}

impl Display for EnrollResult {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let string = match self {
            EnrollResult::Complete => "Complete",
            EnrollResult::Fail => "Fail",
            EnrollResult::Pass => "Pass",
            EnrollResult::Retry => "Retry",
            EnrollResult::RetryTooShort => "Retry: too short",
            EnrollResult::RetryCenterFinger => "Retry: center finger",
            EnrollResult::RetryRemoveFinger => "Retry: remove finger",
        };

        write!(f, "{}", string)
    }
}

impl TryFrom<u32> for EnrollResult {
    type Error = crate::FPrintError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(EnrollResult::Complete),
            2 => Ok(EnrollResult::Fail),
            3 => Ok(EnrollResult::Pass),
            100 => Ok(EnrollResult::Retry),
            101 => Ok(EnrollResult::RetryTooShort),
            102 => Ok(EnrollResult::RetryCenterFinger),
            103 => Ok(EnrollResult::RetryRemoveFinger),
            n @ _ => Err(crate::FPrintError::TryFromError(n)),
        }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum VerifyResult {
    /// The scan completed successfully, but the newly scanned fingerprint
    /// does not match the fingerprint being verified against.
    /// In the case of identification, this return code indicates that the
    /// scanned finger could not be found in the print gallery.
    NoMatch = 0,
    /// The scan completed successfully and the newly scanned fingerprint does
    /// match the fingerprint being verified, or in the case of identification,
    /// the scanned fingerprint was found in the print gallery.
    Match = 1,
    /// The scan did not succeed due to poor scan quality or other general
    /// user scanning problem.
    Retry = EnrollResult::Retry as u32,
    /// The scan did not succeed because the finger swipe was too short.
    RetryTooShort = EnrollResult::RetryTooShort as u32,
    /// The scan did not succeed because the finger was not centered on the scanner.
    RetryCenterFinger = EnrollResult::RetryCenterFinger as u32,
    /// The scan did not succeed due to quality or pressure problems; the user
    /// should remove their finger from the scanner before retrying.
    RetryRemoveFinger = EnrollResult::RetryRemoveFinger as u32,
}

impl Display for VerifyResult {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let string = match self {
            VerifyResult::NoMatch => "NoMatch",
            VerifyResult::Match => "Match",
            VerifyResult::Retry => "Retry",
            VerifyResult::RetryTooShort => "RetryTooShort",
            VerifyResult::RetryCenterFinger => "RetryCenterFinger",
            VerifyResult::RetryRemoveFinger => "RetryRemoveFinger",
        };

        write!(f, "{}", string)
    }
}

impl TryFrom<u32> for VerifyResult {
    type Error = crate::FPrintError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VerifyResult::NoMatch),
            1 => Ok(VerifyResult::Match),
            n if (n == EnrollResult::Retry as u32) => Ok(VerifyResult::Retry),
            n if (n == EnrollResult::RetryCenterFinger as u32) => {
                Ok(VerifyResult::RetryCenterFinger)
            }
            n if (n == EnrollResult::RetryRemoveFinger as u32) => {
                Ok(VerifyResult::RetryRemoveFinger)
            }
            n @ _ => Err(crate::FPrintError::TryFromError(n)),
        }
    }
}

#[derive(Debug)]
pub struct Location {
    inner: *mut c_uchar,
    length: usize,
}

impl Location {
    pub fn new(loc: *mut c_uchar, length: usize) -> Self {
        Location { inner: loc, length }
    }
}
