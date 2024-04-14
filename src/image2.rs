// Draws an image. No updating or event handling functions.

use notan::{
    draw::{Draw, DrawImages, DrawTransform},
    math::{Affine2, Vec2},
    prelude::Texture,
};

use crate::{transform::Transform, view_trait::ViewTrait, TEX_LOADER};

#[derive(Clone)]
pub struct Image2 {
    pub visible: bool,
    pub z_order: usize,
    pub transform: Transform,
    texture_id: String,
    texture: Option<Texture>,
    texture_size_multiplier: f32,
}

impl Image2 {
    /// The size of the image is the texture size mutiplied by tex_size_mult.
    pub fn new(tex_id: &str, translation: Vec2, tex_size_mult: f32) -> Self {
        let transform = Transform::from_translation(translation);
        Self {
            visible: true,
            z_order: 0,
            transform,
            texture_id: tex_id.to_string(),
            texture: None,
            texture_size_multiplier: tex_size_mult,
        }
    }

    pub fn set_texture_id(&mut self, id: String) {
        if self.texture_id != id {
            self.texture_id = id;
            self.texture = None;
        }
    }
}

impl ViewTrait for Image2 {
    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if self.texture.is_none() {
            if let Some(texture) = TEX_LOADER.lock().unwrap().get_tex(&self.texture_id) {
                self.texture = Some(texture.clone());
                let size: Vec2 = texture.size().into();
                self.transform.set_size(size * self.texture_size_multiplier);
            } else {
                return;
            }
        }

        if !self.visible {
            return;
        }

        if let Some(tex) = &self.texture {
            let (size_x, size_y) = self.transform.size().into();
            draw.image(tex)
                .transform(self.transform.mat3_with_parent(parent_affine))
                .size(size_x, size_y);
        }
    }
}
