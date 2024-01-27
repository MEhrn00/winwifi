use thiserror::Error;

#[derive(Error, Debug)]
pub enum WinWifiError {
    #[error("call to Windows API returned an error code")]
    Win32Error(#[from] windows::core::Error),
}
