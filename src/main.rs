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
use std::{collections::HashMap, sync::Mutex};
static PIXEL_RATIO: Mutex<f32> = Mutex::new(0.0);
static FONT: Mutex<Option<notan::draw::Font>> = Mutex::new(None);

// Use once_cell to init textures HashMap.
static TEXTURES: Lazy<Mutex<HashMap<String, Texture>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

#[notan_main]
fn main() -> Result<(), String> {
    // Check the documentation for more options
    let window_config = WindowConfig::default()
        .set_title("Rookre")
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

fn load_textures(gfx: &mut Graphics) {
    let tex = gfx.create_texture().from_image(include_bytes!("assets/club.png")).build().unwrap();
    TEXTURES.lock().unwrap().insert("club".to_string(), tex);
    let tex = gfx.create_texture().from_image(include_bytes!("assets/diamond.png")).build().unwrap();
    TEXTURES.lock().unwrap().insert("diamond".to_string(), tex);
    let tex = gfx.create_texture().from_image(include_bytes!("assets/heart.png")).build().unwrap();
    TEXTURES.lock().unwrap().insert("heart".to_string(), tex);
    let tex = gfx.create_texture().from_image(include_bytes!("assets/spade.png")).build().unwrap();
    TEXTURES.lock().unwrap().insert("spade".to_string(), tex);
}

fn setup(gfx: &mut Graphics) -> Controller {
    // This isn't really dots per inch. It's actually physical pixels per logical pixel.
    *PIXEL_RATIO.lock().unwrap() = gfx.dpi() as f32;

    let font = gfx
        .create_font(include_bytes!("assets/Futura.ttc"))
        .unwrap();
    *FONT.lock().unwrap() = Some(font as Font);

    load_textures(gfx);

    Controller::new(gfx)
}

fn event(controller: &mut Controller, event: Event) {
    controller.event(event);
}

fn update(app: &mut App, controller: &mut Controller) {
    controller.update(app);
}

fn draw(gfx: &mut Graphics, controller: &mut Controller) {
    controller.draw(gfx);
}
