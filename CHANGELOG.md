# Changelog

## [2.3.0] - 2026-01-26

### Added
- **Wireless LAN Scanning**: Added a new feature to automatically discover Android devices on the local network that have ADB wireless debugging enabled (port 5555).
- **VPN/TUN Mode Compatibility**: Optimized scanning logic to work reliably even when VPNs or TUN modes (like sing-box, Clash) are active. Automatically filters out virtual network adapters (VMware, VirtualBox, vEthernet, etc.).
- **TCP Pre-check**: Implemented a fast TCP connection check before calling `adb connect` to prevent UI freezes when connecting to unreachable IP addresses.

### Changed
- **Async Connection**: Refactored the backend connection logic to be asynchronous, ensuring a smooth and responsive user interface during network operations.
- **Code Optimization**: Cleaned up the Rust codebase, removed unused imports, and addressed compiler warnings.
- **Improved UI Feedback**: Added clear error messages and loading states for device scanning and connection processes.

### Fixed
- Fixed an issue where the application would become unresponsive when attempting to connect to an invalid IP address.
- Fixed potential false positives in device scanning by excluding non-physical network ranges.

---

## [Previously]
- Initial release and subsequent patches.
