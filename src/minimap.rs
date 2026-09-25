use crate::scenario::ScenarioInfo;
use image::{Rgb, RgbImage};

#[derive(Clone, Copy)]
struct TerrainPalette {
    #[expect(dead_code)]
    up: [u8; 3],

    level: [u8; 3],

    #[expect(dead_code)]
    down: [u8; 3],
}

const GRASS: TerrainPalette = TerrainPalette {
    up: [0, 169, 0],
    level: [51, 151, 39],
    down: [0, 141, 0],
};

const FOREST: TerrainPalette = TerrainPalette {
    up: [37, 116, 57],
    level: [21, 118, 21],
    down: [0, 114, 0],
};

const DIRT: TerrainPalette = TerrainPalette {
    up: [243, 170, 92],
    level: [228, 162, 82],
    down: [218, 156, 105],
};

const BEACH_DESERT: TerrainPalette = TerrainPalette {
    up: [248, 201, 138],
    level: [232, 180, 120],
    down: [189, 150, 111],
};

const FARM: TerrainPalette = TerrainPalette {
    up: [138, 139, 87],
    level: [130, 136, 77],
    down: [118, 130, 65],
};

const SHALLOWS: TerrainPalette = TerrainPalette {
    up: [84, 146, 176],
    level: [84, 146, 176],
    down: [84, 146, 176],
};

const WATER: TerrainPalette = TerrainPalette {
    up: [48, 93, 182],
    level: [48, 93, 182],
    down: [48, 93, 182],
};

const DEEP_WATER: TerrainPalette = TerrainPalette {
    up: [0, 74, 161],
    level: [0, 74, 161],
    down: [0, 74, 161],
};

const MEDIUM_WATER: TerrainPalette = TerrainPalette {
    up: [0, 74, 187],
    level: [0, 74, 187],
    down: [0, 74, 187],
};

const AZURE_WATER: TerrainPalette = TerrainPalette {
    up: [0, 84, 176],
    level: [0, 84, 176],
    down: [0, 84, 176],
};

const ICE: TerrainPalette = TerrainPalette {
    up: [152, 192, 240],
    level: [152, 192, 240],
    down: [152, 192, 240],
};

const BLACK: TerrainPalette = TerrainPalette {
    up: [28, 28, 28],
    level: [28, 28, 28],
    down: [28, 28, 28],
};

const ROAD_FUNGUS: TerrainPalette = TerrainPalette {
    up: [243, 170, 92],
    level: [228, 162, 82],
    down: [189, 209, 253],
};

fn terrain_palette(terrain_id: u32) -> TerrainPalette {
    match terrain_id {
        0 | 5 | 9 | 12 | 16 | 60 | 68 | 69 | 70 | 71 | 72 | 73 | 74 | 76 | 77 | 83 | 84 | 85
        | 86 | 87 | 100 => GRASS,

        10 | 13 | 17 | 18 | 19 | 20 | 21 | 48 | 49 | 50 | 55 | 56 | 88 | 89 | 91 | 92 | 104
        | 105 | 106 => FOREST,

        3 | 6 | 11 | 24 | 25 | 27 | 36 | 40 | 41 | 42 | 78 => DIRT,

        2 | 14 | 45 | 46 | 51 | 52 | 53 | 79 | 80 | 81 | 82 | 101 | 102 | 107 | 108 | 109 => {
            BEACH_DESERT
        }

        7 | 8 | 29 | 30 | 31 | 32 | 63 | 64 | 65 | 66 | 67 => FARM,

        4 | 54 | 59 | 90 | 93 | 94 => SHALLOWS,

        1 | 15 | 28 | 95 | 96 | 97 | 98 | 99 => WATER,

        22 | 57 => DEEP_WATER,

        23 => MEDIUM_WATER,

        58 => AZURE_WATER,

        26 | 35 | 37 => ICE,

        47 => BLACK,

        75 => ROAD_FUNGUS,

        110 | 112 | 113 | 128 => FOREST,

        111 | 115 => SHALLOWS,

        114 | 130 => WATER,
        116 => DEEP_WATER,

        117..=121 => FARM,

        122 | 123 => GRASS,

        124..=126 => FARM, // temporary snow-family fallback
        127 => ICE,

        129 => BLACK,

        // obsolete terrains
        33 | 34 => FARM, // snow terrains
        38 | 43 | 44 | 103 => DIRT,
        39 => DIRT, // fungus road
        61 | 62 => GRASS,

        _ => TerrainPalette {
            up: [255, 0, 255],
            level: [255, 0, 255],
            down: [255, 0, 255],
        },
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
                let color = terrain_palette(terrain_id).level;

                image.put_pixel(screen_x, screen_y, Rgb(color));
            }
        }
    }

    image
}
