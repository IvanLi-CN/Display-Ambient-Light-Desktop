use std::fmt::Formatter;
use std::sync::Arc;
use std::{fmt::Debug, iter};

use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;

use crate::{ambient_light::LedStripConfig, led_color::LedColor};

#[derive(Clone)]
pub struct Screenshot {
    pub display_id: u32,
    pub height: u32,
    pub width: u32,
    pub bytes_per_row: usize,
    pub bytes: Arc<RwLock<Arc<Vec<u8>>>>,
    pub scale_factor: f32,
    pub bound_scale_factor: f32,
}

impl Debug for Screenshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Screenshot")
            .field("display_id", &self.display_id)
            .field("height", &self.height)
            .field("width", &self.width)
            .field("bytes_per_row", &self.bytes_per_row)
            .field("scale_factor", &self.scale_factor)
            .field("bound_scale_factor", &self.bound_scale_factor)
            .finish()
    }
}

static SINGLE_AXIS_POINTS: usize = 5;

impl Screenshot {
    pub fn new(
        display_id: u32,
        height: u32,
        width: u32,
        bytes_per_row: usize,
        bytes: Arc<Vec<u8>>,
        scale_factor: f32,
        bound_scale_factor: f32,
    ) -> Self {
        Self {
            display_id,
            height,
            width,
            bytes_per_row,
            bytes: Arc::new(RwLock::new(bytes)),
            scale_factor,
            bound_scale_factor,
        }
    }

    pub fn get_sample_points(&self, config: &LedStripConfig) -> Vec<LedSamplePoints> {
        let height = self.height as usize;
        let width = self.width as usize;
        // let height = CGDisplay::new(self.display_id).bounds().size.height as usize;
        // let width = CGDisplay::new(self.display_id).bounds().size.width as usize;

        match config.border {
            crate::ambient_light::Border::Top => {
                Self::get_one_edge_sample_points(height / 18, width, config.len, SINGLE_AXIS_POINTS)
            }
            crate::ambient_light::Border::Bottom => {
                let points = Self::get_one_edge_sample_points(
                    height / 18,
                    width,
                    config.len,
                    SINGLE_AXIS_POINTS,
                );
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (x, height - y)).collect()
                    })
                    .collect()
            }
            crate::ambient_light::Border::Left => {
                let points = Self::get_one_edge_sample_points(
                    width / 32,
                    height,
                    config.len,
                    SINGLE_AXIS_POINTS,
                );
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (y, x)).collect()
                    })
                    .collect()
            }
            crate::ambient_light::Border::Right => {
                let points = Self::get_one_edge_sample_points(
                    width / 32,
                    height,
                    config.len,
                    SINGLE_AXIS_POINTS,
                );
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (width - y, x)).collect()
                    })
                    .collect()
            }
        }
    }

    fn get_one_edge_sample_points(
        width: usize,
        length: usize,
        leds: usize,
        single_axis_points: usize,
    ) -> Vec<LedSamplePoints> {
        if leds == 0 {
            return vec![];
        }

        let cell_size_x = length as f64 / single_axis_points as f64 / leds as f64;
        let cell_size_y = width / single_axis_points;

        let point_start_y = cell_size_y / 2;
        let point_start_x = cell_size_x / 2.0;
        let point_y_list: Vec<usize> = (point_start_y..width).step_by(cell_size_y).collect();
        let point_x_list: Vec<usize> = iter::successors(Some(point_start_x), |i| {
            let next = i + cell_size_x;
            (next < (length as f64)).then_some(next)
        })
        .map(|i| i as usize)
        .collect();

        let points: Vec<Point> = point_x_list
            .iter()
            .map(|&x| point_y_list.iter().map(move |&y| (x, y)))
            .flatten()
            .collect();

        points
            .chunks(single_axis_points * single_axis_points)
            .into_iter()
            .map(|points| Vec::from(points))
            .collect()
    }

    pub fn get_one_edge_colors(
        sample_points_of_leds: &Vec<LedSamplePoints>,
        bitmap: &Vec<u8>,
        bytes_per_row: usize,
    ) -> Vec<LedColor> {
        let mut colors = vec![];
        for led_points in sample_points_of_leds {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            let len = led_points.len() as f64;
            for (x, y) in led_points {
                // log::info!("x: {}, y: {}, bytes_per_row: {}", x, y, bytes_per_row);
                let position = x * 4 + y * bytes_per_row;

                // Add bounds checking to prevent index out of bounds
                if position + 2 < bitmap.len() {
                    b += bitmap[position] as f64;
                    g += bitmap[position + 1] as f64;
                    r += bitmap[position + 2] as f64;
                } else {
                    // Skip invalid positions or use default values
                    log::warn!(
                        "Invalid pixel position: x={}, y={}, position={}, bitmap_len={}",
                        x,
                        y,
                        position,
                        bitmap.len()
                    );
                }
            }
            let color = LedColor::new((r / len) as u8, (g / len) as u8, (b / len) as u8);
            colors.push(color);
        }
        colors
    }

    pub fn get_one_edge_colors_by_cg_image(
        sample_points_of_leds: &Vec<LedSamplePoints>,
        bitmap: core_foundation::data::CFData,
        bytes_per_row: usize,
    ) -> Vec<LedColor> {
        let mut colors = vec![];
        for led_points in sample_points_of_leds {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            let len = led_points.len() as f64;
            for (x, y) in led_points {
                // log::info!("x: {}, y: {}, bytes_per_row: {}", x, y, bytes_per_row);
                let position = x * 4 + y * bytes_per_row;

                // Add bounds checking to prevent index out of bounds
                if position + 2 < bitmap.len() as usize {
                    b += bitmap[position] as f64;
                    g += bitmap[position + 1] as f64;
                    r += bitmap[position + 2] as f64;
                } else {
                    // Skip invalid positions or use default values
                    log::warn!("Invalid pixel position in CG image: x={}, y={}, position={}, bitmap_len={}", x, y, position, bitmap.len());
                }
                // log::info!("position: {}, total: {}", position, bitmap.len());
            }
            let color = LedColor::new((r / len) as u8, (g / len) as u8, (b / len) as u8);
            colors.push(color);
        }
        colors
    }

    pub async fn get_colors_by_sample_points(
        &self,
        points: &Vec<LedSamplePoints>,
    ) -> Vec<LedColor> {
        let bytes = self.bytes.read().await;

        Self::get_one_edge_colors(points, &bytes, self.bytes_per_row)
    }
}
type Point = (usize, usize);
pub type LedSamplePoints = Vec<Point>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ScreenSamplePoints {
    pub top: Vec<LedSamplePoints>,
    pub bottom: Vec<LedSamplePoints>,
    pub left: Vec<LedSamplePoints>,
    pub right: Vec<LedSamplePoints>,
}

pub struct DisplayColorsOfLedStrips {
    pub top: Vec<u8>,
    pub bottom: Vec<u8>,
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ScreenshotPayload {
    pub display_id: u32,
    pub height: u32,
    pub width: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ambient_light::Border;

    // Helper function to create a mock LedStripConfig
    fn mock_led_strip_config(border: Border, len: usize) -> LedStripConfig {
        LedStripConfig {
            index: 0,
            border,
            display_id: 1,
            len,
            led_type: crate::ambient_light::LedType::WS2812B,
            reversed: false,
        }
    }

    #[test]
    fn test_get_one_edge_sample_points_logic() {
        let leds = 10;
        let length = 1000;
        let width = 100;
        let single_axis_points = 5;

        let points =
            Screenshot::get_one_edge_sample_points(width, length, leds, single_axis_points);

        // Expect one group of points for each LED
        assert_eq!(points.len(), leds);

        // Expect each group to have single_axis_points * single_axis_points points
        assert_eq!(points[0].len(), single_axis_points * single_axis_points);
    }

    #[test]
    fn test_get_sample_points_for_each_border() {
        let screenshot = Screenshot::new(1, 1080, 1920, 1920 * 4, Arc::new(vec![]), 1.0, 1.0);

        let top_config = mock_led_strip_config(Border::Top, 100);
        let top_points = screenshot.get_sample_points(&top_config);
        assert!(!top_points.is_empty());
        assert_eq!(top_points.len(), 100);

        let bottom_config = mock_led_strip_config(Border::Bottom, 100);
        let bottom_points = screenshot.get_sample_points(&bottom_config);
        assert!(!bottom_points.is_empty());
        assert_eq!(bottom_points.len(), 100);
        // Verify that bottom points are transformed correctly
        assert!(bottom_points[0][0].1 > 1000);

        let left_config = mock_led_strip_config(Border::Left, 50);
        let left_points = screenshot.get_sample_points(&left_config);
        assert!(!left_points.is_empty());
        assert_eq!(left_points.len(), 50);

        let right_config = mock_led_strip_config(Border::Right, 50);
        let right_points = screenshot.get_sample_points(&right_config);
        assert!(!right_points.is_empty());
        assert_eq!(right_points.len(), 50);
        // Verify that right points are transformed correctly
        assert!(right_points[0][0].0 > 1900);
    }

    #[test]
    fn test_get_one_edge_colors_logic() {
        let width = 20;
        let height = 20;
        let bytes_per_row = width * 4;
        let mut bitmap = vec![0; height * bytes_per_row];

        // Create a solid red area in the bitmap
        for y in 0..10 {
            for x in 0..10 {
                let pos = y * bytes_per_row + x * 4;
                bitmap[pos] = 0; // B
                bitmap[pos + 1] = 0; // G
                bitmap[pos + 2] = 255; // R
                bitmap[pos + 3] = 255; // A
            }
        }

        // Sample points that fall entirely within the red area
        let sample_points: Vec<LedSamplePoints> = vec![
            vec![(2, 2), (3, 3)], // Points for LED 1
            vec![(5, 5), (6, 6)], // Points for LED 2
        ];

        let colors = Screenshot::get_one_edge_colors(&sample_points, &bitmap, bytes_per_row);

        assert_eq!(colors.len(), 2);
        // Both LEDs should be solid red
        assert_eq!(colors[0].get_rgb(), [255, 0, 0]);
        assert_eq!(colors[1].get_rgb(), [255, 0, 0]);
    }
}
