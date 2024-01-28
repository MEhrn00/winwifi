use windows::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE, WIN32_ERROR},
    NetworkManagement::WiFi::{
        WlanCloseHandle, WlanOpenHandle, WLAN_API_VERSION_1_0, WLAN_API_VERSION_2_0,
    },
};

use crate::{errors::WinWifiError, interfaces::WlanInterfaces};

/// Windows WLAN Api version. Refer to https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nf-wlanapi-wlanopenhandle for more details.
#[derive(Default)]
#[repr(u32)]
pub enum WlanApiVersion {
    /// WLAN client api version 1. Client version for Windows XP with SP3 and Wireless LAN API for Windows XP with SP2.
    ApiVersion1 = WLAN_API_VERSION_1_0,

    /// WLAN client api version 2. Client version for Windows Vista and Windows Server 2008
    #[default]
    ApiVersion2 = WLAN_API_VERSION_2_0,
}

impl From<WlanApiVersion> for u32 {
    fn from(val: WlanApiVersion) -> Self {
        match val {
            WlanApiVersion::ApiVersion1 => WLAN_API_VERSION_1_0,
            WlanApiVersion::ApiVersion2 => WLAN_API_VERSION_2_0,
        }
    }
}

/// Object for interacting with the Windows WLAN subsystem
#[repr(transparent)]
pub struct WlanHandle(HANDLE);

impl WlanHandle {
    pub fn new() -> Result<WlanHandle, WinWifiError> {
        Self::with_api_version(WlanApiVersion::ApiVersion2)
    }

    #[allow(unused)]
    pub(crate) const fn new_invalid() -> WlanHandle {
        Self(INVALID_HANDLE_VALUE)
    }

    pub(crate) const unsafe fn as_ptr(&self) -> *const HANDLE {
        &self.0
    }

    pub fn with_api_version(version: WlanApiVersion) -> Result<WlanHandle, WinWifiError> {
        let mut negotiated_version = 0;
        let mut handle = INVALID_HANDLE_VALUE;

        WIN32_ERROR(unsafe {
            WlanOpenHandle(version.into(), None, &mut negotiated_version, &mut handle)
        })
        .ok()?;

        Ok(WlanHandle(handle))
    }

    pub fn into_interfaces(self) -> Result<WlanInterfaces, WinWifiError> {
        WlanInterfaces::with_handle(self)
    }
}

impl Drop for WlanHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { WlanCloseHandle(self.0, None) };
        }
    }
}
