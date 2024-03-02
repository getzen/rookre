use std::sync::mpsc::Sender;

use notan::draw::*;
use notan::math::vec2;
use notan::math::Affine2;
use notan::math::Vec2;
use notan::prelude::*;

use crate::game::PlayerAction;
use crate::image_button::ImageButton;
use crate::transform::Transform;
use crate::view_trait::ViewTrait;


pub struct BidSelector1 {
    pub visible: bool,
    pub transform: Transform,
    pub z_order: usize,
    pub texture: Texture,

    pub accept_button: ImageButton<PlayerAction>,
    pub pass_button: ImageButton<PlayerAction>,
}

impl BidSelector1 {
    pub fn new(
        position: Vec2,
        texture: Texture,
        texture_scale: f32,
        gfx: &mut Graphics,
        sender: Sender<PlayerAction>,
    ) -> Self {

        let mut trans =
            Transform::from_translation_angle_full_size(position, 0.0, texture.size().into());
        trans.set_scale(vec2(texture_scale, texture_scale));
        trans.drawn_size = vec2(texture.size().0 * 0.5, texture.size().1 * 0.5);

        let accept_button = BidSelector1::create_accept_button(gfx, sender.clone());
        let pass_button = BidSelector1::create_pass_button(gfx, sender.clone());

        Self {
            visible: true,
            transform: trans,
            z_order: 0,
            texture,
            accept_button,
            pass_button,
        }
    }

    fn create_accept_button(gfx: &mut Graphics, sender: Sender<PlayerAction>) -> ImageButton<PlayerAction> {
        let enabled = gfx
            .create_texture()
            .from_image(include_bytes!("assets/accept_enabled.png"))
            .build()
            .unwrap();

        let mouse_over = gfx
            .create_texture()
            .from_image(include_bytes!("assets/accept_mouse_over.png"))
            .build()
            .unwrap();

        let pos = vec2(104., 55.);
        let mut button = ImageButton::new(pos, enabled, Some(mouse_over), None, 1.0, String::new(), Some(sender));
        button.transform.set_offset(Vec2::ZERO);
        button.mouse_up_message = None; //
        button
    }

    fn create_pass_button(gfx: &mut Graphics, sender: Sender<PlayerAction>) -> ImageButton<PlayerAction> {
        let enabled = gfx
            .create_texture()
            .from_image(include_bytes!("assets/pass_enabled.png"))
            .build()
            .unwrap();

        let mouse_over = gfx
            .create_texture()
            .from_image(include_bytes!("assets/pass_mouse_over.png"))
            .build()
            .unwrap();

        let pos = vec2(214., 55.);
        let mut button = ImageButton::new(pos, enabled, Some(mouse_over), None, 1.0, String::new(), Some(sender));
        //button.transform.set_offset(Vec2::ZERO);
        button.mouse_up_message = None; //
        button
    }
}

impl ViewTrait for BidSelector1 {
    fn update(&mut self, _app: &mut notan::app::App, _time_delta: f32) {}

    fn mouse_event_handled(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: Option<&notan::math::Affine2>,
    ) -> bool {
        if !self.visible {
            return false;
        }

        let parent_affine = Some(*parent_affine.unwrap() * self.transform.affine2());


        if self.accept_button.mouse_event_handled(event, screen_pt, parent_affine.as_ref()) {
            return true;
        }

        if self.pass_button.mouse_event_handled(event, screen_pt, parent_affine.as_ref()) {
            return true;
        }
        
        false
    }

    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2, gfx: &mut Graphics) {
        if !self.visible {
            return;
        }

        draw.image(&self.texture)
            .transform(self.transform.mat3())
            .size(self.transform.drawn_size.x, self.transform.drawn_size.y);

        let affine = *parent_affine * self.transform.affine2();

        self.accept_button.draw(draw, &affine, gfx);
        self.pass_button.draw(draw, &affine, gfx);
       
    }
}
