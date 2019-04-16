use crate::device::{EnrollResult, Finger, VerifyResult};
use failure::Fail;

// TODO: refactor it!
#[derive(Debug, Fail)]
pub enum FPrintError {
    #[fail(display = "Fail on init. FPrint error code: {}", _0)]
    InitError(i32),
    #[fail(display = "Null ptr found: {}", _0)]
    NullPtr(NullPtrContext),
    #[fail(display = "Requested fingerprint not found (finger: {})", _0)]
    FingerprintNotFound(Finger),
    #[fail(display = "Obscure error conditions (e.g. corruption): {}", _0)]
    Obscure(i32),
    #[fail(display = "Failed removing fingerprint for finger `{}`", _0)]
    RemoveFingerprint(Finger),
    #[fail(display = "Not supported: {}", _0)]
    NotSupported(NotSupportContext),
    #[fail(
        display = "Error not covered by original documentation. Error code: {}",
        _0
    )]
    Other(i32),
    #[fail(
        display = "The enrollment process has been aborted. These error codes only ever indicate unexpected internal errors or I/O problems. Code: {}",
        _0
    )]
    UnexpectedAbort(i32),
    #[fail(
        display = "Enroll image process fails or required retry. Original error: {}",
        _0
    )]
    EnrollImage(EnrollResult),
    #[fail(display = "Verifying fingerprint failed. Error code: {}", _0)]
    VerifyFailed(i32),
    #[fail(display = "Retry verification. Reason: {}", _0)]
    RetryVerification(VerifyResult),
    #[fail(display = "Identify failed. Error code: {}", _0)]
    IdentifyFailed(i32),
    #[fail(display = "Failed to save print data. Error code: {}", _0)]
    SavePrint(i32),
    #[fail(display = "Can not convert stored print into unified representation")]
    ConvertationFailed,
    #[fail(display = "Can not convert from `{}`", _0)]
    TryFromError(u32),
    #[fail(display = "Path not exists")]
    PathNotExists,
    #[fail(display = "Error not specified. Please, write issue")]
    NeedError,
}

#[derive(Debug, Fail)]
pub enum NullPtrContext {
    #[fail(display = "on discovering devices")]
    Discovering,
    #[fail(display = "on loading print data")]
    LoadPrintData,
    #[fail(display = "on binarize")]
    Binarize,
    #[fail(display = "on create discovering device")]
    CreateDiscoveringDevice,
}

#[derive(Debug, Fail)]
pub enum NotSupportContext {
    #[fail(
        display = "either the unconditional flag was set but the device does not support this, or that the device does not support imaging"
    )]
    CapturingImage,
    #[fail(display = "device not support identification")]
    Identify,
}
