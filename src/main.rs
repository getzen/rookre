mod animators;
mod bid;
mod bid_selector;
// mod bid_view;
mod bot;
mod bot_monte;
mod bot_random;
mod card;
mod controller;
mod game;
mod game_options;
mod image_button;
mod text_button;
// mod message_view;
mod player;
// mod score_card_view;
mod sprite;
// mod suit_button;
mod transform;
mod trick;
// mod trump_chooser_view;
// mod trump_view;
mod view;
mod view_fn;
mod view_trait;

use controller::Controller;
use notan::{
    draw::{CreateFont, Font},
    prelude::*,
};

// HashMaps cannot normally be initialized as static variables. The once_cell crate
// comes to the rescue:
// use once_cell::sync::Lazy;
// static TEXTURES: Lazy<Mutex<HashMap<usize, Texture>>> = Lazy::new(|| {
//let mut textures = HashMap::new();
//textures.insert(...);
//Mutex::new(textures)
// or simply:
// Mutex::new(HashMap::new())
// });

// Globals
use std::sync::Mutex;
static PIXEL_RATIO: Mutex<f32> = Mutex::new(0.0);
static FONT: Mutex<Option<notan::draw::Font>> = Mutex::new(None);

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

fn setup(gfx: &mut Graphics) -> Controller {
    // This isn't really dots per inch. It's actually physical pixels per logical pixel.
    *PIXEL_RATIO.lock().unwrap() = gfx.dpi() as f32;

    let font = gfx
        .create_font(include_bytes!("assets/Futura.ttc"))
        .unwrap();
    *FONT.lock().unwrap() = Some(font as Font);

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
