use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::utils::HashMap;
use bevy::window::CursorGrabMode;
use bevy_mod_raycast::prelude::*;
use bevy_sprite3d::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            Sprite3dPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (cursor_grab_system, mouse_motion, input_system, move_player, player_attack, weapon_load, animate_effect, animate_decals))
        .add_systems(Update, (spawn_effects, spawn_decals).after(player_attack))
        .run();
}

#[derive(Component)]
pub struct Collidable;

#[derive(Component, Deref, DerefMut)]
struct EffectAnimation(Timer);

#[derive(Component, Deref, DerefMut)]
struct Decal(Timer);

#[derive(Component)]
pub struct Player {
    weapon: Entity
}

#[derive(Component)]
#[derive(Default)]
pub struct Weapon {
    frame: i32,
    loaded: bool
}

#[derive(Resource)]
#[derive(Default)]
pub struct InputData {
    movement: Vec2,
    aim_movement: Vec2,
    fire: bool
}

#[derive(Resource)]
#[derive(Default)]
pub struct Sprites {
    sprites: HashMap<String, Handle<TextureAtlas>>
}

#[derive(Resource)]
#[derive(Default)]
pub struct Effects {
    queued: Vec<(Vec3, String)>
}

#[derive(Resource)]
#[derive(Default)]
pub struct DecalQueue {
    queued: Vec<(Vec3, Vec3, String)>
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.init_resource::<InputData>();
    commands.init_resource::<Effects>();
    commands.init_resource::<DecalQueue>();

    let mut sprites: HashMap<String, Handle<TextureAtlas>> = HashMap::new();

    sprites.insert("wep_pistol".to_owned(), texture_atlases.add(TextureAtlas::from_grid(asset_server.load("pistol.png"), Vec2::new(110.0, 150.0), 2, 1, None, None)));
    sprites.insert("fx_splat".to_owned(), texture_atlases.add(TextureAtlas::from_grid(asset_server.load("splat.png"), Vec2::new(200.0, 200.0), 1, 1, None, None)));

    let mut mat : StandardMaterial = Color::WHITE.into();
    mat.depth_bias = -1000.0;
    // circular base
    commands.spawn((PbrBundle {
        mesh: meshes.add(shape::Circle::new(8.0).into()),
        material: materials.add(mat),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    },Collidable));
    mat = Color::rgb_u8(124, 144, 255).into();
    mat.depth_bias = -1000.0;
    // cube
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        material: materials.add(mat),
        transform: Transform::from_xyz(0.0, 1.0, -4.0),
        ..default()
    }, Collidable));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // ui

    //Recticle
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(10.),
            height: Val::Px(10.),
            position_type: PositionType::Absolute,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },
        background_color: Color::rgb(1.0, 0.0, 0.0).into(),
        ..default()
    });

    let pistol = commands.spawn((
        Weapon{
            ..default()
        },
        AtlasImageBundle {
            style: Style {
                width: Val::Px(220.),
                height: Val::Px(300.),
                position_type: PositionType::Absolute,
                align_self: AlignSelf::FlexEnd,
                justify_self: JustifySelf::Center,
                ..default()
            },
            texture_atlas: sprites["wep_pistol"].clone(),
            texture_atlas_image: UiTextureAtlasImage::default(),
            ..default()
        }
    )).id();

    // player/camera
    commands.spawn((
        Player { weapon: pistol },
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.5, 4.0),
            ..default()
        }
    ));

    commands.insert_resource(Sprites { sprites: sprites });
}

fn input_system(mut input: ResMut<InputData>, keyboard_input: Res<Input<KeyCode>>, mouse_buttons: Res<Input<MouseButton>>) {
    input.movement = Vec2{ x: 0., y: 0. };
    input.fire = false;
    if mouse_buttons.just_pressed(MouseButton::Left) {
        input.fire = true;
    }
    if keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]) {
        input.movement.y = 1.;
    }
    if keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]) {
        input.movement.x = -1.;
    }
    if keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]) {
        input.movement.y = -1.;
    }
    if keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]) {
        input.movement.x = 1.;
    }
}

fn weapon_load(mut weapons: Query<(&mut Weapon, &mut UiTextureAtlasImage)>){
    for (mut weapon, mut image) in &mut weapons {
        if weapon.loaded == false && weapon.frame <= 0 {
            image.index = 0;
            weapon.loaded = true;
        }else if weapon.frame > 0{
            image.index = 1;
            weapon.frame -= 1;
        }
    }
}

fn move_player(mut input: ResMut<InputData>, time: Res<Time>, mut players: Query<(&mut Transform, &Player)>/*, mut weapons: Query<&mut Weapon>*/){
    for (mut transform, _player) in &mut players {

        //Camera rotate code modified from https://github.com/sburris0/bevy_flycam/tree/master
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        yaw += input.aim_movement.x * -0.08 * time.delta_seconds();
        pitch += input.aim_movement.y * -0.08 * time.delta_seconds();
        pitch = pitch.clamp(-1.54, 1.54);
        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        input.aim_movement = Vec2{ x: 0., y: 0. };

        let mut move_delta = Quat::from_axis_angle(Vec3::Y, yaw) * Vec3{ x: input.movement.x, y: 0., z: -input.movement.y };
        move_delta *= 5.;
        move_delta *= time.delta_seconds();
        transform.translation += move_delta;
    }
}

fn player_attack(
    input: ResMut<InputData>, 
    mut decalqueue: ResMut<DecalQueue>, 
    players: Query<(&Transform, &Player)>,
    mut objects: Query<&mut Transform, Without<Player>>,
    mut weapons: Query<&mut Weapon>, 
    collidable_query: Query<With<Collidable>>,
    mut raycast: Raycast){
    if input.fire {
        for (transform, player) in &players {
            if let Ok(mut weapon) = weapons.get_mut(player.weapon) {
                if weapon.loaded {
                    weapon.loaded = false;
                    weapon.frame = 15;
                    
                    let ray = Ray3d::new(transform.translation, transform.forward());
                    let hits = raycast.cast_ray(ray, &RaycastSettings::default()
                        .with_early_exit_test(&|_entity| true)
                        .with_filter(&|entity| collidable_query.contains(entity))
                    );
                    for (hitentity, hitdata) in hits {
                        decalqueue.queued.push((hitdata.position(), hitdata.normal(), "fx_splat".to_owned()));
                        if let Ok(mut entity) = objects.get_mut(*hitentity) {
                            //entity.translation += transform.forward();
                        }
                    }
                }
            }
        }
    }
}

fn spawn_effects(
    mut commands: Commands,
    sprites: Res<Sprites>, 
    mut effects: ResMut<Effects>, 
    mut sprite_params : Sprite3dParams,
    cameras: Query<&Transform, (With<Camera>, Without<EffectAnimation>)>){
    for (position, effect_sprite) in &effects.queued {
        for cam_transform in cameras.iter() {
            let (roty, _, _) = cam_transform.rotation.to_euler(EulerRot::YXZ);
            commands.spawn(AtlasSprite3d {
                atlas: sprites.sprites[effect_sprite].clone(),

                pixels_per_metre: 100.,
                alpha_mode: AlphaMode::Blend,
                unlit: true,

                transform: Transform::from_translation(*position).with_rotation(Quat::from_axis_angle(Vec3::Y, roty)),
                pivot: Some(Vec2::new(0.5, 0.5)),

                ..default()
            }.bundle(&mut sprite_params))
            .insert(EffectAnimation(Timer::from_seconds(0.1, TimerMode::Repeating)));
            break; //only spawn for one camera
        }
    }
    effects.queued.clear();
}

fn spawn_decals(
    mut commands: Commands,
    sprites: Res<Sprites>, 
    mut decals: ResMut<DecalQueue>, 
    mut sprite_params : Sprite3dParams){
    for (position, normal, decal_sprite) in &decals.queued {
        commands.spawn(AtlasSprite3d {
            atlas: sprites.sprites[decal_sprite].clone(),

            pixels_per_metre: 100.,
            alpha_mode: AlphaMode::Blend,
            unlit: true,

            transform: Transform::from_translation(*position).looking_at(*position + *normal, Vec3::Y),
            pivot: Some(Vec2::new(0.5, 0.5)),

            ..default()
        }.bundle(&mut sprite_params))
        .insert(Decal(Timer::from_seconds(10.0, TimerMode::Repeating)));
        break; //only spawn for one camera
    }
    decals.queued.clear();
}

//modified from https://github.com/FraserLee/bevy_sprite3d/blob/main/examples/sprite_sheet.rs
fn animate_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut EffectAnimation, &mut AtlasSprite3dComponent)>,
    cameras: Query<&Transform, (With<Camera>, Without<EffectAnimation>)>
) {
    for (entity, mut transform, mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = (sprite.index + 1) % sprite.atlas.len();
            if sprite.index == 0 {
                commands.entity(entity).despawn();
            }
        }
        for cam_transform in cameras.iter() {
            let (roty, _, _) = cam_transform.rotation.to_euler(EulerRot::YXZ);
            transform.rotation = Quat::from_axis_angle(Vec3::Y, roty);
        }
    }
}

fn animate_decals(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Decal, &mut AtlasSprite3dComponent)>
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = (sprite.index + 1) % sprite.atlas.len();
            if sprite.index == 0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

//Modified from https://bevy-cheatbook.github.io/window/mouse-grab.html
fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    for mut window in &mut windows {

        if btn.just_pressed(MouseButton::Left) {
            // if you want to use the cursor, but not let it leave the window,
            // use `Confined` mode:
            window.cursor.grab_mode = CursorGrabMode::Confined;

            // also hide the cursor
            window.cursor.visible = false;
        }

        if key.just_pressed(KeyCode::Escape) {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

//https://bevy-cheatbook.github.io/input/mouse.html
fn mouse_motion(
    mut motion_evr: EventReader<MouseMotion>,
    mut input: ResMut<InputData>,
) {
    for ev in motion_evr.read() {
        input.aim_movement += Vec2{ x:ev.delta.x, y:ev.delta.y }
    }
}