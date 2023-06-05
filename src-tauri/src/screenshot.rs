
use std::fmt::Formatter;
use std::{iter, fmt::Debug};
use std::sync::Arc;

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
                let points = Self::get_one_edge_sample_points(height / 18, width, config.len, SINGLE_AXIS_POINTS);
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (x, height - y)).collect()
                    })
                    .collect()
            }
            crate::ambient_light::Border::Left => {
                let points = Self::get_one_edge_sample_points(width / 32, height, config.len, SINGLE_AXIS_POINTS);
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (y, x)).collect()
                    })
                    .collect()
            }
            crate::ambient_light::Border::Right => {
                let points = Self::get_one_edge_sample_points(width / 32, height, config.len, SINGLE_AXIS_POINTS);
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
                b += bitmap[position] as f64;
                g += bitmap[position + 1] as f64;
                r += bitmap[position + 2] as f64;
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
                b += bitmap[position] as f64;
                g += bitmap[position + 1] as f64;
                r += bitmap[position + 2] as f64;
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
