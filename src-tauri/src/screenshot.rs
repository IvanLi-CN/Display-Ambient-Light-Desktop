use std::iter;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;

use crate::{
    ambient_light::{LedStripConfig, LedStripConfigOfDisplays},
    led_color::LedColor,
};

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub display_id: u32,
    pub height: u32,
    pub width: u32,
    pub bytes_per_row: usize,
    pub bytes: Arc<RwLock<Vec<u8>>>,
    pub scale_factor: f32,
    pub sample_points: ScreenSamplePoints,
}

impl Screenshot {
    pub fn new(
        display_id: u32,
        height: u32,
        width: u32,
        bytes_per_row: usize,
        bytes: Vec<u8>,
        scale_factor: f32,
        sample_points: ScreenSamplePoints,
    ) -> Self {
        Self {
            display_id,
            height,
            width,
            bytes_per_row,
            bytes: Arc::new(RwLock::new(bytes)),
            scale_factor,
            sample_points,
        }
    }

    pub fn get_sample_points(
        &self,
        config: &LedStripConfig,
    ) -> Vec<LedSamplePoints> {
        let height = self.height as usize;
        let width = self.width as usize;

        match config.border {
            crate::ambient_light::Border::Top => {
                Self::get_one_edge_sample_points(height / 8, width, config.len, 5)
            }
            crate::ambient_light::Border::Bottom => {
                let points = Self::get_one_edge_sample_points(height / 9, width, config.len, 5);
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (x, height - y)).collect()
                    })
                    .collect()
            }
            crate::ambient_light::Border::Left => {
                let points = Self::get_one_edge_sample_points(width / 16, height, config.len, 5);
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (y, x)).collect()
                    })
                    .collect()
            }
            crate::ambient_light::Border::Right => {
                let points = Self::get_one_edge_sample_points(width / 16, height, config.len, 5);
                points
                    .into_iter()
                    .map(|groups| -> Vec<Point> {
                        groups.into_iter().map(|(x, y)| (width - y, x)).collect()
                    })
                    .collect()
            }
        }
    }

    // fn get_sample_points(config: DisplayConfig) -> ScreenSamplePoints {
    //     let top = match config.led_strip_of_borders.top {
    //         Some(led_strip_config) => Self::get_one_edge_sample_points(
    //             config.display_height / 8,
    //             config.display_width,
    //             led_strip_config.len,
    //             1,
    //         ),
    //         None => {
    //             vec![]
    //         }
    //     };

    //     let bottom: Vec<LedSamplePoints> = match config.led_strip_of_borders.bottom {
    //         Some(led_strip_config) => {
    //             let points = Self::get_one_edge_sample_points(
    //                 config.display_height / 9,
    //                 config.display_width,
    //                 led_strip_config.len,
    //                 5,
    //             );
    //             points
    //                 .into_iter()
    //                 .map(|groups| -> Vec<Point> {
    //                     groups
    //                         .into_iter()
    //                         .map(|(x, y)| (x, config.display_height - y))
    //                         .collect()
    //                 })
    //                 .collect()
    //         }
    //         None => {
    //             vec![]
    //         }
    //     };

    //     let left: Vec<LedSamplePoints> = match config.led_strip_of_borders.left {
    //         Some(led_strip_config) => {
    //             let points = Self::get_one_edge_sample_points(
    //                 config.display_width / 16,
    //                 config.display_height,
    //                 led_strip_config.len,
    //                 5,
    //             );
    //             points
    //                 .into_iter()
    //                 .map(|groups| -> Vec<Point> {
    //                     groups.into_iter().map(|(x, y)| (y, x)).collect()
    //                 })
    //                 .collect()
    //         }
    //         None => {
    //             vec![]
    //         }
    //     };

    //     let right: Vec<LedSamplePoints> = match config.led_strip_of_borders.right {
    //         Some(led_strip_config) => {
    //             let points = Self::get_one_edge_sample_points(
    //                 config.display_width / 16,
    //                 config.display_height,
    //                 led_strip_config.len,
    //                 5,
    //             );
    //             points
    //                 .into_iter()
    //                 .map(|groups| -> Vec<Point> {
    //                     groups
    //                         .into_iter()
    //                         .map(|(x, y)| (config.display_width - y, x))
    //                         .collect()
    //                 })
    //                 .collect()
    //         }
    //         None => {
    //             vec![]
    //         }
    //     };

    //     ScreenSamplePoints {
    //         top,
    //         bottom,
    //         left,
    //         right,
    //     }
    // }

    fn get_one_edge_sample_points(
        width: usize,
        length: usize,
        leds: usize,
        single_axis_points: usize,
    ) -> Vec<LedSamplePoints> {
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

    pub async fn get_colors(&self) -> DisplayColorsOfLedStrips {
        let bitmap = self.bytes.read().await;

        let top =
            Self::get_one_edge_colors(&self.sample_points.top, bitmap.as_ref(), self.bytes_per_row)
                .into_iter()
                .flat_map(|color| color.get_rgb())
                .collect();
        let bottom = Self::get_one_edge_colors(
            &self.sample_points.bottom,
            bitmap.as_ref(),
            self.bytes_per_row,
        )
        .into_iter()
        .flat_map(|color| color.get_rgb())
        .collect();
        let left = Self::get_one_edge_colors(
            &self.sample_points.left,
            bitmap.as_ref(),
            self.bytes_per_row,
        )
        .into_iter()
        .flat_map(|color| color.get_rgb())
        .collect();
        let right = Self::get_one_edge_colors(
            &self.sample_points.right,
            bitmap.as_ref(),
            self.bytes_per_row,
        )
        .into_iter()
        .flat_map(|color| color.get_rgb())
        .collect();
        DisplayColorsOfLedStrips {
            top,
            bottom,
            left,
            right,
        }
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
                let position = x * 4 + y * bytes_per_row;
                b += bitmap[position] as f64;
                g += bitmap[position + 1] as f64;
                r += bitmap[position + 2] as f64;
            }
            let color = LedColor::new((r / len) as u8, (g / len) as u8, (b / len) as u8);
            // paris::info!("color: {:?}", color.get_rgb());
            colors.push(color);
        }
        colors
    }

    pub async fn get_colors_by_sample_points(&self, points: &Vec<LedSamplePoints>) -> Vec<LedColor>  {
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
