mod animators;
mod bid_selector;
mod bot;
mod bot_monte;
mod bot_random;
mod card;
mod card_update;
mod card_view;
mod controller;
mod discard_panel;
mod game;
mod game_options;
mod image;
mod image2;
mod image_button;
mod text_button;
// mod message_view;
mod player;
// mod score_card_view;
// mod suit_button;
mod transform;
mod trick;
// mod trump_view;
mod texture_loader;
mod view;
mod view_geom;
mod view_trait;

use controller::Controller;
use notan::{
    draw::{CreateFont, Font},
    prelude::*,
};
use once_cell::sync::Lazy;

// Globals
use std::{collections::HashMap, sync::Mutex, time};

/// This isn't really dots per inch. It's actually physical pixels per logical pixel.
static PIXEL_RATIO: Mutex<f32> = Mutex::new(0.0);

static FONT: Mutex<Option<notan::draw::Font>> = Mutex::new(None);

// Use once_cell to init textures HashMap.
static ASSET_TEXTURES: Lazy<Mutex<HashMap<String, Asset<Texture>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static TEXTURES: Lazy<Mutex<HashMap<String, Texture>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[notan_main]
fn main() -> Result<(), String> {
    // Check the documentation for more options
    let window_config = WindowConfig::default()
        .set_title("Hundred Fifty")
        .set_position(10, 10)
        .set_size(800, 800)
        .set_vsync(true) // enable vsync
        .set_resizable(true) // window can be resized
        //.set_min_size(400, 400) // set a minimum window size
        // lazy_loop(true) means update only when there is an input event.
        // When using animators, this needs to be false.
        .set_lazy_loop(false)
        .set_high_dpi(true);

    //notan::init_with(setup)
    notan::init_with(setup)
        .add_config(window_config)
        .add_config(notan::draw::DrawConfig)
        .event(event)
        .update(update)
        .draw(draw)
        .build()
}

fn setup(assets: &mut Assets, gfx: &mut Graphics) -> Controller {
    *PIXEL_RATIO.lock().unwrap() = gfx.dpi() as f32;

    let font = gfx
        .create_font(include_bytes!("assets/Futura.ttc"))
        .unwrap();
    *FONT.lock().unwrap() = Some(font as Font);

    load_textures(assets, gfx);

    Controller::new(gfx)
}

fn event(controller: &mut Controller, event: Event) {
    controller.event(event);
}

fn update(app: &mut App, controller: &mut Controller) {
    let mut asset_binding = ASSET_TEXTURES.lock().unwrap();
    let ids: Vec<String> = asset_binding.keys().cloned().collect();

    for id in &ids {
        if let Some(item) = asset_binding.get(id) {
            if item.is_loaded() {
                println!("loaded!");
                let ass_tex = asset_binding.remove(id).unwrap();
                println!("removed!");
                let tex = ass_tex.lock().unwrap().clone();
                TEXTURES.lock().unwrap().insert(id.clone(), tex);
            }
        }
    }

    controller.update(app);
}

fn draw(gfx: &mut Graphics, controller: &mut Controller) {
    controller.draw(gfx);
}

fn asset_path(path: &str) -> String {
    let base = if cfg!(target_arch = "wasm32") { // browsers
        "./assets"
    } else {
        "./src/assets" // development: debug & release
    };
    format!("{base}/{path}")
}

fn load_textures(assets: &mut Assets, gfx: &mut Graphics) {
    // TESTING START --------------------
    let path = std::env::current_dir().expect("whoops");
    println!("The current directory is {}", path.display());

    let tex_name = "done_enabled.png".to_string();
    let path = asset_path(&tex_name);
    let ass_tex: Asset<Texture> = assets.load_asset(&path).unwrap();  
    ASSET_TEXTURES.lock().unwrap().insert(tex_name, ass_tex);  

    //println!("tex.is_loaded: {}", tex.is_loaded());

    // TESTING END ---------------------

    let tex = gfx
        .create_texture()
        .from_image(include_bytes!("assets/club.png"))
        .build()
        .unwrap();
    TEXTURES.lock().unwrap().insert("club".to_string(), tex);
    let tex = gfx
        .create_texture()
        .from_image(include_bytes!("assets/diamond.png"))
        .build()
        .unwrap();
    TEXTURES.lock().unwrap().insert("diamond".to_string(), tex);
    let tex = gfx
        .create_texture()
        .from_image(include_bytes!("assets/heart.png"))
        .build()
        .unwrap();
    TEXTURES.lock().unwrap().insert("heart".to_string(), tex);
    let tex = gfx
        .create_texture()
        .from_image(include_bytes!("assets/spade.png"))
        .build()
        .unwrap();
    TEXTURES.lock().unwrap().insert("spade".to_string(), tex);
}