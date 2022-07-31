use std::{f32::consts::PI, borrow::BorrowMut};

use bevy::{math::const_vec3, prelude::*, render::render_resource::FilterMode};
use bevy_ecs_tilemap::Tilemap2dPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tweening::{
    lens::TransformRotateZLens, Animator, EaseFunction, Tween, Tweenable, TweeningPlugin,
};
// do it

mod chunks;

const CAMERA_SPEED: Vec3 = const_vec3!([1.0, 1.0, 0.0]);

#[derive(Component)]
struct LinearFilter;

fn set_image_filtering(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &Handle<Image>), (Without<LinearFilter>,)>,
) {
    for (entity, handle) in query.iter() {
        if let Some(sampler) = images.get_mut(handle).map(|s| &mut s.sampler_descriptor) {
            sampler.mag_filter = FilterMode::Nearest;
            sampler.min_filter = FilterMode::Nearest;
            commands.entity(entity).insert(LinearFilter);
        }
    }
}

fn player_movement_system(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Player, Option<&mut Animator<Transform>>), With<Player>>,
) {
    let (entity, mut player, animator) = query.get_single_mut().unwrap();
    let mut vec = bevy::prelude::Vec3::new(0.0, 0.0, 0.0);
    if keys.pressed(KeyCode::W) {
        vec.y = 1.0;
    }
    if keys.pressed(KeyCode::A) {
        // Left Ctrl was released
        vec.x = -1.0;
    }
    if keys.pressed(KeyCode::S) {
        // W is being held down
        vec.y = -1.0;
    }
    // we can check multiple at once with `.any_*`
    if keys.pressed(KeyCode::D) {
        // Either the left or right shift are being held down
        vec.x = 1.0;
    }

    player.current_impulse = vec;
    let vcel = player.vertical_acceleration;
    let hcel = player.horizontal_acceleration;
    let cur = player.current_impulse;
    player.velocity += cur * time.delta_seconds() * Vec3::new(hcel, vcel, 0.0);
}

fn player_physics_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Player, &mut Transform), With<Player>>,
    world: Query<&GameWorld>
) {
    for (entity, mut player, mut transform) in query.iter_mut() {
        let world = world.get_single().unwrap();
        transform.translation += player.velocity + Vec3::new(0.0, world.gravity, 0.0);
        println!("{:?}", transform.translation);
    }
}

fn player_animation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Player, Option<&mut Animator<Transform>>), With<Player>>,
    mut prev_target_rotation: Local<Option<Quat>>,
) {
    let (entity, transform, player, animator) = query.get_single_mut().unwrap();
        
    let target_rotation = Quat::from_rotation_z(PI / 5.0);

    // if input has changed, tween to next input
    if let Some(&mut prev_target_rotation) = prev_target_rotation.as_mut() {
        if prev_target_rotation != target_rotation {
           println!("input vec changed");
           if let Some(mut a) = animator {
               a.stop();
           }
            let tween = Tween::new(
                EaseFunction::CubicInOut,
                bevy_tweening::TweeningType::Once,
                std::time::Duration::from_millis(250),
                TransformRotateZLens {
                    start: transform.rotation.z,
                    end: target_rotation.z,
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
        }
    }
    *prev_target_rotation = Some(target_rotation);

}

#[derive(Component)]
struct GameWorld {
    gravity: f32,
}

fn setup(mut commands: Commands) {
    for i in -1..2 {
        for j in -1..2 {
            commands.spawn().insert(chunks::Chunk::new(i, j));
        }
    }

    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scale = 0.25;
    commands.spawn_bundle(camera);
    commands.spawn().insert(GameWorld { gravity: -1.0 });
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);
fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

#[derive(Component)]
struct Player {
    velocity: Vec3,
    current_impulse: Vec3,
    vertical_acceleration: f32,
    horizontal_acceleration: f32,
}

impl Player {
    fn new() -> Self {
        Player {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            current_impulse: Vec3::new(0.0, 0.0, 0.0),
            vertical_acceleration: 100.0,
            horizontal_acceleration: 100.0,
        }
    }
}

fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("digger.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 48.0), 6, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0., 0., 3.),
            ..default()
        })
        .insert(Player::new())
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
        .insert(FollowTarget);
}

#[derive(Component)]
struct FollowTarget;

fn bind_follow_target_camera(
    follow_target: Query<&Transform, With<FollowTarget>>,
    mut camera_transform: Query<&mut Transform, (With<Camera>, Without<FollowTarget>)>,
) {
    let follow_target = follow_target.get_single().unwrap();

    for mut camera_transform in camera_transform.iter_mut() {
        camera_transform.translation = follow_target.translation;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .add_startup_system(player_setup)
        .add_system(set_image_filtering)
        .add_system(player_movement_system)
        .add_system(player_physics_system)
        .add_system(player_animation_system)
        .add_system(bind_follow_target_camera)
        .add_system(chunks::chunk_spawner)
        .add_system(animate_sprite)
        .add_plugin(Tilemap2dPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}
