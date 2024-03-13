use notan::{
    app::Graphics,
    draw::{Draw, DrawImages, DrawTransform},
    math::{vec2, Affine2, Vec2},
    prelude::{Color, Texture},
    Event,
};
use slotmap::DefaultKey;

use crate::{
    animators::{AngleAnimator, TranslationAnimator}, card::Card, card_location::CardLocation, texture_loader::{ViewFn, CARD_TEX_SCALE}, transform::Transform, view_geom::CARD_SIZE, view_trait::ViewTrait
};

pub struct CardView {
    pub id: DefaultKey,
    pub visible: bool,
    pub z_order: usize,
    pub transform: Transform,
    pub face_tex: Texture,

    pub face_down: bool,
    pub back_tex: Texture,

    pub mouse_over: bool,
    pub mouse_over_tex: Texture,

    pub color: Color,

    pub selectable: bool,

    pub location: CardLocation,

    // Animation
    pub translation_animator: Option<TranslationAnimator>,
    pub angle_animator: Option<AngleAnimator>,
}

impl CardView {
    pub fn new(card: &Card, gfx: &mut Graphics) -> Self {
        let face_tex = ViewFn::load_card_texture(gfx, card);

        let transform =
            Transform::from_pos_tex_scale_centered(Vec2::ZERO, &face_tex, CARD_TEX_SCALE, true);

        let back_tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/cards/back.png"))
            .build()
            .unwrap();

        let mouse_over_tex = gfx
            .create_texture()
            .from_image(include_bytes!("assets/cards/outline_solid.png"))
            .build()
            .unwrap();

        Self {
            id: card.id,
            visible: true,
            z_order: 0,
            transform,
            face_tex,

            face_down: false,
            back_tex,

            mouse_over: false,
            mouse_over_tex,

            color: Color::WHITE,
            selectable: false,

            location: CardLocation::default(),
            translation_animator: None,
            angle_animator: None,
        }
    }
}

impl ViewTrait for CardView {
    fn update(&mut self, time_delta: f32, _app: &mut notan::app::App) {
        if let Some(animator) = &mut self.translation_animator {
            self.transform.set_translation(animator.update(time_delta));
            if animator.completed {
                self.translation_animator = None;
            }
        }

        if let Some(animator) = &mut self.angle_animator {
            self.transform.set_angle(animator.update(time_delta));
            if animator.completed {
                self.angle_animator = None;
            }
        }
    }

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

        if self
            .transform
            .contains_screen_point(screen_pt, parent_affine)
        {
            self.mouse_over = send_msg;
            if send_msg {
                self.send_message_for_event(event);
            }
            contains = true;
        } else {
            self.mouse_over = false;
        }

        // Proof of concept
        self.location.mouse_over = self.mouse_over;
        // Create translation animator if needed.
        let new_trans = self.location.translation();
        if !self
            .transform
            .translation()
            .abs_diff_eq(new_trans, 0.1)
        {
            if self.translation_animator.is_none() {
                let animator = TranslationAnimator::new(
                    self.transform.translation(),
                    new_trans,
                    500.0, // velocity
                );
                self.translation_animator = Some(animator);

                // if self.mouse_over {
                //     let size = CARD_SIZE - vec2(0.0, 30.0)
                //     self.transform.set_size(size)
                // }
            }
        }

        contains
    }

    fn send_message_for_event(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseUp { .. } => {
                // if let Some(sender) = &self.sender {
                //     if let Some(message) = self.mouse_up_message {
                //         sender.send(message).expect("Message send error.");
                //         return true;
                //     }
                // }
                println!("Card {:?}: mouse up", self.id);
            }
            _ => {}
        }
        false
    }

    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if !self.visible {
            return;
        }

        let tex = if self.face_down {
            &self.back_tex
        } else {
            &self.face_tex
        };

        let (size_x, size_y) = self.transform.size().into();
        draw.image(tex)
            .transform(self.transform.mat3_with_parent(parent_affine))
            .size(size_x, size_y)
            .color(self.color);

        // if self.mouse_over {
        //     draw.image(&self.mouse_over_tex)
        //         .transform(self.transform.mat3_with_parent(parent_affine))
        //         .size(size_x, size_y);
        // }
    }
}
