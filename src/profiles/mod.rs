use std::{ffi::OsString, marker::PhantomData, os::windows::ffi::OsStringExt, ptr::NonNull};

use windows::Win32::{
    Foundation::WIN32_ERROR,
    NetworkManagement::WiFi::{
        WlanFreeMemory, WlanGetProfileList, WLAN_PROFILE_GROUP_POLICY, WLAN_PROFILE_INFO,
        WLAN_PROFILE_INFO_LIST, WLAN_PROFILE_USER,
    },
};

use crate::{errors::WinWifiError, interfaces::WlanInterface};

mod info;

/// A list of saved wireless profiles for a wireless interface
pub struct WlanInterfaceProfiles<'interfaces, 'handle: 'interfaces> {
    /// The interface associated with this list of profiles
    interface: &'interfaces WlanInterface<'interfaces, 'handle>,

    /// The underlying pointer to the memory of the list of profiles
    profile_list_ptr: NonNull<WLAN_PROFILE_INFO_LIST>,

    /// Marker signifying that this structure owns the memory of the profile_list_ptr
    _marker: PhantomData<WLAN_PROFILE_INFO_LIST>,
}

impl<'interfaces, 'handle: 'interfaces> WlanInterfaceProfiles<'interfaces, 'handle> {
    pub fn new(
        interface: &'interfaces WlanInterface<'interfaces, 'handle>,
    ) -> Result<WlanInterfaceProfiles<'interfaces, 'handle>, WinWifiError> {
        let mut profile_list_ptr = std::ptr::null_mut();

        let wlan_handle = interface.handle;

        WIN32_ERROR(unsafe {
            WlanGetProfileList(
                *wlan_handle.as_ptr(),
                interface.guid().as_ptr(),
                None,
                &mut profile_list_ptr,
            )
        })
        .ok()?;

        Ok(WlanInterfaceProfiles {
            interface,
            profile_list_ptr: unsafe { NonNull::new_unchecked(profile_list_ptr) },
            _marker: PhantomData,
        })
    }

    #[allow(unused)]
    pub(crate) unsafe fn from_raw_parts(
        interface: &'interfaces WlanInterface<'interfaces, 'handle>,
        profile_list_ptr: *mut WLAN_PROFILE_INFO_LIST,
    ) -> WlanInterfaceProfiles<'interfaces, 'handle> {
        WlanInterfaceProfiles {
            interface,
            profile_list_ptr: NonNull::new_unchecked(profile_list_ptr),
            _marker: PhantomData,
        }
    }

    pub const fn len(&self) -> usize {
        let profile_list = unsafe { self.profile_list_ptr.as_ref() };
        profile_list.dwNumberOfItems as usize
    }

    pub const fn is_empty(&self) -> bool {
        let profile_list = unsafe { self.profile_list_ptr.as_ref() };
        profile_list.dwNumberOfItems == 0
    }

    pub fn iter<'profiles>(
        &'profiles self,
    ) -> WlanInterfaceProfilesIterator<'profiles, 'interfaces, 'handle>
    where
        'interfaces: 'profiles,
    {
        WlanInterfaceProfilesIterator {
            interface: &self.interface,
            item_count: self.len(),
            index: 0,
            profiles_list_ptr: self.profile_list_ptr,
            _marker: PhantomData,
        }
    }
}

impl<'interfaces, 'handle: 'interfaces> Drop for WlanInterfaceProfiles<'interfaces, 'handle> {
    fn drop(&mut self) {
        unsafe { WlanFreeMemory(self.profile_list_ptr.as_ptr().cast()) };
    }
}

/// An iterator over the list of wireless interface profiles
pub struct WlanInterfaceProfilesIterator<'profiles, 'interfaces: 'profiles, 'handle: 'interfaces> {
    /// Interface the list of profiles is associated with
    interface: &'interfaces WlanInterface<'interfaces, 'handle>,

    /// Number of items in the profile liset
    item_count: usize,

    /// Current index of the iterator
    index: usize,

    /// Pointer to the underlying list of profiles
    profiles_list_ptr: NonNull<WLAN_PROFILE_INFO_LIST>,

    /// Marker signifying that the pointer does not own the memory to the list of profiles
    _marker: PhantomData<&'profiles WLAN_PROFILE_INFO_LIST>,
}

impl<'profiles, 'interfaces: 'profiles, 'handle: 'interfaces> Iterator
    for WlanInterfaceProfilesIterator<'profiles, 'interfaces, 'handle>
{
    type Item = WlanInterfaceProfile<'profiles, 'interfaces, 'handle>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.item_count {
            let next_profile_ptr = unsafe {
                let profiles_list = self.profiles_list_ptr.as_mut();
                std::ptr::addr_of_mut!(profiles_list.ProfileInfo[0]).add(self.index)
            };

            let profile = WlanInterfaceProfile {
                interface: self.interface,
                profile_ptr: unsafe { NonNull::new_unchecked(next_profile_ptr) },
                _marker: PhantomData,
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
        self.item_count
    }
}

/// A saved profile for a wireless interface
pub struct WlanInterfaceProfile<'profiles, 'interfaces: 'profiles, 'handle: 'interfaces> {
    #[allow(unused)]
    /// The underlying interface associated with this profile
    interface: &'profiles WlanInterface<'interfaces, 'handle>,

    /// A pointer to the underlying profile memory
    profile_ptr: NonNull<WLAN_PROFILE_INFO>,

    /// Marker signifying that the profile_ptr does not own the memory it points to
    _marker: PhantomData<&'profiles WLAN_PROFILE_INFO>,
}

impl<'profiles, 'interfaces: 'profiles, 'handle: 'interfaces> WlanInterfaceProfile<'_, '_, '_> {
    pub fn group_policy_profile(&self) -> bool {
        let profile = unsafe { self.profile_ptr.as_ref() };
        profile.dwFlags & WLAN_PROFILE_GROUP_POLICY != 0
    }

    pub fn user_profile(&self) -> bool {
        let profile = unsafe { self.profile_ptr.as_ref() };
        profile.dwFlags & WLAN_PROFILE_USER != 0
    }

    pub fn name(&self) -> Option<OsString> {
        let profile = unsafe { self.profile_ptr.as_ref() };
        let null_index = profile.strProfileName.iter().position(|v| v == &0)?;
        Some(OsString::from_wide(&profile.strProfileName[..null_index]))
    }
}
