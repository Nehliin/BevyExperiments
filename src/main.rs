use bevy::{
    asset::AssetPlugin,
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    input::keyboard::KeyboardInput,
    prelude::*,
    render::{
        camera::{Camera, OrthographicProjection, VisibleEntities},
        pass::ClearColor,
    },
    scene::ScenePlugin,
    sprite::SpritePlugin,
    type_registry::{TypeRegistry, TypeRegistryPlugin},
};
use std::{fs::File, io::Write};
use tilemap::{TileMap, TileMapLoader, TileMapPlugin, TileMapSpawner};
#[derive(Properties, Default)]
struct Player;
#[derive(Properties, Default)]
struct Enemy;
#[derive(Properties, Default)]
struct Health(u8);

fn create_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut tile_maps: ResMut<TileMapSpawner>,
) {
    let enemy_size = Vec2::new(40.0, 30.0);
    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            translation: Translation(Vec3::new(100.0, 150.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(40.0, 80.0),
            },
            ..Default::default()
        })
        .with(Player)
        .with(Health(100))
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.2, 1.0, 0.5).into()),
            translation: Translation(Vec3::new(0.0, 5.0, 0.0)),
            sprite: Sprite { size: enemy_size },
            ..Default::default()
        })
        .with(Enemy)
        .with(Health(100))
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(1.0, 0.2, 0.8).into()),
            translation: Translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite { size: enemy_size },
            ..Default::default()
        })
        .with(Enemy)
        .with(Health(100))
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
            translation: Translation(Vec3::new(25.0, 25.0, 0.0)),
            sprite: Sprite { size: enemy_size },
            ..Default::default()
        })
        .with(Enemy)
        .with(Health(100));

    let tilemap: Handle<TileMap> = asset_server.load("assets/map.json").unwrap();

    tile_maps.spawn(tilemap, 0);
    //let map: Handle<TileMap> = asset_server.load("assets/scenes/test.json").unwrap();
    println!("Done");
}

fn load_world(asset_server: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>) {
    let scene_handle: Handle<Scene> = asset_server.load("assets/scenes/start_scene.scn").unwrap();

    scene_spawner.load(scene_handle);
    println!("loaded: {:?}", asset_server.get_load_state(scene_handle));
    asset_server.watch_for_changes().unwrap();
}

fn save_scene(world: &mut World, resources: &mut Resources) {
    let type_registry = resources.get::<TypeRegistry>().unwrap();
    let scene = Scene::from_world(&world, &type_registry.component.read().unwrap());

    let mut file = File::create("assets/scenes/start_scene.scn").unwrap();
    file.write_all(
        scene
            .serialize_ron(&type_registry.property.read().unwrap())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
}
#[derive(Default)]
struct KeyboardEventReader {
    event_reader: EventReader<KeyboardInput>,
}

fn input_system(
    mut keyboard_event_reader: ResMut<KeyboardEventReader>,
    keyboard_events: Res<Events<KeyboardInput>>,
) {
    for event in keyboard_event_reader.event_reader.iter(&keyboard_events) {
        println!("Event: {:?}", event);
    }
}

fn direct_input(world: &mut World, resources: &mut Resources) {
    let mut should_save = false;
    {
        let key_input = resources.get::<Input<KeyCode>>().unwrap();
        if key_input.pressed(KeyCode::LControl) && key_input.pressed(KeyCode::S) {
            should_save = true;
        }
    }
    if should_save {
        println!("saved!");
        save_scene(world, resources);
    }
}

// This system prints all ComponentA components in our world. Try making a change to a ComponentA in load_scene_example.scn.
// You should immediately see the changes appear in the console.
fn print_system(mut query: Query<Entity>) {
    for entity in &mut query.iter() {
        println!("  Entity({})", entity.id());
    }
}

fn camera_movement_system(
    mut keyboard_event_reader: ResMut<KeyboardEventReader>,
    keyboard_events: Res<Events<KeyboardInput>>,
    mut query: Query<(&Camera, &mut Translation)>,
) {
    for event in keyboard_event_reader.event_reader.iter(&keyboard_events) {
        let mut horisontal_movement = 0.0;
        let mut latteral_movement = 0.0;
        match event.key_code {
            Some(KeyCode::D) => horisontal_movement += 30.0,
            Some(KeyCode::A) => horisontal_movement -= 30.0,
            Some(KeyCode::W) => latteral_movement += 30.0,
            Some(KeyCode::S) => latteral_movement -= 30.0,
            _ => {}
        }
        for (_, mut translation) in &mut query.iter() {
            *dbg!(translation.x_mut()) += horisontal_movement;
            *dbg!(translation.y_mut()) += latteral_movement;
        }
    }
}

mod tilemap;

fn main() {
    env_logger::init();

    App::build()
        .add_default_plugins()
        .add_plugin(TileMapPlugin)
        .init_resource::<KeyboardEventReader>()
        .register_component::<Health>()
        .register_component::<Player>()
        .register_component::<Enemy>()
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        // .add_plugin(PrintDiagnosticsPlugin::default())
        .add_startup_system(create_world.system())
        .add_system(camera_movement_system.system())
        // .add_startup_system(load_world.system())
        .add_system(direct_input.thread_local_system())
        //.add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        // .add_system(save_scene_system.thread_local_system())
        //.add_startup_stage("start_stage")
        //.add_stage("test")
        //.add_stage_after("test", "load_stage")
        //.add_startup_system_to_stage("start_stage", system_one.system())
        //.add_system_to_stage("test", system_two.system())
        //.add_system_to_stage("load_stage", system_three.system())
        //.add_startup_system_to_stage("load_stage", system_two.system())
        // .add_startup_system(save_scene_system.thread_local_system())
        .run();
}
