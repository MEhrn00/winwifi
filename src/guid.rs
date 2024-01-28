use std::fmt::Display;

use windows::core::GUID;

/// Reference to a Windows GUID with added functionality
#[derive(Debug)]
#[repr(transparent)]
pub struct GuidRef<'a>(&'a GUID);

impl<'a> GuidRef<'a> {
    /// # Safety
    ///
    /// Pointer validation is not done. Caller has to ensure that a valid pointer to a GUID is passed in
    pub unsafe fn from_ptr(guid: *const GUID) -> GuidRef<'a> {
        GuidRef(&*guid)
    }

    pub(crate) fn as_ptr(&self) -> *const GUID {
        self.0
    }
}

impl<'a> std::cmp::PartialEq<GUID> for GuidRef<'a> {
    fn eq(&self, other: &GUID) -> bool {
        self.0 == other
    }
}

impl<'a> From<&'a GUID> for GuidRef<'a> {
    fn from(value: &'a GUID) -> Self {
        GuidRef(value)
    }
}

impl Display for GuidRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut clock_seq: [u8; 2] = Default::default();
        let mut node: [u8; 8] = Default::default();

        // Copy out the clock_seq and node from the guid without unnecessarily allocating a copy of it
        clock_seq.clone_from_slice(&self.0.data4[..2]);
        node[2..].clone_from_slice(&self.0.data4[2..]);

        write!(
            f,
            "{:x}-{:x}-{:x}-{:x}-{:x}",
            self.0.data1,
            self.0.data2,
            self.0.data3,
            u16::from_be_bytes(clock_seq),
            u64::from_be_bytes(node),
        )
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
