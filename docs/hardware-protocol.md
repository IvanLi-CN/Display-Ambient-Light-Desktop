# LED Hardware Communication Protocol

## Overview

UDP-based bidirectional protocol for communication between desktop application and ambient light hardware boards. The protocol supports LED color data transmission, device health monitoring, and remote control capabilities.

## Connection

- **Protocol**: UDP
- **Port**: 23042
- **Discovery**: mDNS (`_ambient_light._udp.local.`)
- **Example Board**: `192.168.31.206:23042`

## mDNS Service Discovery

### Service Registration (Hardware Side)

Hardware boards must register the following mDNS service:

- **Service Type**: `_ambient_light._udp.local.`
- **Port**: 23042
- **TXT Records**: Optional, can include device information

### Service Discovery (Desktop Side)

Desktop application continuously browses for `_ambient_light._udp.local.` services and automatically connects to discovered devices.

## Protocol Messages

The protocol uses different message headers to distinguish message types:

| Header | Direction | Purpose | Format |
|--------|-----------|---------|---------|
| 0x01 | Desktop → Hardware | Ping (Health Check) | `[0x01]` |
| 0x01 | Hardware → Desktop | Pong (Health Response) | `[0x01]` |
| 0x02 | Desktop → Hardware | LED Color Data | `[0x02][Offset_H][Offset_L][Color_Data...]` |
| 0x03 | Hardware → Desktop | Display Brightness Control | `[0x03][Display_Index][Brightness]` |
| 0x04 | Hardware → Desktop | Volume Control | `[0x04][Volume_Percent]` |

## Health Check Protocol (Ping/Pong)

### Desktop → Hardware (Ping)

```text
Byte 0: Header (0x01)
```

### Hardware → Desktop (Pong)

```text
Byte 0: Header (0x01)
```

**Behavior:**

- Desktop sends ping every 1 second to each connected device
- Hardware must respond with pong within 1 second
- Timeout or incorrect response triggers reconnection logic
- After 10 failed attempts, device is marked as disconnected

## LED Color Data Protocol

### Packet Format

```text
Byte 0: Header (0x02)
Byte 1: Offset High (upper 8 bits of data byte offset)
Byte 2: Offset Low (lower 8 bits of data byte offset)
Byte 3+: LED Color Data (variable length)
```

## LED Color Data

The desktop application handles LED strip type selection and color order conversion. Hardware receives final, ready-to-use data that can be directly forwarded to the LED strip without any processing.

### RGB LEDs (3 bytes per LED)

```text
[C1][C2][C3][C1][C2][C3][C1][C2][C3]...
```

### RGBW LEDs (4 bytes per LED)

```text
[C1][C2][C3][C4][C1][C2][C3][C4][C1][C2][C3][C4]...
```

Where C1, C2, C3, C4 represent color channels in the exact order required by the target LED chip (e.g., G-R-B for WS2812B, G-R-B-W for SK6812-RGBW). All values are 0-255.

### Offset Calculation

The offset field specifies the starting byte position in the LED data buffer:

- **16-bit value**: Combines Offset High and Offset Low bytes (big-endian)
- **Range**: 0-65535 bytes supported
- **Purpose**: Allows partial updates of LED strip data at any byte position

**Example Calculations:**

- Byte position 0: `Offset High = 0x00, Offset Low = 0x00`
- Byte position 30: `Offset High = 0x00, Offset Low = 0x1E` (10 RGB LEDs × 3 bytes)
- Byte position 256: `Offset High = 0x01, Offset Low = 0x00`
- Byte position 1000: `Offset High = 0x03, Offset Low = 0xE8`

**LED Position to Byte Offset Conversion:**

- **RGB LEDs**: `byte_offset = led_position × 3`
- **RGBW LEDs**: `byte_offset = led_position × 4`

## LED Chip Specifications

### WS2812B (RGB)

- **Type**: RGB
- **Data Format**: 3 bytes per LED
- **Color Order**: G-R-B (Green, Red, Blue)
- **Voltage**: 5V
- **Protocol**: Single-wire serial
- **Timing**: 800kHz data rate

### SK6812 (RGB)

- **Type**: RGB
- **Data Format**: 3 bytes per LED
- **Color Order**: G-R-B (Green, Red, Blue)
- **Voltage**: 5V
- **Protocol**: Single-wire serial
- **Timing**: 800kHz data rate
- **Features**: Improved PWM linearity compared to WS2812B

### SK6812-RGBW

- **Type**: RGBW
- **Data Format**: 4 bytes per LED
- **Color Order**: G-R-B-W (Green, Red, Blue, White)
- **Voltage**: 5V
- **Protocol**: Single-wire serial
- **Timing**: 800kHz data rate
- **Features**: Dedicated white channel for better color mixing and higher brightness

## Desktop Application Responsibilities

The desktop application is responsible for:

1. **LED Strip Type Selection**: User selects the LED strip type (WS2812B, SK6812, SK6812-RGBW, etc.)
2. **Color Order Conversion**: Converts RGB(W) values to the correct color order for the target LED chip
3. **Color Calibration**: Applies calibration values before transmission
4. **Data Formatting**: Sends final, ready-to-use data that hardware can directly forward to LED strips

### Color Calibration (Desktop Side)

Colors are calibrated by the desktop application before transmission:

**RGB:**

```rust
calibrated_r = (original_r * calibration_r) / 255
calibrated_g = (original_g * calibration_g) / 255
calibrated_b = (original_b * calibration_b) / 255
```

**RGBW:**

```rust
calibrated_r = (original_r * calibration_r) / 255
calibrated_g = (original_g * calibration_g) / 255
calibrated_b = (original_b * calibration_b) / 255
calibrated_w = calibration_w  // Direct value
```

## Hardware Control Protocol (Hardware → Desktop)

### Display Brightness Control

Hardware can send display brightness adjustment commands to the desktop:

```text
Byte 0: Header (0x03)
Byte 1: Display Index (0-based display number)
Byte 2: Brightness (0-255, where 255 = 100% brightness)
```

**Example:** Set display 0 to 50% brightness

```text
03 00 80
│  │  └─ Brightness (128 = ~50%)
│  └─ Display Index (0)
└─ Header (0x03)
```

### Volume Control

Hardware can send system volume adjustment commands to the desktop:

```text
Byte 0: Header (0x04)
Byte 1: Volume Percent (0-100)
```

**Example:** Set system volume to 75%

```text
04 4B
│  └─ Volume (75%)
└─ Header (0x04)
```

## Connection State Management

### Connection States

- **Unknown**: Initial state when device is first discovered
- **Connecting**: Device is being tested, includes retry count (1-10)
- **Connected**: Device is responding to ping requests normally
- **Disconnected**: Device failed to respond after 10 retry attempts

### State Transitions

```text
Unknown → Connecting(1) → Connected
    ↓           ↓             ↓
    ↓      Connecting(2-10)   ↓
    ↓           ↓             ↓
    └─→ Disconnected ←────────┘
```

### Retry Logic

1. **Initial Connection**: When device discovered via mDNS
2. **Health Check Failure**: If ping timeout or wrong response
3. **Retry Attempts**: Up to 10 attempts with 1-second intervals
4. **Disconnection**: After 10 failed attempts, mark as disconnected
5. **Recovery**: Disconnected devices continue to receive ping attempts

## Packet Examples

### RGB Example

3 RGB LEDs starting at byte offset 0: Red, Green, Blue

```text
02 00 00 FF 00 00 00 FF 00 00 00 FF
│  │  │  └─────────────────────────── 9 bytes color data
│  │  └─ Offset Low (0 bytes)
│  └─ Offset High (0)
└─ Header (0x02)
```

### RGBW Example

2 RGBW LEDs starting at byte offset 40 (LED position 10): White, Warm White

```text
02 00 28 FF FF FF FF FF C8 96 C8
│  │  │  └─────────────────────── 8 bytes color data
│  │  └─ Offset Low (40 bytes = 0x28)
│  └─ Offset High (0)
└─ Header (0x02)
```

## Implementation Notes

- **Byte Order**: Big-endian for multi-byte values (offset field)
- **Delivery**: Fire-and-forget UDP (no acknowledgment required)
- **Hardware Role**: Simple UDP-to-LED bridge, no data processing required
- **LED Type Logic**: Handled entirely on desktop side, not hardware
- **Mixed Types**: Desktop can handle multiple LED strip types per display
- **Data Flow**: Desktop → UDP → Hardware → LED Strip (direct forward)
- **Color Order**: Desktop handles all color order conversion, hardware receives final data
- **LED Compatibility**: Hardware acts as transparent bridge for any LED chip type

## Hardware Implementation

The hardware board handles multiple protocol functions: UDP-to-WS2812 bridge for LED data, health monitoring, and optional control input capabilities.

### Required Functions

1. **mDNS Service Registration**: Advertise `_ambient_light._udp.local.` service
2. **UDP Server**: Listen on port 23042 for incoming packets
3. **Packet Processing**: Handle different message types based on header
4. **Health Monitoring**: Respond to ping requests with pong
5. **LED Control**: Forward color data to WS2812 strips
6. **Optional Control**: Send brightness/volume commands to desktop

### Packet Processing Logic

```c
void process_packet(uint8_t* data, size_t len) {
    if (len < 1) return;

    switch (data[0]) {
        case 0x01: // Ping request
            handle_ping(data, len);
            break;

        case 0x02: // LED color data
            handle_led_data(data, len);
            break;

        default:
            // Unknown packet type, ignore
            break;
    }
}

void handle_ping(uint8_t* data, size_t len) {
    if (len != 1) return;

    // Respond with pong
    uint8_t pong = 0x01;
    udp_send_response(&pong, 1);
}

void handle_led_data(uint8_t* data, size_t len) {
    if (len < 3) return;

    uint16_t byte_offset = (data[1] << 8) | data[2];
    uint8_t* color_data = &data[3];
    size_t color_len = len - 3;

    // Direct forward to LED strip - data is already in correct format
    // byte_offset specifies the starting byte position in LED data buffer
    // No processing required, desktop sends ready-to-use data
    led_strip_update(byte_offset, color_data, color_len);
}
```

### Optional Control Features

Hardware can optionally send control commands to desktop:

```c
// Send display brightness control
void send_brightness_control(uint8_t display_index, uint8_t brightness) {
    uint8_t packet[3] = {0x03, display_index, brightness};
    udp_send_to_desktop(packet, 3);
}

// Send volume control
void send_volume_control(uint8_t volume_percent) {
    uint8_t packet[2] = {0x04, volume_percent};
    udp_send_to_desktop(packet, 2);
}
```

### LED Strip Driver Implementation

Hardware simply forwards data directly to LED strips without any processing:

```c
// Example LED strip update function - direct forward
void led_strip_update(uint16_t offset, uint8_t* data, size_t len) {
    // Desktop sends data in correct format for target LED chip
    // Hardware acts as transparent bridge - no processing required

    // Direct forward to LED strip with proper timing
    led_strip_send(offset, data, len);
}
```

### Key Implementation Notes

- **Ping Response**: Must respond to ping (0x01) within 1 second
- **LED Data**: Direct forward to LED strip, no processing required
- **Data Format**: Desktop sends final data in correct color order for target LED chip
- **Control Commands**: Optional feature for hardware with input capabilities
- **mDNS Registration**: Essential for automatic device discovery
- **UDP Server**: Must handle concurrent connections from multiple desktops
- **LED Chip Support**: Hardware acts as transparent bridge, desktop handles chip-specific formatting

## Troubleshooting

### Device Discovery Issues

**Device Not Found**:

- Verify mDNS service registration on hardware
- Check service type: `_ambient_light._udp.local.`
- Ensure port 23042 is accessible
- Verify network connectivity between desktop and hardware

**Device Shows as Disconnected**:

- Check ping/pong response implementation
- Verify hardware responds to 0x01 packets within 1 second
- Monitor network latency and packet loss
- Check UDP server implementation on hardware

### LED Control Issues

**No LED Updates**:

- Verify hardware processes 0x02 packets correctly
- Check WS2812 wiring and power supply
- Monitor packet reception on hardware side
- Verify byte offset calculations and LED strip configuration

**Wrong Colors**:

- Check LED strip type selection in desktop application
- Verify desktop application is configured for correct LED chip type
- Check color calibration settings on desktop
- Monitor color data in packets (bytes 3+)
- Ensure desktop application is sending data in correct color order for target LED chip
- Hardware should forward data directly without any color processing

**Flickering or Lag**:

- Monitor packet rate and network congestion
- Check power supply stability for LED strips
- Verify WS2812 data signal integrity
- Consider reducing update frequency

### Control Protocol Issues

**Brightness/Volume Control Not Working**:

- Verify hardware sends correct packet format (0x03/0x04)
- Check desktop receives and processes control packets
- Monitor packet transmission from hardware
- Verify display index and value ranges

### Connection State Issues

**Frequent Disconnections**:

- Check network stability and latency
- Verify ping response timing (< 1 second)
- Monitor retry logic and connection state transitions
- Check for UDP packet loss

**Stuck in Connecting State**:

- Verify ping/pong packet format
- Check hardware UDP server implementation
- Monitor ping response timing
- Verify network firewall settings

### Network Debugging

**Packet Monitoring**:

```bash
# Monitor UDP traffic on port 23042
tcpdump -i any -X port 23042

# Check mDNS service discovery
dns-sd -B _ambient_light._udp.local.
```

**Hardware Debug Output**:

- Log received packet headers and lengths
- Monitor ping/pong timing
- Track LED data processing
- Log mDNS service registration status

## Protocol Version

- **Current**: 1.0
- **Headers**: 0x01 (Ping/Pong), 0x02 (LED Data), 0x03 (Brightness), 0x04 (Volume)
- **Future**: Additional headers for new features, backward compatibility maintained
