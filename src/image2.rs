// Draws an image. No updating or event handling functions.

use notan::{
    draw::{Draw, DrawImages, DrawTransform},
    math::{Affine2, Vec2},
    prelude::{Color, Texture},
};

use crate::{TEX_LOADER, transform::Transform, view_trait::ViewTrait};

#[derive(Clone)]
pub struct Image2 {
    pub visible: bool,
    pub z_order: usize,
    pub transform: Transform,
    pub texture_id: String,
    pub texture: Option<Texture>,
}

impl Image2 {
    pub fn new(texture_id: String, size: Vec2, position: Vec2) -> Self {
        let transform = Transform::from_translation_size_centered(position, size, true);
        Self {
            visible: true,
            z_order: 0,
            transform,
            texture_id,
            texture: None,
        }
    }
}

impl ViewTrait for Image2 {
    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if self.texture.is_none() {
            if let Some(texture) = TEX_LOADER.lock().unwrap().get_tex(&self.texture_id) {
                println!("Image texture loaded");
                self.texture = Some(texture.clone());
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
