# winwifi
Windows WiFi management inspired by https://devblogs.microsoft.com/scripting/view-passwords-of-wireless-profiles-without-using-netsh-exe/

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
