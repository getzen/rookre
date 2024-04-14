mod animators;
mod bid_selector;
mod bot;
mod bot_monte;
mod bot_random;
mod card;
mod card_update;
// mod card_view;
mod card_view2;
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
use texture_loader::TextureLoader;

// Globals
use std::sync::Mutex;

/// This isn't really dots per inch. It's actually physical pixels per logical pixel.
static PIXEL_RATIO: Mutex<f32> = Mutex::new(0.0);

static FONT: Mutex<Option<notan::draw::Font>> = Mutex::new(None);

// Use once_cell to init lazily.
static TEX_LOADER: Lazy<Mutex<TextureLoader>> = Lazy::new(|| Mutex::new(TextureLoader::new()));

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
    let path = std::env::current_dir().expect("whoops");
    println!("Current directory: {}", path.display());

    Controller::new(assets, gfx)
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
