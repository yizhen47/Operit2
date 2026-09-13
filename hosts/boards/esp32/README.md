# operit-board-esp32

Board profile for ESP32-2432S028 hardware.

This crate owns the board-specific Host API implementations used by
`apps/esp32`. The implemented surfaces are:

- `DeviceIoHost` for the onboard RGB status LEDs
- `RobotFaceHost` for the ILI9341 face display

The crate does not own Operit Core startup, Access pairing, Link packets,
Wi-Fi transport, or the firmware HTTP home page. Those belong to `apps/esp32`.

Display rotation `270` is clockwise from the CircuitPython landscape default
and yields a 240x320 portrait face layout.
