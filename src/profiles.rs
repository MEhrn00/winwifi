use std::{marker::PhantomData, ptr::NonNull};

use windows::Win32::{
    Foundation::WIN32_ERROR,
    NetworkManagement::WiFi::{
        WlanGetProfileList, WLAN_PROFILE_GROUP_POLICY, WLAN_PROFILE_INFO, WLAN_PROFILE_INFO_LIST,
        WLAN_PROFILE_USER,
    },
};

use crate::{errors::WinWifiError, interfaces::WlanInterface};

pub struct WlanInterfaceProfiles<'handle: 'interfaces, 'interfaces> {
    interface: &'interfaces WlanInterface<'handle, 'interfaces>,
    profile_list_ptr: NonNull<WLAN_PROFILE_INFO_LIST>,
    _profile_list_marker: PhantomData<WLAN_PROFILE_INFO_LIST>,
}

impl<'handle: 'interfaces, 'interfaces> WlanInterfaceProfiles<'handle, 'interfaces> {
    pub fn new(
        interface: &'interfaces WlanInterface<'handle, 'interfaces>,
    ) -> Result<WlanInterfaceProfiles<'handle, 'interfaces>, WinWifiError> {
        let mut profile_list_ptr = std::ptr::null_mut();

        WIN32_ERROR(unsafe {
            WlanGetProfileList(
                *interface.handle.as_ptr(),
                interface.guid().as_ptr(),
                None,
                &mut profile_list_ptr,
            )
        })
        .ok()?;

        Ok(WlanInterfaceProfiles {
            interface,
            profile_list_ptr: unsafe { NonNull::new_unchecked(profile_list_ptr) },
            _profile_list_marker: PhantomData,
        })
    }

    #[allow(unused)]
    pub(crate) unsafe fn from_raw(
        interface: &'interfaces WlanInterface<'handle, 'interfaces>,
        profile_list_ptr: *mut WLAN_PROFILE_INFO_LIST,
    ) -> WlanInterfaceProfiles<'handle, 'interfaces> {
        WlanInterfaceProfiles {
            interface,
            profile_list_ptr: NonNull::new_unchecked(profile_list_ptr),
            _profile_list_marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        unsafe { *self.profile_list_ptr.as_ptr() }.dwNumberOfItems as usize
    }

    pub fn iter<'profiles>(
        &'profiles self,
    ) -> WlanInterfaceProfilesIterator<'handle, 'interfaces, 'profiles>
    where
        'interfaces: 'profiles,
    {
        WlanInterfaceProfilesIterator {
            profiles_list: self,
            index: 0,
        }
    }
}

pub struct WlanInterfaceProfilesIterator<'handle: 'interfaces, 'interfaces: 'profiles, 'profiles> {
    profiles_list: &'profiles WlanInterfaceProfiles<'handle, 'interfaces>,
    index: usize,
}

impl<'handle: 'interfaces, 'interfaces: 'profiles, 'profiles> Iterator
    for WlanInterfaceProfilesIterator<'handle, 'interfaces, 'profiles>
{
    type Item = WlanInterfaceProfile<'handle, 'interfaces, 'profiles>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.profiles_list.len() {
            let next_profile_ptr = unsafe {
                std::ptr::addr_of!(
                    (*(*self).profiles_list.profile_list_ptr.as_ptr()).ProfileInfo[0]
                )
                .add(1)
            };

            let profile = WlanInterfaceProfile {
                interface: self.profiles_list.interface,
                profile_ptr: next_profile_ptr,
                _profile_marker: PhantomData,
            };

            self.index += 1;

            return Some(profile);
        }

        None
    }
}

impl<'handle: 'interfaces, 'interfaces: 'profiles, 'profiles> ExactSizeIterator
    for WlanInterfaceProfilesIterator<'_, '_, '_>
{
    fn len(&self) -> usize {
        self.profiles_list.len()
    }
}

pub struct WlanInterfaceProfile<'handle: 'interfaces, 'interfaces: 'profiles, 'profiles> {
    interface: &'profiles WlanInterface<'handle, 'interfaces>,
    profile_ptr: *const WLAN_PROFILE_INFO,
    _profile_marker: PhantomData<&'profiles WLAN_PROFILE_INFO>,
}

impl<'handle: 'interfaces, 'interfaces: 'profiles, 'profiles> WlanInterfaceProfile<'_, '_, '_> {
    pub fn group_policy_profile(&self) -> bool {
        (unsafe { *self.profile_ptr }.dwFlags & WLAN_PROFILE_GROUP_POLICY) != 0
    }

    pub fn user_profile(&self) -> bool {
        (unsafe { *self.profile_ptr }.dwFlags & WLAN_PROFILE_USER) != 0
    }
}
