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

    pub texture_enabled: Texture,
    pub texture_mouse_over: Option<Texture>,
    pub texture_disabled: Option<Texture>,

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
        position: Vec2,
        tex_enabled: Texture,
        tex_mouse_over: Option<Texture>,
        tex_disabled: Option<Texture>,
        text: String,
        sender: Option<Sender<T>>,
    ) -> Self {
        let trans = Transform::from_pos_tex_scale_centered(
            position,
            &tex_enabled,
            crate::texture_loader::TEX_SCALE,
            true,
        );

        Self {
            visible: true,
            transform: trans,
            z_order: 0,
            state: ButtonState::Enabled,

            texture_enabled: tex_enabled,
            texture_mouse_over: tex_mouse_over,
            texture_disabled: tex_disabled,

            text,
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
                                println!("sending!");
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
        if !self.visible {
            return;
        }

        let tex = match self.state {
            ButtonState::Enabled => &self.texture_enabled,
            ButtonState::Disabled => &self
                .texture_disabled
                .as_ref()
                .unwrap_or(&self.texture_enabled),
            ButtonState::MouseOver => &self
                .texture_mouse_over
                .as_ref()
                .unwrap_or(&self.texture_enabled),
            ButtonState::MouseDown => &self
                .texture_mouse_over
                .as_ref()
                .unwrap_or(&self.texture_enabled),
        };

        let (size_x, size_y) = self.transform.size().into();
        draw.image(tex)
            .transform(self.transform.mat3_with_parent(parent_affine))
            .size(size_x, size_y);

        // Need to move the position to the center, then use draw methods to center from there.
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
