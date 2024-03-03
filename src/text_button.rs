use std::sync::mpsc::Sender;

use notan::draw::*;
use notan::math::Affine2;
use notan::math::Vec2;
use notan::prelude::*;

//use crate::PIXEL_RATIO;

use crate::transform::Transform;
use crate::view_trait::ViewTrait;

#[derive(PartialEq)]
pub enum ButtonState {
    Enabled,
    Disabled,
    MouseOver,
    MouseDown,
}

// In this app, just the fill_color changes with the state. Other fields,
// such as stroke_color and stroke_width could be associated with the state.
impl ButtonState {
    fn fill_color(&self) -> Color {
        match *self {
            ButtonState::Enabled => crate::view_fn::LIGHT_GRAY,
            ButtonState::Disabled => Color::GRAY,
            ButtonState::MouseOver => crate::view_fn::LIGHT_GRAY,
            ButtonState::MouseDown => Color::WHITE,
        }
    }
}

pub struct TextButton<T> {
    pub visible: bool,
    pub transform: Transform,
    //size: Vec2,
    pub state: ButtonState,
    pub stroke_color: Color,
    pub stroke_width: f32,

    pub text: String,
    pub font: Font,
    pub font_size: f32,
    pub font_color: Color,

    pub sender: Option<Sender<T>>,
    pub mouse_up_message: Option<T>,

    pixel_ratio: f32,
}

impl<T> TextButton<T> {
    pub fn new(text: String, size: Vec2, position: Vec2, sender: Option<Sender<T>>) -> Self {
        let transform = Transform::from_translation_size_centered(position, size, true);

        Self {
            visible: true,
            transform,
            //size,
            state: ButtonState::Enabled,
            stroke_color: Color::BLACK,
            stroke_width: 1.0,
            text,
            font: crate::FONT.lock().unwrap().expect("Font is None"), // see main.rs setup()
            font_size: 16.0,
            font_color: Color::BLACK,
            sender,
            mouse_up_message: None,
            pixel_ratio: *crate::PIXEL_RATIO.lock().unwrap(),
        }
    }
}

impl<T: Copy> ViewTrait for TextButton<T> {
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
        if self.state == ButtonState::Disabled {
            return false;
        }

        if self
            .transform
            .contains_screen_point(screen_pt, parent_affine)
        {
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
            return true;
        } else {
            // Mouse is not over button, so set plain Enabled state.
            self.state = ButtonState::Enabled;
        }
        false
    }

    fn draw(&mut self, draw: &mut Draw, _parent_affine: &Affine2) {
        if !self.visible {
            return;
        }

        // No need to center the rect with 'position' since the transform will do it.
        let (size_x, size_y) = self.transform.size().into();
        draw.rect((0.0, 0.0), (size_x, size_y))
            .transform(self.transform.mat3())
            .corner_radius(6.0)
            .fill()
            .fill_color(self.state.fill_color())
            .stroke(self.stroke_width)
            .stroke_color(self.stroke_color);

        draw.text(&self.font, &self.text)
            // Need to move the position to the center, then use draw methods to center from there.
            .position(size_x * 0.5, size_y * 0.5)
            .transform(self.transform.mat3())
            .size(self.font_size * self.pixel_ratio)
            .h_align_center()
            .v_align_middle()
            .color(self.font_color);
    }
}
