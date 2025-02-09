use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use bevy::log::LogPlugin;
use bevy_asset::LoadedUntypedAsset;
use std::default::Default;

#[derive(Clone, States, Default, Eq, PartialEq, Hash, Debug)]
enum GameState {
    #[default]
    Loading,
    Running,
}

// PRO TIP: _technically_ since Sprite's are just created using strings, an easier way to load them from an external
// file is just:
//static TITLE_TEXT: &str = include_str!("assets/demo/title.txt");
// then just:
//sprites.add(Sprite::new(TITLE_TEXT));
// and boom, you have yourself a sprite in the asset system.
// That's nice and easy - don't have to worry about async, don't need to distribute files alongside your exe.
// But then you can't take advantage of hot reloading, and plus it only works for sprites. StyleMaps have to go through
// the AssetServer if you want to load them from an external file.

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Assets example");

    App::new()
        .insert_resource(settings)
        .add_plugins(bevy_app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(50),
        ))
        .add_plugins(
            DefaultPlugins
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(1),
                })
                .set(LogPlugin {
                    filter: "off".into(),
                    level: bevy::log::Level::ERROR,
                }),
        )
        .add_plugins(CrosstermPlugin)
        .add_state::<GameState>()
        .add_systems(OnEnter(GameState::Loading), default_settings)
        .add_systems(OnEnter(GameState::Loading), load_assets)
        .add_systems(Update, check_for_loaded)
        .add_systems(OnEnter(GameState::Running), create_entities)
        .run();
}

static ASSETS: &[&str] = &["demo/title.txt", "demo/title.stylemap"];

#[derive(Resource)]
struct CrosstermAssets(Vec<Handle<LoadedUntypedAsset>>);

fn default_settings(mut cursor: ResMut<Cursor>) {
    cursor.hidden = true;
}

// This is a simple system that loads assets from the filesystem
fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the assets we want
    let mut handles = Vec::new();
    for asset in ASSETS {
        handles.push(asset_server.load_untyped(*asset));
    }

    commands.insert_resource(CrosstermAssets(handles));
}

// This function exists solely because bevy's asset loading is async.
// We need to wait until all assets are loaded before we do anything with them.
fn check_for_loaded(
    asset_server: Res<AssetServer>,
    handles: Res<CrosstermAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut all_done = true;
    for handle in handles.0.iter() {
        let data = asset_server.load_state(handle);

        match data {
            bevy::asset::LoadState::NotLoaded | bevy::asset::LoadState::Loading => {
                all_done = false;
                break;
            }
            bevy::asset::LoadState::Loaded => {}
            bevy::asset::LoadState::Failed => {
                panic!("This is an example and should not fail")
            }
        }
    }

    if all_done {
        next_state.set(GameState::Running);
    }
}

// Now that we're sure the assets are loaded, spawn a new sprite into the world
fn create_entities(
    mut commands: Commands,
    window: Query<&CrosstermWindow>,
    asset_server: Res<AssetServer>,
    mut sprites: ResMut<Assets<Sprite>>,
    mut stylemaps: ResMut<Assets<StyleMap>>,
) {
    // I want to center the title, so i needed to wait until it was loaded before I could actually access
    // the underlying data to see how wide the sprite is and do the math
    let title_handle = asset_server.get_handle("demo/title.txt").unwrap();
    let title_sprite = sprites
        .get(&title_handle)
        .expect("We waited for asset loading");

    let window = window.single();
    let center_x = window.x_center() as i32 - title_sprite.x_center() as i32;
    let center_y = window.y_center() as i32 - title_sprite.y_center() as i32;

    commands.spawn(SpriteBundle {
        sprite: title_handle.clone(),
        position: Position::with_xy(center_x, center_y),
        stylemap: asset_server.get_handle("demo/title.stylemap").unwrap(),
        ..Default::default()
    });

    let text = Sprite::new(
        "You may freely change demo/title.txt and demo/title.stylemap,\n\
    bevy_crossterm will automatically reload changed assets and redraw affected sprites.",
    );

    let center_x = window.x_center() as i32 - text.x_center() as i32;
    let center_y = window.y_center() as i32 - text.y_center() as i32;

    let text = sprites.add(text);
    let color = stylemaps.add(StyleMap::default());

    // Spawn two sprites into the world
    commands.spawn(SpriteBundle {
        sprite: text,
        position: Position::with_xy(center_x, center_y + 6),
        stylemap: color.clone(),
        ..Default::default()
    });
}
