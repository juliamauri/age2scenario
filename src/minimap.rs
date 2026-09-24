use crate::scenario::ScenarioInfo;
use image::{Rgb, RgbImage};

fn terrain_color(terrain_id: u32) -> [u8; 3] {
    match terrain_id {
        0 => [70, 125, 55],
        2 => [210, 190, 130],
        3 => [135, 100, 65],
        9 => [95, 145, 65],
        12 => [80, 135, 55],
        13 => [45, 95, 45],
        14 => [215, 180, 110],
        22 => [35, 70, 130],
        23 => [50, 105, 175],
        24 => [155, 130, 95],
        40 => [110, 110, 110],
        _ => [255, 0, 255],
    }
}

pub(crate) fn render_isometric_minimap(scenario: &ScenarioInfo) -> RgbImage {
    let tile_width: u32 = 8;
    let tile_height: u32 = 4;
    let padding: u32 = 4;

    let half_width = tile_width as f32 / 2.0;
    let half_height = tile_height as f32 / 2.0;

    let image_width = (scenario.width + scenario.height) * (tile_width / 2) + padding * 2;
    let image_height = (scenario.width + scenario.height) * (tile_height / 2) + padding * 2;

    let mut image = RgbImage::new(image_width, image_height);

    let origin_x = (scenario.height * (tile_width / 2) + padding) as f32;
    let origin_y = padding as f32;

    for screen_y in 0..image_height {
        for screen_x in 0..image_width {
            let projected_x = (screen_x as f32 - origin_x) / half_width;
            let projected_y = (screen_y as f32 - origin_y) / half_height;

            let map_x = (projected_x + projected_y) / 2.0;
            let map_y = (projected_y - projected_x) / 2.0;

            let tile_x = map_x.floor() as i32;
            let tile_y = map_y.floor() as i32;

            if tile_x >= 0
                && tile_y >= 0
                && tile_x < scenario.width as i32
                && tile_y < scenario.height as i32
            {
                let index = (tile_y as u32 * scenario.width + tile_x as u32) as usize;
                let terrain_id = scenario.terrain[index];
                let color = terrain_color(terrain_id);

                image.put_pixel(screen_x, screen_y, Rgb(color));
            }
        }
    }

    image
}
