use crate::{Device, Driver, PrintData};
use std::mem::{size_of, size_of_val};

/// These functions allow you to scan the system for supported fingerprint scanning hardware.
/// This is your starting point when integrating libfprint into your software.
///
/// When you've identified a discovered device that you would like to control, you can open it
/// with `open()`. Note that discovered devices may no longer be available at the time when you
/// want to open them, for example the user may have unplugged the device.
#[derive(Debug, Clone)]
pub struct DiscoveredDev(*mut fprint_sys::fp_dscv_dev);

impl DiscoveredDev {
    pub fn new(inner: *mut fprint_sys::fp_dscv_dev) -> Self {
        DiscoveredDev(inner)
    }

    /// Gets the `Driver` for a discovered device.
    pub fn get_driver(&self) -> Driver {
        let driver = unsafe { fprint_sys::fp_dscv_dev_get_driver(self.0) };

        Driver::new(driver)
    }

    /// Gets the devtype for a discovered device.
    pub fn get_devtype(&self) -> u32 {
        unsafe { fprint_sys::fp_dscv_dev_get_devtype(self.0) }
    }

    /// Determines if a specific `PrintData` stored print appears to be compatible
    /// with a discovered device.
    pub fn supports_print_data(&self, data: &mut PrintData) -> bool {
        let result = unsafe { fprint_sys::fp_dscv_dev_supports_print_data(self.0, data.0) };

        result == 1
    }

    /// Opens and initialises a device. This is the function you call in order to convert
    /// a discovered device into an actual device handle that you can perform operations with.
    pub fn open(&self) -> Device {
        let device = unsafe { fprint_sys::fp_dev_open(self.0) };

        Device::new(device)
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevices {
    inner: *mut *mut fprint_sys::fp_dscv_dev,
    current_item_number: isize,
}

impl Iterator for DiscoveredDevices {
    type Item = DiscoveredDev;

    fn next(&mut self) -> Option<Self::Item> {
        let item = unsafe { self.inner.offset(self.current_item_number) };
        let device: *mut fprint_sys::fp_dscv_dev = unsafe { item.read() };
        if device.is_null() {
            None
        } else {
            Some(DiscoveredDev::new(device))
        }
    }
}

impl DiscoveredDevices {
    pub fn new() -> Self {
        let devices = std::ptr::null_mut();

        Self::with_devices(devices)
    }

    pub fn with_devices(devices: *mut *mut fprint_sys::fp_dscv_dev) -> Self {
        DiscoveredDevices {
            inner: devices,
            current_item_number: 0,
        }
    }

    pub fn get(&self, index: isize) -> Option<DiscoveredDev> {
        if index as usize >= self.count() {
            return None;
        }

        let item = unsafe { self.inner.offset(index) };
        let device: *mut fprint_sys::fp_dscv_dev = unsafe { item.read() };

        if device.is_null() {
            None
        } else {
            Some(DiscoveredDev::new(device))
        }
    }

    pub fn count(&self) -> usize {
        size_of_val(&self.inner) / size_of::<*mut fprint_sys::fp_dscv_dev>()
    }
}

impl Default for DiscoveredDevices {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DiscoveredDevices {
    fn drop(&mut self) {
        // If inner is null all ok, because fp_dscv_devs_free simply returns if des is null.
        unsafe { fprint_sys::fp_dscv_devs_free(self.inner) };
    }
}
