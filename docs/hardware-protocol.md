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
Byte 1: Offset High (upper 8 bits of LED start position)
Byte 2: Offset Low (lower 8 bits of LED start position)
Byte 3+: LED Color Data (variable length)
```

## LED Color Data

### RGB LEDs (3 bytes per LED)

```text
[R][G][B][R][G][B][R][G][B]...
```

### RGBW LEDs (4 bytes per LED)

```text
[R][G][B][W][R][G][B][W][R][G][B][W]...
```

All values are 0-255.

## Color Calibration

Colors are calibrated before transmission:

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

3 RGB LEDs starting at position 0: Red, Green, Blue

```text
02 00 00 FF 00 00 00 FF 00 00 00 FF
│  │  │  └─────────────────────────── 9 bytes color data
│  │  └─ Offset Low (0)
│  └─ Offset High (0)
└─ Header (0x02)
```

### RGBW Example

2 RGBW LEDs starting at position 10: White, Warm White

```text
02 00 0A FF FF FF FF FF C8 96 C8
│  │  │  └─────────────────────── 8 bytes color data
│  │  └─ Offset Low (10)
│  └─ Offset High (0)
└─ Header (0x02)
```

## Implementation Notes

- **Byte Order**: Big-endian for multi-byte values (offset field)
- **Delivery**: Fire-and-forget UDP (no acknowledgment required)
- **Hardware Role**: Simple UDP-to-WS2812 bridge, no data processing
- **LED Type Logic**: Handled entirely on desktop side, not hardware
- **Mixed Types**: Same display can have both RGB and RGBW strips
- **Data Flow**: Desktop → UDP → Hardware → WS2812 (direct forward)

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

    uint16_t offset = (data[1] << 8) | data[2];
    uint8_t* color_data = &data[3];
    size_t color_len = len - 3;

    // Direct forward to WS2812 - no RGB/RGBW distinction needed
    ws2812_update(offset, color_data, color_len);
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

### Key Implementation Notes

- **Ping Response**: Must respond to ping (0x01) within 1 second
- **LED Data**: Direct forward to WS2812, no processing required
- **Control Commands**: Optional feature for hardware with input capabilities
- **mDNS Registration**: Essential for automatic device discovery
- **UDP Server**: Must handle concurrent connections from multiple desktops

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
- Verify offset calculations and LED strip configuration

**Wrong Colors**:

- Check color calibration settings on desktop
- Verify RGB/RGBW data format matches LED strip type
- Monitor color data in packets (bytes 3+)
- Check WS2812 color order (GRB vs RGB)

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
