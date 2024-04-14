// Draws an image. No updating or event handling functions.

use notan::{
    draw::{Draw, DrawImages, DrawTransform},
    math::{Affine2, Vec2},
    prelude::Texture,
};

use crate::{transform::Transform, view_trait::ViewTrait, TEX_LOADER};

#[derive(Clone)]
pub struct Image {
    pub visible: bool,
    pub z_order: usize,
    pub transform: Transform,
    tex_id: String,
    tex: Option<Texture>,
    tex_size_mult: f32,
}

impl Image {
    /// The size of the image is the texture size mutiplied by tex_size_mult.
    pub fn new(tex_id: &str, translation: Vec2, tex_size_mult: f32) -> Self {
        let transform = Transform::from_translation(translation);
        Self {
            visible: true,
            z_order: 0,
            transform,
            tex_id: tex_id.to_string(),
            tex: None,
            tex_size_mult,
        }
    }

    pub fn set_texture_id(&mut self, id: String) {
        if self.tex_id != id {
            self.tex_id = id;
            self.tex = None;
        }
    }
}

impl ViewTrait for Image {
    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if self.tex.is_none() && !self.tex_id.is_empty() {
            if let Some(texture) = TEX_LOADER.lock().unwrap().get_tex(&self.tex_id) {
                self.tex = Some(texture.clone());
                let size: Vec2 = texture.size().into();
                self.transform.set_size(size * self.tex_size_mult);
            } else {
                return;
            }
        }

        if !self.visible {
            return;
        }

        if let Some(tex) = &self.tex {
            let (size_x, size_y) = self.transform.size().into();
            draw.image(tex)
                .transform(self.transform.mat3_with_parent(parent_affine))
                .size(size_x, size_y);
        }
    }
}
