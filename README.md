# winwifi

[![GitHub License](https://img.shields.io/github/license/MEhrn00/winwifi)](https://github.com/MEhrn00/winwifi/blob/main/LICENSE)
[![CI Status](https://github.com/MEhrn00/winwifi/workflows/CI/badge.svg)](https://github.com/MEhrn00/winwifi/actions?workflow=CI)
[![Crates.io](https://img.shields.io/crates/v/winwifi)](https://crates.io/crates/winwifi)
[![Docs.rs](https://img.shields.io/docsrs/winwifi)](https://docs.rs/crate/winwifi/latest)

Windows WiFi management inspired by https://devblogs.microsoft.com/scripting/view-passwords-of-wireless-profiles-without-using-netsh-exe/

> [!WARNING]
> Work in progress

## Usage
List currently saved WiFi profiles
```powershell
winwifi profile list
```

Get information about a WiFi profile
```powershell
winwifi profile get --name <profile name>
```

Remove a WiFi profile
```powershell
winwifi profile remove --name <profile name>
```

Scan for available WiFi networks
```powershell
winwifi network scan
```

List available WiFi networks
```powershell
winwifi network list
```
