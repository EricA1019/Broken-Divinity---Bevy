use crate::core::components::TileKind;
use crate::core::resources::PlaceholderTileAtlas;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_ecs_tilemap::prelude::*;

const TILE_SIZE: u32 = 16;
const PLACEHOLDER_ATLAS_COLUMNS: u32 = 5;
const PLACEHOLDER_ATLAS_ROWS: u32 = 3;
const PLACEHOLDER_ATLAS_WIDTH: u32 = TILE_SIZE * PLACEHOLDER_ATLAS_COLUMNS;
const PLACEHOLDER_ATLAS_HEIGHT: u32 = TILE_SIZE * PLACEHOLDER_ATLAS_ROWS;
const PLACEHOLDER_TILE_COUNT: u32 = PLACEHOLDER_ATLAS_COLUMNS * PLACEHOLDER_ATLAS_ROWS;

pub fn spawn_tilemap_layer(
    commands: &mut Commands,
    _asset_server: &Res<AssetServer>,
    map_size: TilemapSize,
    _tiles: &[Vec<TileKind>],
    texture_handle: Handle<Image>,
) -> Entity {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for y in 0..map_size.y {
        for x in 0..map_size.x {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    // We arbitrarily give them default texture index here.
                    // Theme applies the real index later.
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TilemapGridSize {
            x: TILE_SIZE as f32,
            y: TILE_SIZE as f32,
        },
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size: TilemapTileSize {
            x: TILE_SIZE as f32,
            y: TILE_SIZE as f32,
        },
        anchor: bevy_ecs_tilemap::prelude::TilemapAnchor::Center,
        ..Default::default()
    });

    tilemap_entity
}

#[derive(Clone, Copy)]
enum PlaceholderGlyph {
    Wall,
    Floor,
    Door,
    StairsUp,
    StairsDown,
}

#[derive(Clone, Copy)]
struct PlaceholderTileSpec {
    base: [u8; 4],
    accent: [u8; 4],
    glyph: PlaceholderGlyph,
}

fn placeholder_tile_spec(index: u32) -> PlaceholderTileSpec {
    match index {
        0 => PlaceholderTileSpec {
            base: [58, 58, 66, 255],
            accent: [160, 160, 178, 255],
            glyph: PlaceholderGlyph::Wall,
        },
        1 => PlaceholderTileSpec {
            base: [82, 74, 62, 255],
            accent: [144, 128, 96, 255],
            glyph: PlaceholderGlyph::Floor,
        },
        2 => PlaceholderTileSpec {
            base: [96, 68, 40, 255],
            accent: [222, 180, 112, 255],
            glyph: PlaceholderGlyph::Door,
        },
        3 => PlaceholderTileSpec {
            base: [54, 70, 92, 255],
            accent: [168, 224, 255, 255],
            glyph: PlaceholderGlyph::StairsUp,
        },
        4 => PlaceholderTileSpec {
            base: [70, 52, 52, 255],
            accent: [255, 160, 136, 255],
            glyph: PlaceholderGlyph::StairsDown,
        },
        5 => PlaceholderTileSpec {
            base: [48, 58, 50, 255],
            accent: [146, 182, 152, 255],
            glyph: PlaceholderGlyph::Wall,
        },
        6 => PlaceholderTileSpec {
            base: [58, 74, 64, 255],
            accent: [126, 166, 138, 255],
            glyph: PlaceholderGlyph::Floor,
        },
        7 => PlaceholderTileSpec {
            base: [70, 88, 74, 255],
            accent: [210, 232, 166, 255],
            glyph: PlaceholderGlyph::Door,
        },
        8 => PlaceholderTileSpec {
            base: [50, 70, 78, 255],
            accent: [166, 224, 208, 255],
            glyph: PlaceholderGlyph::StairsUp,
        },
        9 => PlaceholderTileSpec {
            base: [72, 56, 70, 255],
            accent: [214, 166, 208, 255],
            glyph: PlaceholderGlyph::StairsDown,
        },
        10 => PlaceholderTileSpec {
            base: [52, 56, 62, 255],
            accent: [180, 192, 210, 255],
            glyph: PlaceholderGlyph::Wall,
        },
        11 => PlaceholderTileSpec {
            base: [66, 72, 82, 255],
            accent: [144, 168, 196, 255],
            glyph: PlaceholderGlyph::Floor,
        },
        12 => PlaceholderTileSpec {
            base: [96, 84, 60, 255],
            accent: [240, 212, 132, 255],
            glyph: PlaceholderGlyph::Door,
        },
        13 => PlaceholderTileSpec {
            base: [48, 72, 92, 255],
            accent: [156, 214, 255, 255],
            glyph: PlaceholderGlyph::StairsUp,
        },
        14 => PlaceholderTileSpec {
            base: [84, 54, 54, 255],
            accent: [255, 146, 146, 255],
            glyph: PlaceholderGlyph::StairsDown,
        },
        _ => PlaceholderTileSpec {
            base: [255, 0, 255, 255],
            accent: [0, 0, 0, 255],
            glyph: PlaceholderGlyph::Floor,
        },
    }
}

fn placeholder_pixel(spec: PlaceholderTileSpec, local_x: u32, local_y: u32) -> [u8; 4] {
    let border =
        local_x == 0 || local_y == 0 || local_x == TILE_SIZE - 1 || local_y == TILE_SIZE - 1;
    let center = TILE_SIZE / 2;
    let on_center_x = local_x == center || local_x + 1 == center;

    let glyph_pixel = match spec.glyph {
        PlaceholderGlyph::Wall => local_x % 4 == 0 || local_y % 4 == 0,
        PlaceholderGlyph::Floor => {
            ((5..=10).contains(&local_x) && (5..=10).contains(&local_y))
                && (local_x + local_y) % 2 == 0
        }
        PlaceholderGlyph::Door => (6..=9).contains(&local_x) && (2..=13).contains(&local_y),
        PlaceholderGlyph::StairsUp => {
            (on_center_x && (4..=11).contains(&local_y))
                || (local_y == 4 && (5..=10).contains(&local_x))
                || (local_y == 5 && (4..=11).contains(&local_x))
        }
        PlaceholderGlyph::StairsDown => {
            (on_center_x && (4..=11).contains(&local_y))
                || (local_y == 11 && (5..=10).contains(&local_x))
                || (local_y == 10 && (4..=11).contains(&local_x))
        }
    };

    if border || glyph_pixel {
        spec.accent
    } else {
        spec.base
    }
}

fn pixel_offset(x: u32, y: u32) -> usize {
    ((y * PLACEHOLDER_ATLAS_WIDTH + x) * 4) as usize
}

fn build_placeholder_tile_pixels() -> Vec<u8> {
    let mut pixels = vec![0; (PLACEHOLDER_ATLAS_WIDTH * PLACEHOLDER_ATLAS_HEIGHT * 4) as usize];

    for tile_index in 0..PLACEHOLDER_TILE_COUNT {
        let spec = placeholder_tile_spec(tile_index);
        let tile_origin_x = (tile_index % PLACEHOLDER_ATLAS_COLUMNS) * TILE_SIZE;
        let tile_origin_y = (tile_index / PLACEHOLDER_ATLAS_COLUMNS) * TILE_SIZE;

        for local_y in 0..TILE_SIZE {
            for local_x in 0..TILE_SIZE {
                let color = placeholder_pixel(spec, local_x, local_y);
                let offset = pixel_offset(tile_origin_x + local_x, tile_origin_y + local_y);
                pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }

    pixels
}

pub fn build_placeholder_tile_atlas_image() -> Image {
    Image::new(
        Extent3d {
            width: PLACEHOLDER_ATLAS_WIDTH,
            height: PLACEHOLDER_ATLAS_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        build_placeholder_tile_pixels(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub fn init_placeholder_tile_atlas(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    existing: Option<Res<PlaceholderTileAtlas>>,
) {
    if existing.is_some() {
        return;
    }

    let handle = images.add(build_placeholder_tile_atlas_image());
    commands.insert_resource(PlaceholderTileAtlas(handle));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba_at(data: &[u8], x: u32, y: u32) -> [u8; 4] {
        let offset = pixel_offset(x, y);
        [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]
    }

    #[test]
    fn placeholder_atlas_has_expected_size() {
        let pixels = build_placeholder_tile_pixels();
        assert_eq!(
            pixels.len(),
            (PLACEHOLDER_ATLAS_WIDTH * PLACEHOLDER_ATLAS_HEIGHT * 4) as usize
        );
    }

    #[test]
    fn placeholder_atlas_tiles_have_distinct_visuals() {
        let pixels = build_placeholder_tile_pixels();
        let urban_floor = rgba_at(&pixels, TILE_SIZE + 8, 8);
        let urban_door = rgba_at(&pixels, 2 * TILE_SIZE + 8, 8);
        let military_wall = rgba_at(&pixels, 8, 2 * TILE_SIZE + 8);

        assert_ne!(urban_floor, urban_door);
        assert_ne!(urban_floor, military_wall);
    }
}
