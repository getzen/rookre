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

pub struct BidSelector {
    pub visible: bool,
    pub transform: Transform,
    pub z_order: usize,
    pub texture: Texture,

    pub accept_button: ImageButton<PlayerAction>,
    pub pass_button: ImageButton<PlayerAction>,
}

impl BidSelector {
    pub fn new(
        position: Vec2,
        texture: Texture,
        gfx: &mut Graphics,
        sender: Sender<PlayerAction>,
    ) -> Self {
        let trans = Transform::from_pos_tex_scale_centered(
            position,
            &texture,
            crate::view_fn::TEX_SCALE,
            true,
        );

        let accept_button = BidSelector::create_accept_button(gfx, sender.clone());
        let pass_button = BidSelector::create_pass_button(gfx, sender.clone());

        Self {
            visible: false,
            transform: trans,
            z_order: 0,
            texture,
            accept_button,
            pass_button,
        }
    }

    fn create_accept_button(
        gfx: &mut Graphics,
        sender: Sender<PlayerAction>,
    ) -> ImageButton<PlayerAction> {
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
        let mut button = ImageButton::new(
            pos,
            enabled,
            Some(mouse_over),
            None,
            1.0,
            String::new(),
            Some(sender),
        );
        button.mouse_up_message = None; //
        button
    }

    fn create_pass_button(
        gfx: &mut Graphics,
        sender: Sender<PlayerAction>,
    ) -> ImageButton<PlayerAction> {
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
        let mut button = ImageButton::new(
            pos,
            enabled,
            Some(mouse_over),
            None,
            1.0,
            String::new(),
            Some(sender),
        );
        button.mouse_up_message = None; //
        button
    }
}

impl ViewTrait for BidSelector {
    fn update(&mut self, _app: &mut notan::app::App, _time_delta: f32) {}

    fn mouse_event_handled(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
    ) -> bool {
        if !self.visible {
            return false;
        }

        let affine = *parent_affine * self.transform.affine2();

        if self
            .accept_button
            .mouse_event_handled(event, screen_pt, &affine)
        {
            return true;
        }

        if self
            .pass_button
            .mouse_event_handled(event, screen_pt, &affine)
        {
            return true;
        }

        false
    }

    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if !self.visible {
            return;
        }

        let (size_x, size_y) = self.transform.size().into();
        draw.image(&self.texture)
            .transform(self.transform.mat3_with_parent(parent_affine))
            .size(size_x, size_y);

        let affine = *parent_affine * self.transform.affine2();
        self.accept_button.draw(draw, &affine);
        self.pass_button.draw(draw, &affine);
    }
}
