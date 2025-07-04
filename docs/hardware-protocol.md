# LED Hardware Communication Protocol

## Overview

UDP-based protocol for sending LED color data from desktop application to ambient light hardware boards. The hardware acts as a simple UDP-to-WS2812 bridge, directly forwarding color data without any processing or LED type distinction.

## Connection

- **Protocol**: UDP
- **Port**: 23042
- **Discovery**: mDNS (`_ambient_light._udp.local.`)
- **Example Board**: `192.168.31.206:23042`

## Packet Format

```
Byte 0: Header (0x02)
Byte 1: Offset High (upper 8 bits of LED start position)
Byte 2: Offset Low (lower 8 bits of LED start position)  
Byte 3+: LED Color Data (variable length)
```

## LED Color Data

### RGB LEDs (3 bytes per LED)

```
[R][G][B][R][G][B][R][G][B]...
```

### RGBW LEDs (4 bytes per LED)

```
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

## Packet Examples

### RGB Example

3 RGB LEDs starting at position 0: Red, Green, Blue

```
02 00 00 FF 00 00 00 FF 00 00 00 FF
│  │  │  └─────────────────────────── 9 bytes color data
│  │  └─ Offset Low (0)
│  └─ Offset High (0)
└─ Header (0x02)
```

### RGBW Example  

2 RGBW LEDs starting at position 10: White, Warm White

```
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

The hardware board acts as a simple UDP-to-WS2812 bridge, directly forwarding color data to the LED strips without any processing or type distinction.

### Packet Processing

1. **Validation**: Check minimum 3 bytes and header (0x02)
2. **Extract Offset**: Parse 16-bit LED start position
3. **Forward Data**: Send color data directly to WS2812 controller
4. **No Type Logic**: Hardware doesn't distinguish RGB/RGBW - just forwards bytes

### Example C Code

```c
void process_packet(uint8_t* data, size_t len) {
    if (len < 3 || data[0] != 0x02) return;

    uint16_t offset = (data[1] << 8) | data[2];
    uint8_t* color_data = &data[3];
    size_t color_len = len - 3;

    // Direct forward to WS2812 - no RGB/RGBW distinction needed
    ws2812_update(offset, color_data, color_len);
}
```

### Key Simplifications

- **No LED Type Detection**: Hardware doesn't need to know RGB vs RGBW
- **Direct Data Forward**: Color bytes sent as-is to WS2812 controller
- **Desktop Handles Logic**: All RGB/RGBW processing done on desktop side
- **Simple Bridge**: Hardware is just a UDP-to-WS2812 data bridge

## Troubleshooting

**No Updates**: Check network connectivity, mDNS discovery, port 23042
**Wrong Colors**: Verify calibration settings on desktop application
**Flickering**: Monitor packet rate, network congestion, power supply
**Partial Updates**: Check strip configuration, offset calculations
**Hardware Issues**: Verify WS2812 wiring, power supply, data signal integrity

## Protocol Version

- **Current**: 1.0
- **Header**: 0x02
- **Future**: Different headers for backward compatibility
