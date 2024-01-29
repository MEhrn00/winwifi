# Changelog

List project changes

## [0.1.0-rc1] - 2024-01-28

### Added

- Extra CI steps for building and testing
- README.md badges

### Changed

- README.md item layout
- Github workflow badge text
- Renamed workflow from workflow.yml to ci.yml
- Docs.rs metadata to fix docs build

### Removed

- Cargo check in clippy CI job

## [0.1.0] - 2024-01-28

### Changed

- Change versioning tag back to 0.1.0 to follow [semver](https://semver.org/#how-should-i-deal-with-revisions-in-the-0yz-initial-development-phase) guidelines

## [0.0.1] - 2024-01-28

### Added

- Colors to the actions CI output
- Github actions CI workflow
- Work in progress message to README.md
- Example wireless profiles XML file
- LICENSE file
- Wireless profile parsing support

### Changed

- Bumped CI actions checkout version to v4
- Combined CI actions into one workflow

### Removed

- wlan DLL linkage in tests to work with CI

[0.1.0-rc1]: https://github.com/MEhrn00/winwifi/compare/v0.1.0...v0.1.0-rc1
[0.1.0]: https://github.com/MEhrn00/winwifi/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/MEhrn00/winwifi/releases/tag/v0.0.1
