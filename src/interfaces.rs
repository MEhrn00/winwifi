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
pub struct WlanInterfaces {
    /// The wlan handle associated with this list of wireless interfaces
    handle: WlanHandle,

    /// Pointer to the raw list of interfaces
    interface_list_ptr: NonNull<WLAN_INTERFACE_INFO_LIST>,

    /// Marker for declaring ownership of the interface_list_ptr memory
    _marker: PhantomData<WLAN_INTERFACE_INFO_LIST>,
}

impl WlanInterfaces {
    /// Gets the list of wireless interfaces on the system
    pub fn new() -> Result<WlanInterfaces, WinWifiError> {
        let handle = WlanHandle::new()?;
        Self::with_handle(handle)
    }

    /// Gets the list of wireless interfaces on the system but using an already opened WlanHandle
    pub fn with_handle(handle: WlanHandle) -> Result<WlanInterfaces, WinWifiError> {
        let mut interface_list_ptr = std::ptr::null_mut();
        WIN32_ERROR(unsafe { WlanEnumInterfaces(*handle.as_ptr(), None, &mut interface_list_ptr) })
            .ok()?;

        Ok(WlanInterfaces {
            handle,
            interface_list_ptr: unsafe { NonNull::new_unchecked(interface_list_ptr) },
            _marker: PhantomData,
        })
    }

    #[allow(unused)]
    /// Creates a new list of wireless interfaces from a handle and a pointer to a list of interfaces.
    /// Used for testing
    pub(crate) unsafe fn from_raw_parts(
        handle: WlanHandle,
        interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST,
    ) -> WlanInterfaces {
        WlanInterfaces {
            handle,
            interface_list_ptr: NonNull::new_unchecked(interface_list_ptr),
            _marker: PhantomData,
        }
    }

    /// Get the number of wireless interfaces in the list
    pub const fn len(&self) -> usize {
        let interface = unsafe { self.interface_list_ptr.as_ref() };
        interface.dwNumberOfItems as usize
    }

    pub const fn is_empty(&self) -> bool {
        let interface = unsafe { self.interface_list_ptr.as_ref() };
        interface.dwNumberOfItems == 0
    }

    /// Returns an iterator by reference over the wireless interfaces
    pub fn iter<'interfaces, 'handle>(&'handle self) -> WlanInterfacesIterator<'interfaces, 'handle>
    where
        'handle: 'interfaces,
    {
        WlanInterfacesIterator {
            handle: &self.handle,
            item_count: self.len(),
            index: 0,
            interface_list_ptr: self.interface_list_ptr,
            _marker: PhantomData,
        }
    }
}

impl Drop for WlanInterfaces {
    fn drop(&mut self) {
        unsafe { WlanFreeMemory(self.interface_list_ptr.as_ptr().cast()) };
    }
}

/// Iterator over the list of wireless interfaces
pub struct WlanInterfacesIterator<'interfaces, 'handle: 'interfaces> {
    /// The wlan handle associated with this list of wireless interfaces
    handle: &'handle WlanHandle,

    /// The number of items in the list of wireless interfaces
    item_count: usize,

    /// The current indes into the list of wireless interfaces
    index: usize,

    /// Pointer to the underlying memory containing the wireless interface list
    interface_list_ptr: NonNull<WLAN_INTERFACE_INFO_LIST>,

    /// Marker signifying that the interface_list_ptr does NOT own the underlying memory
    _marker: PhantomData<&'interfaces WLAN_INTERFACE_INFO_LIST>,
}

impl<'interfaces, 'handle: 'interfaces> Iterator for WlanInterfacesIterator<'interfaces, 'handle> {
    type Item = WlanInterface<'interfaces, 'handle>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.item_count {
            let next_interface_ptr = unsafe {
                let interface_list = self.interface_list_ptr.as_mut();
                std::ptr::addr_of_mut!(interface_list.InterfaceInfo[0]).add(self.index)
            };

            let interface = WlanInterface {
                handle: self.handle,
                interface_ptr: unsafe { NonNull::new_unchecked(next_interface_ptr) },
                _marker: PhantomData,
            };

            self.index += 1;

            return Some(interface);
        }

        None
    }
}

impl<'interfaces, 'handle: 'interfaces> ExactSizeIterator for WlanInterfacesIterator<'_, '_> {
    fn len(&self) -> usize {
        self.item_count
    }
}

/// A wireless interface
pub struct WlanInterface<'interfaces, 'handle: 'interfaces> {
    /// Wlan handle associated with this wireless interface
    pub(crate) handle: &'handle WlanHandle,

    /// Pointer to the underlying wireless interface
    interface_ptr: NonNull<WLAN_INTERFACE_INFO>,

    /// Marker signifying that the interface_ptr does NOT own the underlying memory
    _marker: PhantomData<&'interfaces WLAN_INTERFACE_INFO>,
}

impl<'interfaces, 'handle: 'interfaces> WlanInterface<'interfaces, 'handle> {
    /// Returns the interface GUID
    pub fn guid(&self) -> GuidRef<'interfaces> {
        let interface = unsafe { self.interface_ptr.as_ref() };
        GuidRef::from(&interface.InterfaceGuid)
    }

    /// Returns the description of the interface
    pub fn description(&self) -> Option<OsString> {
        let interface = unsafe { self.interface_ptr.as_ref() };

        let null_index = interface
            .strInterfaceDescription
            .iter()
            .position(|v| v == &0)?;

        Some(OsString::from_wide(
            &interface.strInterfaceDescription[..null_index],
        ))
    }

    /// Returns the state of the interface
    // TODO: Write test
    pub fn if_state(&self) -> WlanInterfaceState {
        let interface = unsafe { self.interface_ptr.as_ref() };
        interface.isState.into()
    }

    /// Get the saved profiles associated with this interface
    pub fn profiles(
        &'interfaces self,
    ) -> Result<WlanInterfaceProfiles<'interfaces, 'handle>, WinWifiError> {
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
            WlanInterfaces::from_raw_parts(handle, &mut raw_interface_list)
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
            WlanInterfaces::from_raw_parts(handle, &mut raw_interface_list)
        });

        // Get the first interface in the interface list
        let interface_list_first = interface_list
            .iter()
            .next()
            .expect("Failed to get the first interface");

        // The internal interface pointer should point to the interface created above
        assert_eq!(
            interface_list_first.interface_ptr.as_ptr().cast_const(),
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
            WlanInterfaces::from_raw_parts(handle, &mut raw_interface_list)
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
            WlanInterfaces::from_raw_parts(handle, &mut raw_interface_list)
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
            WlanInterfaces::from_raw_parts(handle, &mut raw_interface_list)
        });

        let first_interface = interface_list
            .iter()
            .next()
            .expect("Failed to get first interface");

        let interface_guid = first_interface.guid();

        let test_guid_ref = GuidRef::from(&TEST_GUID);
        assert_eq!(
            interface_guid, TEST_GUID,
            "Expected GUID value of '{}' did not match found GUID value of '{}'",
            test_guid_ref, interface_guid
        );
    }

    /// Checks for a correct interface description
    #[test]
    fn correct_interface_description() {
        const TEST_DESCRIPTION: &str = "testing testing";

        let test_description_os_string = OsString::from(TEST_DESCRIPTION);
        let test_description_bytes = test_description_os_string
            .encode_wide()
            .collect::<Vec<u16>>();

        let mut interface_description = [0u16; 256];
        interface_description[..test_description_bytes.len()].copy_from_slice(
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
            WlanInterfaces::from_raw_parts(handle, &mut raw_interface_list)
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
            test_description_os_string.to_string_lossy(),
            found_description.to_string_lossy()
        );
    }
}
