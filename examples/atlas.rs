//! An example showing how the [`TileAtlasBuilder`] can be used
//!
//! The end result should show the tiles in the following order:
//! 1. Grass
//! 2. Dirt
//! 3. Wall
//! 4. Dirt
//! 5. Grass
//!
//! This example uses a state system to manage the loading of the various tiles
//! in order to be easier to follow, but other methods may be used of course

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_tile_atlas::TileAtlasBuilder;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.init_resource::<TileHandles>()
		.init_resource::<MyAtlas>()
		.add_state(AppState::LoadTileset)
		.add_system_set(SystemSet::on_enter(AppState::LoadTileset).with_system(load_tiles))
		.add_system_set(SystemSet::on_update(AppState::CreateTileset).with_system(create_atlas))
		.add_system_set(SystemSet::on_enter(AppState::DisplayTileset).with_system(display_atlas))
		.run();
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppState {
	LoadTileset,
	CreateTileset,
	DisplayTileset,
}

/// The resultant atlas (or `None` if not yet generated)
#[derive(Default)]
struct MyAtlas(Option<TextureAtlas>);

/// Contains the list of handles we need to be loaded before we can build the atlas
#[derive(Default)]
struct TileHandles(Vec<HandleUntyped>);

fn load_tiles(
	mut handles: ResMut<TileHandles>,
	mut state: ResMut<State<AppState>>,
	asset_server: Res<AssetServer>,
) {
	let tiles = vec![
		asset_server.load_untyped("tiles/grass.png"),
		asset_server.load_untyped("tiles/dirt.png"),
		asset_server.load_untyped("tiles/wall.png"),
		asset_server.load_untyped("tiles/dirt.png"),
		asset_server.load_untyped("tiles/grass.png"),
	];
	handles.0 = tiles;
	state.overwrite_set(AppState::CreateTileset).unwrap();
}

fn create_atlas(
	mut atlas: ResMut<MyAtlas>,
	mut textures: ResMut<Assets<Image>>,
	mut state: ResMut<State<AppState>>,
	handles: Res<TileHandles>,
	asset_server: Res<AssetServer>,
) {
	let ids = handles.0.iter().map(|h| h.id);
	if LoadState::Loaded != asset_server.get_group_load_state(ids) {
		// All textures must first be loaded
		return;
	}

	let mut builder = TileAtlasBuilder::default();
	let mut is_first = true;

	for handle in &handles.0 {
		if let Some(texture) = textures.get(&handle.typed_weak()) {
			if let Ok(index) = builder.add_texture(handle.clone().typed::<Image>(), texture) {
				println!("Added texture at index: {}", index);
			}
		}

		if is_first {
			is_first = false;
			if let Some(size) = builder.get_tile_size() {
				println!("Detected tile size: {}", size);
			}
		}
	}

	atlas.0 = builder.finish(&mut textures).ok();

	state.set(AppState::DisplayTileset).unwrap();
}

fn display_atlas(
	mut atlas_res: ResMut<MyAtlas>,
	mut commands: Commands,
	mut atlases: ResMut<Assets<TextureAtlas>>,
) {
	commands.spawn_bundle(Camera2dBundle::default());

	let atlas = atlas_res.0.take().unwrap();
	let handle = atlas.texture.clone();
	let atlas_handle = atlases.add(atlas);

	// Display the third tile (Wall)
	commands.spawn_bundle(SpriteSheetBundle {
		transform: Transform {
			translation: Vec3::new(0.0, 48.0, 0.0),
			..Default::default()
		},
		sprite: TextureAtlasSprite::new(2),
		texture_atlas: atlas_handle,
		..Default::default()
	});

	// Display the whole tileset
	commands.spawn_bundle(SpriteBundle {
		texture: handle,
		..Default::default()
	});

	atlas_res.0 = None;
}
