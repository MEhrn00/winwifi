use windows::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE, WIN32_ERROR},
    NetworkManagement::WiFi::{
        WlanCloseHandle, WlanEnumInterfaces, WlanOpenHandle, WLAN_API_VERSION_1_0,
        WLAN_API_VERSION_2_0,
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

impl Into<u32> for WlanApiVersion {
    fn into(self) -> u32 {
        match self {
            Self::ApiVersion1 => WLAN_API_VERSION_1_0,
            Self::ApiVersion2 => WLAN_API_VERSION_2_0,
        }
    }
}

/// Object for interacting with the Windows WLAN subsystem
pub struct WlanHandle {
    handle: HANDLE,
}

impl WlanHandle {
    pub fn new() -> Result<WlanHandle, WinWifiError> {
        Self::with_api_version(WlanApiVersion::ApiVersion2)
    }

    pub fn with_api_version(version: WlanApiVersion) -> Result<WlanHandle, WinWifiError> {
        let mut negotiated_version = 0;
        let mut handle = INVALID_HANDLE_VALUE;

        WIN32_ERROR(unsafe {
            WlanOpenHandle(version.into(), None, &mut negotiated_version, &mut handle)
        })
        .ok()?;

        Ok(WlanHandle { handle })
    }

    pub fn interfaces(&self) -> Result<WlanInterfaces, WinWifiError> {
        let mut interface_list_ptr = std::ptr::null_mut();
        WIN32_ERROR(unsafe { WlanEnumInterfaces(self.handle, None, &mut interface_list_ptr) })
            .ok()?;

        Ok(unsafe { WlanInterfaces::from_raw(interface_list_ptr) })
    }
}

impl Drop for WlanHandle {
    fn drop(&mut self) {
        unsafe { WlanCloseHandle(self.handle, None) };
    }
}
