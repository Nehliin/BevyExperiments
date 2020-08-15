use bevy::asset::{AssetLoader, HandleId, LoadState};
use bevy::{prelude::*, sprite::TextureAtlasBuilder};
use serde::Deserialize;
use std::fs::File;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Deserialize)]
pub struct TileMap {
    height: u32,
    width: u32,
    orientation: String,
    layers: Vec<TileLayer>,
    tilesets: Vec<TileSet>,
    tilewidth: u32,
    tileheight: u32,
}
#[derive(Debug, Default, Deserialize)]
pub struct TileLayer {
    data: Vec<u32>,
    height: u32,
    width: u32,
    id: u32,
    name: String,
    opacity: f32,
    x: u32,
    y: u32,
}
#[derive(Debug, Default, Deserialize)]
pub struct Grid {
    height: u32,
    width: u32,
    orientation: String,
}
#[derive(Debug, Default, Deserialize)]
pub struct TileSet {
    columns: u32,
    firstgid: i32,
    grid: Grid,
    margin: u32,
    name: String,
    spacing: u32,
    tilecount: u32,
    tileheight: u32,
    tilewidth: u32,
    tiles: Vec<Tile>,
}
#[derive(Debug, Default, Deserialize)]
pub struct Tile {
    id: u32,
    image: String,
    imageheight: u32,
    imagewidth: u32,
}
#[derive(Default, Debug)]
pub struct TileMapLoader;

impl AssetLoader<TileMap> for TileMapLoader {
    fn from_bytes(&self, _asset_path: &Path, bytes: Vec<u8>) -> Result<TileMap, anyhow::Error> {
        let tile_map = serde_json::from_slice(&bytes)?;
        Ok(tile_map)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Default)]
pub struct TileMapSpawner {
    loaded_maps: HashMap<Handle<TileMap>, Handle<TextureAtlas>>,
    tilemap_event_reader: EventReader<AssetEvent<TileMap>>,
    staged_maps: HashMap<Handle<TileMap>, Vec<Handle<Texture>>>,
    to_be_spawned: Vec<Handle<TileMap>>,
}

impl TileMapSpawner {
    // show layer?
    pub fn spawn(&mut self, handle: Handle<TileMap>) {
        self.to_be_spawned.push(handle);
    }

    pub fn is_loaded(&self, handle: Handle<TileMap>) -> bool {
        self.loaded_maps.contains_key(&handle)
    }

    fn stage_map(
        &mut self,
        handle: Handle<TileMap>,
        tilemap_assets: &Assets<TileMap>,
        asset_server: &AssetServer,
    ) {
        if self.staged_maps.contains_key(&handle) {
            return;
        }

        let tilemap = tilemap_assets.get(&handle).unwrap();

        let tiles = tilemap.tilesets.iter().flat_map(|set| &set.tiles);
        let mut texture_handles = Vec::with_capacity(tiles.size_hint().0);

        for tile in tiles {
            File::open(PathBuf::from(&tile.image)).unwrap();
            let texture_handle: Handle<Texture> =
                asset_server.load(dbg!(PathBuf::from(&tile.image))).unwrap();

            texture_handles.push(texture_handle);
        }

        self.staged_maps.insert(handle, texture_handles);
    }

    fn poll_staged_maps(
        &mut self,
        asset_server: &AssetServer,
        texture_store: &mut Assets<Texture>,
        texture_atlas_store: &mut Assets<TextureAtlas>,
    ) {
        let mut hack_remove = Vec::new();
        for (tilemap_handle, staged_textures) in self.staged_maps.iter() {
            let still_loading = staged_textures.iter().any(|handle| {
                match asset_server.get_load_state(*handle).unwrap() {
                    LoadState::Loaded(..) => false,
                    LoadState::Failed(tmp) => {
                        // TODO: find out why these fails
                       // dbg!(*handle);
                        false
                    }
                    _ => true,
                }
            });

            if !still_loading {
                println!("DONE LOADING!");
                let mut texture_atlas_builder = TextureAtlasBuilder::default();
                for texture_handle in staged_textures.iter() {
                    if let Some(texture) = texture_store.get(texture_handle) {
                        println!("Added texture");
                        texture_atlas_builder.add_texture(*texture_handle, &texture);
                    }
                    
                }
                let texture_atlas = texture_atlas_builder.finish(texture_store).unwrap();
                let atlas_handle = texture_atlas_store.add(texture_atlas);
                hack_remove.push(*tilemap_handle);
                self.loaded_maps.insert(*tilemap_handle, atlas_handle);
                println!("inserted");
            }
        }
        for handle in hack_remove.iter() {
            self.staged_maps.remove(handle);
        }
    }
}

fn tilemap_load_system(
    mut commands: Commands,
    asset_events: Res<Events<AssetEvent<TileMap>>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlas_store: ResMut<Assets<TextureAtlas>>,
    mut tilemap_spawner: ResMut<TileMapSpawner>,
    asset_server: Res<AssetServer>,
    tilemap_store: Res<Assets<TileMap>>,
    mut texture_store: ResMut<Assets<Texture>>,
) {
    for event in tilemap_spawner.tilemap_event_reader.iter(&asset_events) {
        if let AssetEvent::Created { handle } | AssetEvent::Modified { handle } = event {
            if let Some(tilemap) = tilemap_spawner.loaded_maps.get(handle) {
                println!("Alredy Loaded {:?}", handle);
            } else {
                println!("Stageing map");
                tilemap_spawner.stage_map(*handle, &tilemap_store, &asset_server);
            }
        }
    }

    tilemap_spawner.poll_staged_maps(&asset_server, &mut texture_store, &mut texture_atlas_store);

    // this introduces hard to find bugs
    let loaded_maps = std::mem::replace(&mut tilemap_spawner.loaded_maps, HashMap::new());

    let mut hack = Vec::new();
    //rest of queue
    tilemap_spawner
        .to_be_spawned
        .iter()
        .filter(|&handle| {
            if loaded_maps.contains_key(handle) {
                println!("LOADED");
                true
            } else {
                println!("NOT LOADED");
                hack.push(*handle);
                false
            }
        })
        .map(|handle| loaded_maps.get(&handle).unwrap())
        .for_each(|handle| {
            let atlas = texture_atlas_store.get(handle).unwrap();
            println!("spawned");
            commands.spawn(SpriteComponents {
                material: materials.add(atlas.texture.into()),
                translation: Vec3::new(0.0, 0., 0.0).into(),
                ..Default::default()
            });
        });
    tilemap_spawner.to_be_spawned = hack;
    tilemap_spawner.loaded_maps = loaded_maps;
}

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<TileMap>()
            .init_resource::<TileMapSpawner>()
            .add_asset_loader::<TileMap, TileMapLoader>()
            .add_system(tilemap_load_system.system());
    }
}
