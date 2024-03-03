use notan::{
    draw::{Draw, DrawImages, DrawTransform},
    math::{Affine2, Vec2},
    prelude::{Color, Texture},
};

use crate::{transform::Transform, view_fn::TEX_SCALE, view_trait::ViewTrait};

pub struct Image {
    pub visible: bool,
    pub z_order: usize,
    pub transform: Transform,
    pub texture: Texture,
    pub color: Color,
}

impl Image {
    pub fn new(texture: Texture, position: Vec2) -> Self {
        let transform = Transform::from_pos_tex_scale_centered(position, &texture, TEX_SCALE, true);
        Self {
            visible: true,
            z_order: 0,
            transform,
            texture,
            color: Color::WHITE,
        }
    }
}

impl ViewTrait for Image {
    fn draw(&mut self, draw: &mut Draw, parent_affine: &Affine2) {
        if !self.visible {
            return;
        }

        let (size_x, size_y) = self.transform.size().into();
        draw.image(&self.texture)
            .transform(self.transform.mat3_with_parent(parent_affine))
            .size(size_x, size_y)
            .color(self.color);
    }
}
