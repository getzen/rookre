use std::sync::mpsc::Sender;

use notan::draw::*;
use notan::math::vec2;
use notan::math::Affine2;
use notan::math::Vec2;
use notan::prelude::*;

use crate::game::PlayerAction;
use crate::image_button::ButtonState;
use crate::image_button::ImageButton;
use crate::texture_loader::ViewFn;
use crate::transform::Transform;
use crate::view_geom::ViewGeom;
use crate::view_trait::ViewTrait;

pub struct DiscardPanel {
    pub visible: bool,
    pub transform: Transform,
    pub z_order: usize,
    pub texture: Texture,
}

impl DiscardPanel {
    pub fn new(gfx: &mut Graphics) -> Self {
        let texture = gfx
            .create_texture()
            .from_image(include_bytes!("assets/discard.png"))
            .build()
            .unwrap();

        let trans = Transform::from_pos_tex_scale_centered(
            ViewGeom::discard_panel_position(),
            &texture,
            crate::texture_loader::TEX_SCALE,
            true,
        );

        Self {
            visible: false,
            transform: trans,
            z_order: 0,
            texture,
        }
    }
}

impl ViewTrait for DiscardPanel {
    fn handle_mouse_event(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
        send_msg: bool,
    ) -> bool {
        if !self.visible {
            return false;
        }

        let mut contains = false;

        let affine = *parent_affine * self.transform.affine2();

        // if self
        //     .done_button
        //     .handle_mouse_event(event, screen_pt, &affine, send_msg)
        // {
        //     contains = true;
        // }

        contains
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

        //self.done_button.draw(draw, &affine);
    }
}
