#[macro_export]
macro_rules! create_test_handle {
    () => {
        $crate::handle::WlanHandle::new_invalid()
    };
}

#[macro_export]
macro_rules! create_test_interfaces {
    ($handle:ident, $ifaces:expr) => {
        ManuallyDrop::new(unsafe { $crate::WlanInterfaces::from_raw_parts($handle, $ifaces) })
    };
}
