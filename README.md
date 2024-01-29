> [!WARNING]
> Work in progress

# winwifi
Windows WiFi management inspired by https://devblogs.microsoft.com/scripting/view-passwords-of-wireless-profiles-without-using-netsh-exe/

![GitHub License](https://img.shields.io/github/license/MEhrn00/winwifi)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/MEhrn00/winwifi/workflow.yml)
![Crates.io Version](https://img.shields.io/crates/v/winwifi)
![docs.rs](https://img.shields.io/docsrs/winwifi)


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
