use bevy::prelude::*;

/// Tile type (also a component for future collision / path-finding)
#[derive(Clone, Copy, PartialEq, Component)]
pub enum TileType {
    Grass,
    Stone,
    Dirt,
    Water,
}

impl TileType {
    pub fn walkable(&self) -> bool {
        !matches!(self, TileType::Water)
    }
}

/// All map textures in one resource
#[derive(Resource)]
pub struct MapAssets {
    pub grass: Handle<Image>,
    pub stone: Handle<Image>,
    pub dirt: Handle<Image>,
    pub water: Handle<Image>,
    pub wall: Handle<Image>,
    pub door: Handle<Image>,
    pub pillar: Handle<Image>,
    pub chest: Handle<Image>,
    pub torch: Handle<Image>,
    pub floor_corner: Handle<Image>,
}

/// Load every PNG from `assets/map/`
pub fn load_map_assets(mut commands: Commands, server: Res<AssetServer>) {
    let a = MapAssets {
        grass: server.load("map/Floor_Corner_01.png"),
        stone: server.load("map/WallBrick_Tall_01.png"),
        dirt: server.load("map/SmallChest_02.png"),
        water: server.load("map/Torch_Wall_01.png"),
        wall: server.load("map/WallBrick_Tall_01.png"),
        door: server.load("map/Door_05.png"),
        pillar: server.load("map/Pillar_03.png"),
        chest: server.load("map/SmallChest_02.png"),
        torch: server.load("map/Torch_Wall_01.png"),
        floor_corner: server.load("map/Floor_Corner_01.png"),
    };
    commands.insert_resource(a);
    info!("Map assets loaded.");
}

/// Build the isometric tilemap with real sprites
pub fn generate_map(
    mut commands: Commands,
    assets: Res<MapAssets>,
    images: Res<Assets<Image>>,
) {
    const MAP_SIZE: i32 = 20; // 40×40 tiles
    const TILE_SIZE: f32 = 1.0;
    const SPRITE_SCALE: f32 = 0.15; // tweak until tiles line up

    // Wait until at least one texture is ready
    if !images.contains(&assets.grass) {
        return;
    }

    for x in -MAP_SIZE..MAP_SIZE {
        for z in -MAP_SIZE..MAP_SIZE {
            let tile = generate_tile_type(x, z);

            // ----- isometric → screen coordinates -----
            let wx = x as f32 * TILE_SIZE;
            let wz = z as f32 * TILE_SIZE;
            let iso_x = (wx - wz) * 0.5;
            let iso_y = (wx + wz) * 0.25;

            // ----- base floor sprite -----
            let tex = match tile {
                TileType::Grass => &assets.grass,
                TileType::Stone => &assets.stone,
                TileType::Dirt => &assets.dirt,
                TileType::Water => &assets.water,
            };

            let mut sprite = Sprite::default();
            if tile == TileType::Water {
                sprite.custom_size = Some(Vec2::splat(1.0));
                sprite.color = Color::srgba(0.3, 0.5, 0.9, 0.7);
            }

            commands.spawn((
                SpriteBundle {
                    texture: tex.clone(),
                    sprite,
                    transform: Transform {
                        translation: Vec3::new(iso_x, iso_y + 0.1, 0.0),
                        scale: Vec3::splat(SPRITE_SCALE),
                        ..default()
                    },
                    ..default()
                },
                tile,
            ));

            // ----- walls on stone tiles -----
            if tile == TileType::Stone {
                commands.spawn(SpriteBundle {
                    texture: assets.wall.clone(),
                    transform: Transform {
                        translation: Vec3::new(iso_x, iso_y + 0.5, 0.0),
                        scale: Vec3::splat(SPRITE_SCALE * 1.2),
                        ..default()
                    },
                    ..default()
                });
            }

            // ----- random decor on grass -----
            if tile == TileType::Grass && (x.abs() + z.abs()) % 7 == 0 {
                let decor = if x % 3 == 0 {
                    &assets.pillar
                } else if x % 5 == 0 {
                    &assets.chest
                } else {
                    &assets.torch
                };
                commands.spawn(SpriteBundle {
                    texture: decor.clone(),
                    transform: Transform {
                        translation: Vec3::new(iso_x, iso_y + 0.4, 0.0),
                        scale: Vec3::splat(SPRITE_SCALE * 0.9),
                        ..default()
                    },
                    ..default()
                });
            }
        }
    }
    info!("Map generated – {}×{} tiles", MAP_SIZE * 2, MAP_SIZE * 2);
}

/// Very simple procedural generation
fn generate_tile_type(x: i32, z: i32) -> TileType {
    let v = ((x * 73 + z * 179) % 100) as f32 / 100.0;

    if (x % 17 == 0 || z % 23 == 0) && v < 0.3 {
        TileType::Water
    } else if v > 0.85 {
        TileType::Stone
    } else if v < 0.15 {
        TileType::Dirt
    } else {
        TileType::Grass
    }
}