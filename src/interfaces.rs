use std::{marker::PhantomData, ptr::NonNull};

use windows::Win32::NetworkManagement::WiFi::{
    WlanFreeMemory, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST,
};

use crate::guid::GuidRef;

/// Object for interacting with a list of Windows WLAN interfacs
#[repr(transparent)]
pub struct WlanInterfaces {
    interface_list_ptr: NonNull<WLAN_INTERFACE_INFO_LIST>,
    _marker: PhantomData<WLAN_INTERFACE_INFO_LIST>,
}

impl WlanInterfaces {
    pub(crate) unsafe fn from_raw(interface_list: *mut WLAN_INTERFACE_INFO_LIST) -> WlanInterfaces {
        Self {
            interface_list_ptr: NonNull::new_unchecked(interface_list),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        unsafe { *self.interface_list_ptr.as_ptr() }.dwNumberOfItems as usize
    }

    pub fn iter<'interfaces>(&'interfaces self) -> WlanInterfacesIterator<'interfaces> {
        let interfaces =
            unsafe { std::ptr::addr_of!((*self.interface_list_ptr.as_ptr()).InterfaceInfo[0]) };

        WlanInterfacesIterator {
            interface_list: self,
            interface_ptr: interfaces,
            index: 0,
            _marker: PhantomData,
        }
    }
}

impl Drop for WlanInterfaces {
    fn drop(&mut self) {
        unsafe { WlanFreeMemory(self.interface_list_ptr.as_ptr().cast()) };
    }
}

pub struct WlanInterfacesIterator<'interfaces> {
    interface_list: &'interfaces WlanInterfaces,
    interface_ptr: *const WLAN_INTERFACE_INFO,
    index: usize,
    _marker: PhantomData<&'interfaces WLAN_INTERFACE_INFO>,
}

impl<'interfaces> Iterator for WlanInterfacesIterator<'interfaces> {
    type Item = WlanInterface<'interfaces>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.interface_list.len() {
            let interface = WlanInterface {
                interface_ptr: self.interface_ptr,
                _marker: PhantomData,
            };

            self.index += 1;

            return Some(interface);
        }

        None
    }
}

impl<'interfaces> ExactSizeIterator for WlanInterfacesIterator<'_> {
    fn len(&self) -> usize {
        self.interface_list.len()
    }
}

#[repr(transparent)]
pub struct WlanInterface<'interface> {
    interface_ptr: *const WLAN_INTERFACE_INFO,
    _marker: PhantomData<&'interface WLAN_INTERFACE_INFO>,
}

impl<'interface> WlanInterface<'interface> {
    pub fn guid(&self) -> GuidRef<'interface> {
        GuidRef::from_ptr(unsafe { std::ptr::addr_of!((*self.interface_ptr).InterfaceGuid) })
    }
}

#[cfg(test)]
mod tests {
    use std::mem::ManuallyDrop;

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

        // Create the interface list. Ensure that the list doesn't get Dropped since
        // the interface list is being allocated on the stack and not from the Windows API
        let interface_list =
            ManuallyDrop::new(unsafe { WlanInterfaces::from_raw(&mut raw_interface_list) });

        // Make sure the returned length matches the correct length
        assert_eq!(interface_list.len(), TEST_INTERFACE_NUMBER as usize);
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

        // Create an interface list
        let interface_list =
            ManuallyDrop::new(unsafe { WlanInterfaces::from_raw(&mut raw_interface_list) });

        // Get the first interface in the interface list
        let interface_list_first = interface_list
            .iter()
            .next()
            .expect("Failed to get the first interface");

        // The internal interface pointer should point to the interface created above
        assert_eq!(interface_list_first.interface_ptr, test_interface_ptr);
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

        // Interface list
        let interface_list =
            ManuallyDrop::new(unsafe { WlanInterfaces::from_raw(&mut raw_interface_list) });

        let mut list_iter = interface_list.iter();

        // This should return a valid interface
        assert!(list_iter.next().is_some());

        // This should go out of bounds
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn zero_sized_list() {
        // Zero sized interface list
        let mut raw_interface_list = WLAN_INTERFACE_INFO_LIST {
            dwNumberOfItems: 0,
            dwIndex: 0,
            InterfaceInfo: [WLAN_INTERFACE_INFO::default()],
        };

        let interface_list =
            ManuallyDrop::new(unsafe { WlanInterfaces::from_raw(&mut raw_interface_list) });

        // There should not be anything in the list
        assert!(interface_list.iter().next().is_none());
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

        let interface_list =
            ManuallyDrop::new(unsafe { WlanInterfaces::from_raw(&mut raw_interface_list) });

        let first_interface = interface_list
            .iter()
            .next()
            .expect("Failed to get first interface");

        let interface_guid = first_interface.guid();

        assert_eq!(interface_guid, TEST_GUID);
    }
}
