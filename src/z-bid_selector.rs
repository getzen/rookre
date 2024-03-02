use std::sync::mpsc::Sender;

use notan::{
    app::{Graphics, Texture},
    draw::{DrawImages, DrawTransform},
    math::{vec2, Affine2, Vec2},
    Event,
};

use crate::{
    bid::Bid, game::PlayerAction, sprite::Sprite, transform::Transform, view_trait::ViewTrait,
};

pub struct BidSelector {
    pub visible: bool,
    texture: Texture,
    transform: Transform,
    buttons: Vec<Sprite>,
    sender: Sender<PlayerAction>,
}

impl BidSelector {
    pub fn new(position: Vec2, gfx: &mut Graphics, sender: Sender<PlayerAction>) -> Self {
        let texture = gfx
            .create_texture()
            .from_image(include_bytes!("assets/1x1.png"))
            .build()
            .unwrap();

        let mut transform =
            Transform::from_translation_angle_full_size(position, 0.0, vec2(1., 1.));
        transform.set_scale_from_size(vec2(100., 100.));

        let mut buttons = Vec::new();
        let id = slotmap::DefaultKey::default();
        //let pass = Sprite::new(id, texture, vec2(0., 0.), None);

        Self {
            visible: false,
            texture,
            transform,
            buttons,
            sender,
        }
    }
}

impl ViewTrait for BidSelector {
    fn draw(
        &mut self,
        draw: &mut notan::draw::Draw,
        parent_affine: &notan::math::Affine2,
        gfx: &mut notan::prelude::Graphics,
    ) {
        if !self.visible {
            return;
        }
        draw.image(&self.texture).transform(self.transform.mat3());

        let affine = *parent_affine * self.transform.affine2();
        for button in &mut self.buttons {
            button.draw(draw, &affine, gfx);
        }
    }

    fn mouse_event_handled(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: Option<&Affine2>,
    ) -> bool {
        if !self.visible {
            return false;
        }

        // Check children reverse to check on-top kids first.
        for child in self.buttons.iter_mut().rev() {
            if child.mouse_event_handled(event, screen_pt, parent_affine) {
                return true;
            }
        }

        // Now check self.
        if self
            .transform
            .contains_screen_point(screen_pt, parent_affine)
        {
            match event {
                Event::MouseUp { .. } => {
                    println!("bam");
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}
