use bevy::prelude::*;
use rand::Rng;
use bevy::render::render_resource::FilterMode;

use bevy_ecs_tilemap::{
    map::{
        Tilemap2dGridSize, Tilemap2dSize, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapTexture,
    },
    tiles::{Tile2dStorage, TileBundle, TilePos2d, TileTexture, TileVisible},
    Tilemap2dPlugin, TilemapBundle,
};


#[derive(Component, Clone)]
pub struct Chunk {
    x: i32,
    y: i32,
}


impl Chunk {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

fn get_global_tile_position(chunk: &Chunk, tilemap: &Tilemap2dSize, pos: &TilePos2d) -> IVec2
{
    return IVec2::new(chunk.x * tilemap.x as i32 + pos.x as i32, chunk.y * tilemap.y as i32 + pos.y as i32);
}

#[derive(Component)]
pub struct Loaded;


pub fn chunk_spawner(mut commands: Commands, asset_server: Res<AssetServer>,
                     mut query: Query<(Entity, &Chunk), Without<Loaded>>
) {
    let mut rng = rand::thread_rng();
    for chunk in query.iter() {
        println!("Spawning chunk at ({}, {})", chunk.1.x, chunk.1.y);
        let texture_handle: Handle<Image> = asset_server.load("tileset.png");

        let tilemap_size = Tilemap2dSize { x: 32, y: 32 };
        let mut tile_storage = Tile2dStorage::empty(tilemap_size);
        let l1 = commands.spawn().id();

        for x in 0..tilemap_size.x {
            for y in 0..tilemap_size.y {
                let tile_pos = TilePos2d { x, y };

                let v: u8 = rng.gen();


                let position = get_global_tile_position(&chunk.1, &tilemap_size, &tile_pos);

                let texture = if position.y == 0 { Some(5) } else if position.y < 0 { Some(21) } else { None };
                
                if let Some(texture) = texture {
                    let tile_entity = commands
                        .spawn()
                        .insert_bundle(TileBundle {
                            position: tile_pos,
                            texture: TileTexture(texture),
                            tilemap_id: TilemapId(l1),
                            ..Default::default()
                        })
                        .id();
                    
                    tile_storage.set(&tile_pos, Some(tile_entity));
                }
            }
        }

        let tile_size = Tilemap2dTileSize { x: 16.0, y: 16.0 };

        let mut t1 = bevy_ecs_tilemap::helpers::get_centered_transform_2d(
            &tilemap_size,
            &tile_size,
            0.0,
        );

        t1.translation += Vec3::new(chunk.1.x as f32 * tilemap_size.x as f32 * tile_size.x,
                                    chunk.1.y as f32 * tilemap_size.y as f32 * tile_size.y,
                                    1.0);

        // FG
        commands
            .entity(l1)
            .insert_bundle(TilemapBundle {
                grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
                size: tilemap_size,
                storage: tile_storage,
                texture_size: Tilemap2dTextureSize { x: 320.0, y: 320.0 },
                texture: TilemapTexture(texture_handle.clone()),
                tile_size,
                transform: t1,
                ..Default::default()
            });

        // Layer 2
        let mut tile_storage = Tile2dStorage::empty(tilemap_size);
        let l2 = commands.spawn().id();

        bevy_ecs_tilemap::helpers::fill_tilemap(
            TileTexture(54),
            tilemap_size,
            TilemapId(l2),
            &mut commands,
            &mut tile_storage,
        );

        let mut t2 = bevy_ecs_tilemap::helpers::get_centered_transform_2d(
            &tilemap_size,
            &tile_size,
            0.0,
        );

        t2.translation += Vec3::new(chunk.1.x as f32 * tilemap_size.x as f32 * tile_size.x,
                                    chunk.1.y as f32 * tilemap_size.y as f32 * tile_size.y,
                                    0.0);

        // BG
        commands
            .entity(l2)
            .insert_bundle(TilemapBundle {
                grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
                size: tilemap_size,
                storage: tile_storage,
                texture_size: Tilemap2dTextureSize { x: 320.0, y: 320.0 },
                texture: TilemapTexture(texture_handle),
                tile_size: Tilemap2dTileSize { x: 16.0, y: 16.0 },
                transform: t2,
                ..Default::default()
            });

        // mark it as loaded and mark the new layers as children
        commands.entity(chunk.0).insert(Loaded).push_children(&[l1, l2]);
    }
}
