use notan::math::{Affine2, Mat3, Vec2};

/*
Notes:
- Glam docs say Affine2 is much faster than Mat3 for 2D work.
- Using 'scale' to control image size is problematic when it comes to drawing
children. Use the 'draw_size' field to store this value.
*/

#[derive(Clone)]
pub struct Transform {
    translation: Vec2,
    /// Radians
    angle: f32,
    /// Default is Vec2::ONE.
    scale: Vec2,
    /// For centering and hit detection. Default is Vec2::ZERO.
    size: Vec2,
    /// Default is centered (0.5, 0.5). Children of this transform do not inherit
    /// the parent's offset.
    offset: Vec2,
    /// Stores the Affine2 representing the above fields. If None, the affine is calculated
    /// and then stored. Setting the other fields to new values sets the affine to None again.
    affine: Option<Affine2>,
    /// Stores the Mat3 representing the above fields. If None, the matrix is calculated
    /// and then stored. Setting the other fields to new values sets the matrix to None again.
    matrix: Option<Mat3>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vec2::ZERO,
            angle: 0.0,
            scale: Vec2::ONE,

            size: Vec2::ZERO,
            offset: Vec2::new(0.5, 0.5),

            affine: None,
            matrix: None,
        }
    }
}

impl Transform {
    #[allow(dead_code)]
    pub fn from_translation_angle(translation: Vec2, angle: f32) -> Self {
        Transform {
            translation,
            angle,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn from_translation_size_centered(
        translation: Vec2,
        draw_size: Vec2,
        centered: bool,
    ) -> Self {
        let offset = if centered {
            Vec2::new(0.5, 0.5)
        } else {
            Vec2::ZERO
        };
        Transform {
            translation,
            size: draw_size,
            offset,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    /// A game friendly constructor. Texture and texture scale are supplied
    /// to compute draw size.
    pub fn from_pos_tex_scale_centered(
        pos: Vec2,
        tex: &notan::app::Texture,
        tex_scale: f32,
        centered: bool,
    ) -> Self {
        let offset = if centered {
            Vec2::new(0.5, 0.5)
        } else {
            Vec2::ZERO
        };
        Transform {
            translation: pos,
            size: Vec2::new(tex.size().0 / tex_scale, tex.size().1 / tex_scale),
            offset,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn translation(&self) -> Vec2 {
        self.translation
    }

    #[allow(dead_code)]
    pub fn set_translation(&mut self, translation: Vec2) {
        if self.translation == translation {
            return;
        }
        self.translation = translation;
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    pub fn angle(&self) -> f32 {
        self.angle
    }

    #[allow(dead_code)]
    pub fn set_angle(&mut self, angle: f32) {
        if self.angle == angle {
            return;
        }
        self.angle = angle;
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    #[allow(dead_code)]
    pub fn set_scale(&mut self, scale: Vec2) {
        if self.scale == scale {
            return;
        }
        self.scale = scale;
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    #[allow(dead_code)]
    pub fn set_size(&mut self, size: Vec2) {
        if self.size == size {
            return;
        }
        self.size = size;
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    #[allow(dead_code)]
    pub fn set_offset(&mut self, offset: Vec2) {
        if self.offset == offset {
            return;
        }
        self.offset = offset;
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    pub fn affine2(&mut self) -> Affine2 {
        match self.affine {
            Some(a) => return a,
            None => {
                let affine =
                    Affine2::from_scale_angle_translation(self.scale, self.angle, self.translation);

                let offset = Vec2::new(-self.size.x * self.offset.x, -self.size.y * self.offset.y);
                let offset_affine = Affine2::from_translation(offset);

                self.affine = Some(affine * offset_affine);
                self.affine.unwrap()
            }
        }
    }

    /// Convert self.affine to Mat3 (if needed) and return it.
    pub fn mat3(&mut self) -> Mat3 {
        match self.matrix {
            Some(m) => m,
            None => {
                self.matrix = Some((self.affine2()).into());
                self.matrix.unwrap()
            }
        }
    }

    /// Combine the transform with the parent affine. Useful for Notan's draw.transform() fn.
    pub fn mat3_with_parent(&mut self, parent_affine: &Affine2) -> Mat3 {
        Into::<Mat3>::into(*parent_affine) * self.mat3()
    }

    pub fn transform_screen_point(&mut self, screen_pt: Vec2, parent_affine: &Affine2) -> Vec2 {
        let affine = *parent_affine * self.affine2();
        affine.inverse().transform_point2(screen_pt)
    }

    #[allow(dead_code)]
    pub fn contains_screen_point(&mut self, screen_pt: Vec2, parent_affine: &Affine2) -> bool {
        let pt = self.transform_screen_point(screen_pt, parent_affine);
        pt.x >= 0.0 && pt.y >= 0.0 && pt.x <= self.size.x && pt.y <= self.size.y
    }
}
