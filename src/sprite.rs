use notan::{
    draw::{Draw, DrawImages, DrawTransform},
    math::{Affine2, Vec2},
    prelude::{Color, Graphics, Texture},
    Event,
};
use slotmap::DefaultKey;

use crate::{
    animators::{AngleAnimator, TranslationAnimator},
    transform::Transform,
    view_fn::CARD_SCALE,
    view_trait::ViewTrait,
};

pub struct Sprite {
    pub id: DefaultKey,
    pub visible: bool,
    pub z_order: usize,
    pub transform: Transform,
    pub texture: Texture,
    pub color: Color,
    pub children: Vec<Box<Sprite>>,
    /// Enabled for mouse events. Default is false.
    pub enabled: bool,
    /// True means face-down card. Default is false.
    pub use_alt_texture: bool,
    /// Card back
    pub alt_texture: Option<Texture>,

    // Animation
    pub translation_animator: Option<TranslationAnimator>,
    pub angle_animator: Option<AngleAnimator>,
}

impl Sprite {
    pub fn new(
        id: DefaultKey,
        texture: Texture,
        position: Vec2,
        alt_texture: Option<Texture>,
    ) -> Self {
        let transform =
            Transform::from_pos_tex_scale_centered(position, &texture, CARD_SCALE, true); //
        Self {
            id,
            visible: true,
            z_order: 0,
            transform,
            texture,
            color: Color::WHITE,
            children: Vec::new(),
            enabled: false,
            use_alt_texture: false,
            alt_texture,

            translation_animator: None,
            angle_animator: None,
        }
    }
}

impl ViewTrait for Sprite {
    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if !self.visible {
            return;
        }

        let tex = if self.use_alt_texture {
            self.alt_texture.as_ref().unwrap()
        } else {
            &self.texture
        };

        let (size_x, size_y) = self.transform.size().into();
        draw.image(tex)
            .transform(self.transform.mat3_with_parent(parent_affine))
            .size(size_x, size_y)
            .color(self.color);

        if !self.children.is_empty() {
            let affine = *parent_affine * self.transform.affine2();
            for child in &mut self.children {
                child.draw(draw, &affine);
            }
        }
    }

    fn mouse_event_handled(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
    ) -> bool {
        // If not visible, don't check this view or its children.
        if !self.visible {
            return false;
        }

        let mut hit = false;

        let affine = *parent_affine * self.transform.affine2();
        // Check children reverse to check on-top kids first.
        for child in self.children.iter_mut().rev() {
            hit |= child.mouse_event_handled(event, screen_pt, &affine);
        }

        // Now check self.
        if self
            .transform
            .contains_screen_point(screen_pt, parent_affine)
        {
            hit = true;
            match event {
                Event::MouseUp { .. } => {
                    println!("mouse up");

                    // if let Some(sender) = &self.message_sender {
                    //     if let Some(message) = self.mouse_up_message {
                    //         sender.send(message).expect("Message send error.");
                    //     }
                    // }

                    return true;
                }
                _ => {}
            }
        }
        hit
    }

    fn update(&mut self, _app: &mut notan::app::App, time_delta: f32) {
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
}
