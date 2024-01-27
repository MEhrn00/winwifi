use std::marker::PhantomData;

use windows::core::GUID;

/// Reference to a Windows GUID with added functionality
#[derive(Debug)]
#[repr(transparent)]
pub struct GuidRef<'a> {
    guid: *const GUID,
    _marker: PhantomData<&'a GUID>,
}

impl<'a> GuidRef<'a> {
    pub fn from_ptr(guid: *const GUID) -> GuidRef<'a> {
        GuidRef {
            guid,
            _marker: PhantomData,
        }
    }
}

impl<'a> std::cmp::PartialEq<GUID> for GuidRef<'a> {
    fn eq(&self, other: &GUID) -> bool {
        return unsafe { &(*self.guid) } == other;
    }
}

impl<'a> From<&'a GUID> for GuidRef<'a> {
    fn from(value: &'a GUID) -> Self {
        Self::from_ptr(value)
    }
}

impl<'a> ToString for GuidRef<'_> {
    fn to_string(&self) -> String {
        unsafe {
            let mut clock_seq: [u8; 2] = Default::default();
            let mut node: [u8; 8] = Default::default();

            // Copy out the clock_seq and node from the guid without needing to allocate it
            clock_seq.clone_from_slice(&(*self.guid).data4[..2]);
            (&mut node[2..]).clone_from_slice(&(*self.guid).data4[2..]);

            format!(
                "{:x}-{:x}-{:x}-{:x}-{:x}",
                (*self.guid).data1,
                (*self.guid).data2,
                (*self.guid).data3,
                u16::from_be_bytes(clock_seq),
                u64::from_be_bytes(node),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn to_string() {
        const TEST_GUID_STRING: &'static str = "12345678-1234-1234-1122-334455667788";

        let guid = GUID::from_values(
            0x12345678,
            0x1234,
            0x1234,
            [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        );

        let guid_ref = GuidRef::from(&guid);
        let guid_string = guid_ref.to_string();

        assert_eq!(guid_string.as_str(), TEST_GUID_STRING);
    }
}
