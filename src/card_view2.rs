use std::sync::mpsc::Sender;

use notan::{
    app::Graphics,
    draw::*,
    math::{Affine2, Vec2},
    prelude::{Color, Texture},
    Event,
};
use slotmap::DefaultKey;

use crate::{
    animators::{AngleAnimator, TranslationAnimator},
    card::{Card, CardId, SelectState},
    card_update::CardUpdate,
    game::PlayerAction,
    texture_loader::{TextureLoader, CARD_TEX_SCALE},
    transform::Transform,
    view_geom::{CARD_SIZE, CARD_SIZE_HOVER},
    view_trait::ViewTrait,
};

pub struct CardView2 {
    pub id: DefaultKey,
    pub visible: bool,
    pub z_order: usize,
    pub transform: Transform,

    tex_size_multiplier: f32,
    face_tex_id: String,
    face_tex: Option<Texture>,
    pub face_up: bool,
    back_tex_id: String,
    back_tex: Option<Texture>,
    color: Color,

    pt_text: String,

    pub update: CardUpdate,
    pub select_state: SelectState,
    pub mouse_over: bool,

    // Animation
    pub translation_animator: Option<TranslationAnimator>,
    pub angle_animator: Option<AngleAnimator>,

    pub sender: Option<Sender<PlayerAction>>,
    pub mouse_up_message: Option<PlayerAction>,
}

impl CardView2 {
    pub fn new(
        card_id: CardId,
        points: i16,
        face_tex_id: &str,
        back_tex_id: &str,
        tex_size_multiplier: f32,
        sender: Option<Sender<PlayerAction>>,
    ) -> Self {
        let transform = Transform::default();

        let mut view = Self {
            id: card_id,
            visible: true,
            z_order: 0,
            transform,

            tex_size_multiplier,
            face_tex_id: face_tex_id.to_string(),
            face_tex: None,
            face_up: false,
            back_tex_id: back_tex_id.to_string(),
            back_tex: None,
            color: Color::WHITE,

            pt_text: String::new(),

            update: CardUpdate::default(),
            select_state: SelectState::Unselectable,
            mouse_over: false,

            translation_animator: None,
            angle_animator: None,

            sender,
            mouse_up_message: None,
        };
        view.update_pt_text(points);
        view
    }

    pub fn update_pt_text(&mut self, points: i16) {
        self.pt_text = match points {
            0 => String::new(),
            _ => format!("{} pts", points),
        };
    }

    pub fn animate_to(&mut self, location: CardUpdate, trans_vel: f32, angle_vel: f32) {
        // Create translation animator if needed.
        let end_pt = location.translation();
        if !self.transform.translation().abs_diff_eq(end_pt, 0.1) {
            let animator =
                TranslationAnimator::new(self.transform.translation(), end_pt, trans_vel);
            self.translation_animator = Some(animator);
        }

        // Create angle animator if needed.
        let end_angle = location.angle();
        if (self.transform.angle() - end_angle).abs() > 0.01 {
            let animator = AngleAnimator::new(self.transform.angle(), end_angle, angle_vel);
            self.angle_animator = Some(animator);
        }

        self.z_order = location.z_order();
    }
}

impl ViewTrait for CardView2 {
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

        match self.select_state {
            SelectState::Selectable => {}
            SelectState::Unselectable => return false,
            SelectState::Dimmed => return false,
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

        contains
    }

    fn send_message_for_event(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseUp { .. } => {
                if let Some(sender) = &self.sender {
                    if let Some(message) = &self.mouse_up_message {
                        sender.send(*message).expect("Message send error.");
                        return true;
                    }
                }
                println!("Card {:?}: mouse up", self.id);
            }
            _ => {}
        }
        false
    }

    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if self.face_tex.is_none() && !self.face_tex_id.is_empty() {
            if let Some(texture) = crate::TEX_LOADER.lock().unwrap().get_tex(&self.face_tex_id) {
                self.face_tex = Some(texture.clone());
                let size: Vec2 = texture.size().into();
                self.transform.set_size(size * self.tex_size_multiplier);
            } else {
                return;
            }
        }

        if self.back_tex.is_none() && !self.back_tex_id.is_empty(){
            if let Some(texture) = crate::TEX_LOADER.lock().unwrap().get_tex(&self.back_tex_id) {
                self.back_tex = Some(texture.clone());
            } else {
                return;
            }
        }

        if !self.visible {
            return;
        }

        let mut color = self.color;

        match self.select_state {
            SelectState::Selectable => match self.mouse_over {
                true => self.transform.set_size(CARD_SIZE_HOVER),
                false => self.transform.set_size(CARD_SIZE),
            },
            SelectState::Unselectable => {
                self.transform.set_size(CARD_SIZE);
            }
            SelectState::Dimmed => {
                self.transform.set_size(CARD_SIZE);
                color = crate::view::LIGHT_GRAY;
            }
        }

        let (size_x, size_y) = self.transform.size().into();

        let tex = match self.face_up {
            true => &self.face_tex,
            false => &self.back_tex,
        };

        if let Some(tex) = tex {
            draw.image(tex)
                .transform(self.transform.mat3_with_parent(parent_affine))
                .size(size_x, size_y)
                .color(color);
        }

        let font = crate::FONT.lock().unwrap().expect("Font is None");

        if self.face_up && !self.pt_text.is_empty() {
            draw.text(&font, &self.pt_text)
                .position(6.0, 104.0)
                .transform(self.transform.mat3_with_parent(parent_affine))
                .size(10.0 * 2.0)
                .h_align_left()
                .v_align_middle()
                .color(Color::BLACK);
        }
    }
}
