use std::{ffi::OsString, marker::PhantomData, os::windows::ffi::OsStringExt, ptr::NonNull};

use windows::Win32::{
    Foundation::WIN32_ERROR,
    NetworkManagement::WiFi::{
        wlan_interface_state_ad_hoc_network_formed, wlan_interface_state_associating,
        wlan_interface_state_authenticating, wlan_interface_state_connected,
        wlan_interface_state_disconnected, wlan_interface_state_disconnecting,
        wlan_interface_state_discovering, wlan_interface_state_not_ready, WlanEnumInterfaces,
        WlanFreeMemory, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST, WLAN_INTERFACE_STATE,
    },
};

use crate::{
    errors::WinWifiError, guid::GuidRef, handle::WlanHandle, profiles::WlanInterfaceProfiles,
};

/// A wireless interface state
#[repr(i32)]
pub enum WlanInterfaceState {
    /// Wireless interface is not ready
    NotReady = wlan_interface_state_not_ready.0,

    /// Wireless interface is connected
    Connected = wlan_interface_state_connected.0,

    /// An AD HOC network is formed on the wireless interface
    AdHocNetworkFormed = wlan_interface_state_ad_hoc_network_formed.0,

    /// Wireless interface is currently disconnecting
    Disconnecting = wlan_interface_state_disconnecting.0,

    /// Wireless interface is disconnected
    Disconnected = wlan_interface_state_disconnected.0,

    /// Wireless interface is currently associating
    Associating = wlan_interface_state_associating.0,

    /// Wireless interface is currently discovering
    Discovering = wlan_interface_state_discovering.0,

    /// Wireless interface is currently authenticating
    Authenticating = wlan_interface_state_authenticating.0,
}

impl From<WLAN_INTERFACE_STATE> for WlanInterfaceState {
    #[allow(non_upper_case_globals)]
    fn from(value: WLAN_INTERFACE_STATE) -> Self {
        match value {
            wlan_interface_state_not_ready => Self::NotReady,
            wlan_interface_state_connected => Self::Connected,
            wlan_interface_state_ad_hoc_network_formed => Self::AdHocNetworkFormed,
            wlan_interface_state_disconnecting => Self::Disconnecting,
            wlan_interface_state_disconnected => Self::Disconnected,
            wlan_interface_state_associating => Self::Associating,
            wlan_interface_state_discovering => Self::Discovering,
            wlan_interface_state_authenticating => Self::Authenticating,
            _ => unreachable!(),
        }
    }
}

/// List of wireless interfaces
pub struct WlanInterfaces<'handle> {
    handle: &'handle WlanHandle,
    interface_list_ptr: NonNull<WLAN_INTERFACE_INFO_LIST>,
    _interface_list_marker: PhantomData<WLAN_INTERFACE_INFO_LIST>,
}

impl<'handle> WlanInterfaces<'handle> {
    pub fn new(handle: &'handle WlanHandle) -> Result<WlanInterfaces<'handle>, WinWifiError> {
        let mut interface_list_ptr = std::ptr::null_mut();
        WIN32_ERROR(unsafe { WlanEnumInterfaces(*handle.as_ptr(), None, &mut interface_list_ptr) })
            .ok()?;

        Ok(WlanInterfaces {
            handle,
            interface_list_ptr: unsafe { NonNull::new_unchecked(interface_list_ptr) },
            _interface_list_marker: PhantomData,
        })
    }

    #[allow(unused)]
    pub(crate) unsafe fn from_raw(
        handle: &'handle WlanHandle,
        interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST,
    ) -> WlanInterfaces<'handle> {
        WlanInterfaces {
            handle,
            interface_list_ptr: NonNull::new_unchecked(interface_list_ptr),
            _interface_list_marker: PhantomData,
        }
    }

    /// Get the number of wireless interfaces in the list
    pub fn len(&self) -> usize {
        unsafe { *self.interface_list_ptr.as_ptr() }.dwNumberOfItems as usize
    }

    /// Returns an iterator over the wireless interfaces
    pub fn iter<'interfaces>(&'interfaces self) -> WlanInterfacesIterator<'handle, 'interfaces>
    where
        'handle: 'interfaces,
    {
        WlanInterfacesIterator {
            interface_list: self,
            index: 0,
        }
    }
}

impl<'handle> Drop for WlanInterfaces<'_> {
    fn drop(&mut self) {
        unsafe { WlanFreeMemory(self.interface_list_ptr.as_ptr().cast()) };
    }
}

/// Iterator over the list of wireless interfaces
pub struct WlanInterfacesIterator<'handle: 'interfaces, 'interfaces> {
    interface_list: &'interfaces WlanInterfaces<'handle>,
    index: usize,
}

impl<'handle: 'interfaces, 'interfaces> Iterator for WlanInterfacesIterator<'handle, 'interfaces> {
    type Item = WlanInterface<'handle, 'interfaces>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.interface_list.len() {
            let next_interface_ptr = unsafe {
                std::ptr::addr_of!(
                    (*(*self).interface_list.interface_list_ptr.as_ptr()).InterfaceInfo[0]
                )
                .add(1)
            };

            let interface = WlanInterface {
                handle: self.interface_list.handle,
                interface_ptr: next_interface_ptr,
                _interface_marker: PhantomData,
            };

            self.index += 1;

            return Some(interface);
        }

        None
    }
}

impl<'handle: 'interfaces, 'interfaces> ExactSizeIterator for WlanInterfacesIterator<'_, '_> {
    fn len(&self) -> usize {
        self.interface_list.len()
    }
}

pub struct WlanInterface<'handle: 'interfaces, 'interfaces> {
    pub(crate) handle: &'handle WlanHandle,
    interface_ptr: *const WLAN_INTERFACE_INFO,
    _interface_marker: PhantomData<&'interfaces WLAN_INTERFACE_INFO>,
}

impl<'handle: 'interfaces, 'interfaces> WlanInterface<'handle, 'interfaces> {
    /// Returns the interface GUID
    pub fn guid(&self) -> GuidRef<'interfaces> {
        unsafe { GuidRef::from_ptr(std::ptr::addr_of!((*self.interface_ptr).InterfaceGuid)) }
    }

    /// Returns the description of the interface
    pub fn description(&self) -> Option<OsString> {
        let null_index = unsafe { *self.interface_ptr }
            .strInterfaceDescription
            .iter()
            .position(|v| v == &0)?;

        Some(OsString::from_wide(
            &unsafe { (*self.interface_ptr).strInterfaceDescription }[..null_index],
        ))
    }

    /// Returns the state of the interface
    // TODO: Write test
    pub fn if_state(&self) -> WlanInterfaceState {
        unsafe { *self.interface_ptr }.isState.into()
    }

    /// Get the saved profiles associated with this interface
    pub fn profiles(
        &'interfaces self,
    ) -> Result<WlanInterfaceProfiles<'handle, 'interfaces>, WinWifiError> {
        WlanInterfaceProfiles::new(self)
    }
}

#[cfg(test)]
mod tests {
    use std::{mem::ManuallyDrop, os::windows::ffi::OsStrExt};

    use windows::core::GUID;

    use super::*;

    /// Tests that the return value from `WlanInterfaces::len()` matches the correct length
    #[test]
    fn valid_list_length() {
        const TEST_INTERFACE_NUMBER: u32 = 1;

        // Create a mock raw interface list with one interface
        let mut raw_interface_list = WLAN_INTERFACE_INFO_LIST {
            dwNumberOfItems: TEST_INTERFACE_NUMBER,
            dwIndex: 0,
            InterfaceInfo: [WLAN_INTERFACE_INFO::default()],
        };

        let handle = WlanHandle::new_invalid();

        // Create the interface list. Ensure that the list doesn't get Dropped since
        // the interface list is being allocated on the stack and not from the Windows API
        let interface_list = ManuallyDrop::new(unsafe {
            WlanInterfaces::from_raw(&handle, &mut raw_interface_list)
        });

        // Make sure the returned length matches the correct length
        assert_eq!(
            interface_list.len(),
            TEST_INTERFACE_NUMBER as usize,
            "Expected interface length of '{}' does not match found interface length of '{}'",
            TEST_INTERFACE_NUMBER,
            interface_list.len()
        );
    }

    /// Tests that the first wlan interface points to the correct data
    #[test]
    fn correct_first_interface() {
        // Mock interface containing data
        let mut raw_interface_list = WLAN_INTERFACE_INFO_LIST {
            dwNumberOfItems: 1,
            dwIndex: 0,
            InterfaceInfo: [WLAN_INTERFACE_INFO::default()],
        };

        // Pointer to the mock interface
        let test_interface_ptr = std::ptr::addr_of!(raw_interface_list.InterfaceInfo[0]);

        let handle = WlanHandle::new_invalid();

        // Create an interface list
        let interface_list = ManuallyDrop::new(unsafe {
            WlanInterfaces::from_raw(&handle, &mut raw_interface_list)
        });

        // Get the first interface in the interface list
        let interface_list_first = interface_list
            .iter()
            .next()
            .expect("Failed to get the first interface");

        // The internal interface pointer should point to the interface created above
        assert_eq!(
            interface_list_first.interface_ptr,
            test_interface_ptr,
            "Expected interface pointer value of '{:x?}' does not match found interface pointer value of '{:x?}'",
            test_interface_ptr,
            interface_list_first.interface_ptr
        );
    }

    /// Ensure that the wireless interface iterator stays in bounds
    #[test]
    fn list_bounds() {
        // Mock interface list with 1 element
        let mut raw_interface_list = WLAN_INTERFACE_INFO_LIST {
            dwNumberOfItems: 1,
            dwIndex: 0,
            InterfaceInfo: [WLAN_INTERFACE_INFO::default()],
        };

        let handle = WlanHandle::new_invalid();

        // Interface list
        let interface_list = ManuallyDrop::new(unsafe {
            WlanInterfaces::from_raw(&handle, &mut raw_interface_list)
        });

        let mut list_iter = interface_list.iter();

        // This should return a valid interface
        assert!(
            list_iter.next().is_some(),
            "Interface list iterator did not have a first item"
        );

        // This should go out of bounds
        assert!(
            list_iter.next().is_none(),
            "Interface list iterator went out of bounds"
        );
    }

    #[test]
    fn zero_sized_list() {
        // Zero sized interface list
        let mut raw_interface_list = WLAN_INTERFACE_INFO_LIST {
            dwNumberOfItems: 0,
            dwIndex: 0,
            InterfaceInfo: [WLAN_INTERFACE_INFO::default()],
        };

        let handle = WlanHandle::new_invalid();

        let interface_list = ManuallyDrop::new(unsafe {
            WlanInterfaces::from_raw(&handle, &mut raw_interface_list)
        });

        // There should not be anything in the list
        assert!(
            interface_list.iter().next().is_none(),
            "Interface list iterator returned a value in a zero sized list"
        );
    }

    /// Checks that the guid returned from the interface is correct
    #[test]
    fn correct_interface_guid() {
        const TEST_GUID: GUID = GUID::from_values(
            0x12345678,
            0x1234,
            0x1234,
            [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        );

        let mut raw_interface_list = WLAN_INTERFACE_INFO_LIST {
            dwNumberOfItems: 1,
            dwIndex: 0,
            InterfaceInfo: [WLAN_INTERFACE_INFO {
                InterfaceGuid: TEST_GUID,
                ..Default::default()
            }],
        };

        let handle = WlanHandle::new_invalid();

        let interface_list = ManuallyDrop::new(unsafe {
            WlanInterfaces::from_raw(&handle, &mut raw_interface_list)
        });

        let first_interface = interface_list
            .iter()
            .next()
            .expect("Failed to get first interface");

        let interface_guid = first_interface.guid();

        let test_guid_ref = GuidRef::from(&TEST_GUID);
        assert_eq!(
            interface_guid,
            TEST_GUID,
            "Expected GUID value of '{}' did not match found GUID value of '{}'",
            test_guid_ref.to_string(),
            interface_guid.to_string()
        );
    }

    /// Checks for a correct interface description
    #[test]
    fn correct_interface_description() {
        const TEST_DESCRIPTION: &'static str = "testing testing";

        let test_description_os_string = OsString::from(TEST_DESCRIPTION);
        let test_description_bytes = test_description_os_string
            .encode_wide()
            .collect::<Vec<u16>>();

        let mut interface_description = [0u16; 256];
        (&mut interface_description[..test_description_bytes.len()]).copy_from_slice(
            &test_description_os_string
                .encode_wide()
                .collect::<Vec<u16>>(),
        );

        let mut raw_interface_list = WLAN_INTERFACE_INFO_LIST {
            dwNumberOfItems: 1,
            dwIndex: 0,
            InterfaceInfo: [WLAN_INTERFACE_INFO {
                strInterfaceDescription: interface_description,
                ..Default::default()
            }],
        };

        let handle = WlanHandle::new_invalid();
        let interface_list = ManuallyDrop::new(unsafe {
            WlanInterfaces::from_raw(&handle, &mut raw_interface_list)
        });

        let first_interface = interface_list
            .iter()
            .next()
            .expect("Failed to get first interface");

        let found_description = first_interface
            .description()
            .expect("Interface description was malformed");

        assert_eq!(
            test_description_os_string,
            found_description,
            "Expected interface description of '{}' did not match found interface description of '{}'",
            test_description_os_string.to_string_lossy().to_string(),
            found_description.to_string_lossy().to_string()
        );
    }
}
