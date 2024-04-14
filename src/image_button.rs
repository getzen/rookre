use std::sync::mpsc::Sender;

use notan::draw::*;
use notan::math::Affine2;
use notan::math::Vec2;
use notan::prelude::*;

use crate::transform::Transform;
use crate::view_trait::ViewTrait;

#[derive(PartialEq)]
pub enum ButtonState {
    Enabled,
    Disabled,
    MouseOver,
    MouseDown,
}

pub struct ImageButton<T> {
    pub visible: bool,
    pub transform: Transform,
    pub z_order: usize,
    pub state: ButtonState,

    tex_size_multiplier: f32,

    tex_enabled_id: String,
    tex_enabled: Option<Texture>,

    tex_mouse_over_id: String,
    tex_mouse_over: Option<Texture>,

    tex_disabled_id: String,
    tex_disabled: Option<Texture>,

    pub text: String,
    pub font: Font,
    pub font_size: f32,
    pub font_color: Color,

    pub sender: Option<Sender<T>>,
    pub mouse_up_message: Option<T>,

    pixel_ratio: f32,
}

impl<T> ImageButton<T> {
    pub fn new(
        translation: Vec2,
        tex_enabled_id: &str,
        tex_mouse_over_id: &str,
        tex_disabled_id: &str,
        tex_size_multiplier: f32,
        text: &str,
        sender: Option<Sender<T>>,
    ) -> Self {
        let transform = Transform::from_translation(translation);

        Self {
            visible: true,
            transform,
            z_order: 0,
            state: ButtonState::Enabled,

            tex_size_multiplier,
            tex_enabled_id: tex_enabled_id.to_string(),
            tex_enabled: None,
            tex_mouse_over_id: tex_mouse_over_id.to_string(),
            tex_mouse_over: None,
            tex_disabled_id: tex_disabled_id.to_string(),
            tex_disabled: None,

            text: text.to_string(),
            font: crate::FONT.lock().unwrap().expect("Font is None"), // see main.rs setup()
            font_size: 12.0,
            font_color: Color::BLACK,

            sender,
            mouse_up_message: None,

            pixel_ratio: *crate::PIXEL_RATIO.lock().unwrap(),
        }
    }
}

impl<T: Copy> ViewTrait for ImageButton<T> {
    fn handle_mouse_event(
        &mut self,
        event: &Event,
        screen_pt: Vec2,
        parent_affine: &Affine2,
        mut _send_msg: bool,
    ) -> bool {
        if !self.visible {
            return false;
        }
        if self.state == ButtonState::Disabled {
            return false;
        }

        let mut contains = false;

        if self
            .transform
            .contains_screen_point(screen_pt, parent_affine)
        {
            contains = true;
            match event {
                Event::MouseDown { button, .. } => match button {
                    MouseButton::Left => self.state = ButtonState::MouseDown,
                    _ => {}
                },
                Event::MouseUp { button, .. } => match button {
                    MouseButton::Left => {
                        if let Some(sender) = &self.sender {
                            if let Some(message) = &self.mouse_up_message {
                                sender.send(*message).expect("Message send error.");
                                return true;
                            }
                        }
                    }
                    _ => {}
                },
                _ => self.state = ButtonState::MouseOver,
            }
        } else {
            // Mouse is not over button, so set plain Enabled state.
            self.state = ButtonState::Enabled;
        }
        contains
    }

    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if self.tex_enabled.is_none() && !self.tex_enabled_id.is_empty() {
            if let Some(texture) = crate::TEX_LOADER
                .lock()
                .unwrap()
                .get_tex(&self.tex_enabled_id)
            {
                self.tex_enabled = Some(texture.clone());
                let size: Vec2 = texture.size().into();
                self.transform.set_size(size * self.tex_size_multiplier);
            } else {
                return;
            }
        }

        if self.tex_mouse_over.is_none() && !self.tex_mouse_over_id.is_empty() {
            if let Some(texture) = crate::TEX_LOADER
                .lock()
                .unwrap()
                .get_tex(&self.tex_mouse_over_id)
            {
                self.tex_mouse_over = Some(texture.clone());
            } else {
                return;
            }
        }

        if self.tex_disabled.is_none() && !self.tex_disabled_id.is_empty() {
            if let Some(texture) = crate::TEX_LOADER
                .lock()
                .unwrap()
                .get_tex(&self.tex_disabled_id)
            {
                self.tex_disabled = Some(texture.clone());
            } else {
                return;
            }
        }
        if !self.visible {
            return;
        }

        let tex = match self.state {
            ButtonState::Enabled => &self.tex_enabled,
            ButtonState::Disabled => &self.tex_disabled,
            ButtonState::MouseOver => &self.tex_mouse_over,
            ButtonState::MouseDown => &self.tex_mouse_over,
        };

        if let Some(tex) = tex {
            let (size_x, size_y) = self.transform.size().into();
            draw.image(tex)
                .transform(self.transform.mat3_with_parent(parent_affine))
                .size(size_x, size_y);

            // Need to move the position to the center, then use draw methods to center from there.
            if !self.text.is_empty() {
                let pos = self.transform.size() * 0.5;
                draw.text(&self.font, &self.text)
                    .position(pos.x, pos.y)
                    .transform(self.transform.mat3_with_parent(parent_affine))
                    .size(self.font_size * self.pixel_ratio)
                    .h_align_center()
                    .v_align_middle()
                    .color(self.font_color);
            }
        }
    }
}
