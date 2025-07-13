use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestEffectType {
    FlowingRainbow,
    GroupCounting,
    SingleScan,
    Breathing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEffectConfig {
    pub effect_type: TestEffectType,
    pub led_count: u32,
    pub led_type: LedType,
    pub speed: f64,  // Speed multiplier
    pub offset: u32, // LED offset
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedType {
    WS2812B,
    SK6812,
}

pub struct LedTestEffects;

impl LedTestEffects {
    /// Check if LED type supports white channel (RGBW)
    fn is_rgbw_type(led_type: &LedType) -> bool {
        matches!(led_type, LedType::SK6812)
    }
    /// Generate LED colors for a specific test effect at a given time
    pub fn generate_colors(config: &TestEffectConfig, time_ms: u64) -> Vec<u8> {
        let time_seconds = time_ms as f64 / 1000.0;

        match config.effect_type {
            TestEffectType::FlowingRainbow => Self::flowing_rainbow(
                config.led_count,
                config.led_type.clone(),
                time_seconds,
                config.speed,
            ),
            TestEffectType::GroupCounting => {
                Self::group_counting(config.led_count, config.led_type.clone())
            }
            TestEffectType::SingleScan => Self::single_scan(
                config.led_count,
                config.led_type.clone(),
                time_seconds,
                config.speed,
            ),
            TestEffectType::Breathing => Self::breathing(
                config.led_count,
                config.led_type.clone(),
                time_seconds,
                config.speed,
            ),
        }
    }

    /// Calculate byte offset for 0x02 packet based on LED offset and LED type
    pub fn calculate_byte_offset(config: &TestEffectConfig) -> u16 {
        let bytes_per_led = if Self::is_rgbw_type(&config.led_type) {
            4
        } else {
            3
        };
        (config.offset * bytes_per_led) as u16
    }

    /// Flowing rainbow effect - smooth rainbow colors flowing along the strip
    fn flowing_rainbow(led_count: u32, led_type: LedType, time: f64, speed: f64) -> Vec<u8> {
        let mut buffer = Vec::new();
        let time_offset = (time * speed * 60.0) % 360.0; // 60 degrees per second at speed 1.0

        for i in 0..led_count {
            // Create longer wavelength for smoother color transitions
            let hue = ((i as f64 * 720.0 / led_count as f64) + time_offset) % 360.0;
            let rgb = Self::hsv_to_rgb(hue, 1.0, 1.0);

            buffer.push(rgb.0);
            buffer.push(rgb.1);
            buffer.push(rgb.2);

            if Self::is_rgbw_type(&led_type) {
                buffer.push(0); // White channel
            }
        }

        buffer
    }

    /// Group counting effect - every 10 LEDs have different colors
    fn group_counting(led_count: u32, led_type: LedType) -> Vec<u8> {
        let mut buffer = Vec::new();

        let group_colors = [
            (255, 0, 0),     // Red (1-10)
            (0, 255, 0),     // Green (11-20)
            (0, 0, 255),     // Blue (21-30)
            (255, 255, 0),   // Yellow (31-40)
            (255, 0, 255),   // Magenta (41-50)
            (0, 255, 255),   // Cyan (51-60)
            (255, 128, 0),   // Orange (61-70)
            (128, 255, 0),   // Lime (71-80)
            (255, 255, 255), // White (81-90)
            (128, 128, 128), // Gray (91-100)
        ];

        for i in 0..led_count {
            let group_index = (i / 10) % group_colors.len() as u32;
            let color = group_colors[group_index as usize];

            buffer.push(color.0);
            buffer.push(color.1);
            buffer.push(color.2);

            if Self::is_rgbw_type(&led_type) {
                buffer.push(0); // White channel
            }
        }

        buffer
    }

    /// Single LED scan effect - one LED moves along the strip
    fn single_scan(led_count: u32, led_type: LedType, time: f64, speed: f64) -> Vec<u8> {
        let mut buffer = Vec::new();
        let scan_period = 2.0 / speed; // 2 seconds per full scan at speed 1.0
        let active_index = ((time / scan_period * led_count as f64) as u32) % led_count;

        for i in 0..led_count {
            if i == active_index {
                // Bright white LED
                buffer.push(255);
                buffer.push(255);
                buffer.push(255);

                if Self::is_rgbw_type(&led_type) {
                    buffer.push(255); // White channel
                }
            } else {
                // Off
                buffer.push(0);
                buffer.push(0);
                buffer.push(0);

                if Self::is_rgbw_type(&led_type) {
                    buffer.push(0); // White channel
                }
            }
        }

        buffer
    }

    /// Breathing effect - entire strip breathes with white light
    fn breathing(led_count: u32, led_type: LedType, time: f64, speed: f64) -> Vec<u8> {
        let mut buffer = Vec::new();
        let breathing_period = 4.0 / speed; // 4 seconds per breath at speed 1.0
        let brightness = ((time / breathing_period * 2.0 * PI).sin() * 0.5 + 0.5) * 255.0;
        let brightness = brightness as u8;

        for _i in 0..led_count {
            buffer.push(brightness);
            buffer.push(brightness);
            buffer.push(brightness);

            if Self::is_rgbw_type(&led_type) {
                buffer.push(brightness); // White channel
            }
        }

        buffer
    }

    /// Convert HSV to RGB
    /// H: 0-360, S: 0-1, V: 0-1
    /// Returns: (R, G, B) where each component is 0-255
    fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0).round() as u8;
        let g = ((g_prime + m) * 255.0).round() as u8;
        let b = ((b_prime + m) * 255.0).round() as u8;

        (r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsv_to_rgb() {
        // Test red
        let (r, g, b) = LedTestEffects::hsv_to_rgb(0.0, 1.0, 1.0);
        assert_eq!((r, g, b), (255, 0, 0));

        // Test green
        let (r, g, b) = LedTestEffects::hsv_to_rgb(120.0, 1.0, 1.0);
        assert_eq!((r, g, b), (0, 255, 0));

        // Test blue
        let (r, g, b) = LedTestEffects::hsv_to_rgb(240.0, 1.0, 1.0);
        assert_eq!((r, g, b), (0, 0, 255));
    }

    #[test]
    fn test_flowing_rainbow() {
        let config = TestEffectConfig {
            effect_type: TestEffectType::FlowingRainbow,
            led_count: 10,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 0,
        };

        let colors = LedTestEffects::generate_colors(&config, 0);
        assert_eq!(colors.len(), 30); // 10 LEDs * 3 bytes each
    }

    #[test]
    fn test_calculate_byte_offset() {
        // Test WS2812B (3 bytes per LED)
        let config_ws2812b = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 60,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 10, // 10 LEDs offset
        };

        let byte_offset_ws2812b = LedTestEffects::calculate_byte_offset(&config_ws2812b);
        assert_eq!(byte_offset_ws2812b, 30); // 10 LEDs * 3 bytes = 30 bytes

        // Test SK6812 (4 bytes per LED)
        let config_sk6812 = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 60,
            led_type: LedType::SK6812,
            speed: 1.0,
            offset: 10, // 10 LEDs offset
        };

        let byte_offset_sk6812 = LedTestEffects::calculate_byte_offset(&config_sk6812);
        assert_eq!(byte_offset_sk6812, 40); // 10 LEDs * 4 bytes = 40 bytes

        // Test zero offset
        let config_zero_offset = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 60,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 0,
        };

        let byte_offset_zero = LedTestEffects::calculate_byte_offset(&config_zero_offset);
        assert_eq!(byte_offset_zero, 0); // 0 LEDs * 3 bytes = 0 bytes
    }

    #[test]
    fn test_group_counting() {
        let config = TestEffectConfig {
            effect_type: TestEffectType::GroupCounting,
            led_count: 20,
            led_type: LedType::WS2812B,
            speed: 1.0,
            offset: 0,
        };

        let colors = LedTestEffects::generate_colors(&config, 0);
        assert_eq!(colors.len(), 60); // 20 LEDs * 3 bytes each

        // First 10 should be red
        assert_eq!(colors[0], 255); // R
        assert_eq!(colors[1], 0); // G
        assert_eq!(colors[2], 0); // B

        // Next 10 should be green
        assert_eq!(colors[30], 0); // R
        assert_eq!(colors[31], 255); // G
        assert_eq!(colors[32], 0); // B
    }
}
