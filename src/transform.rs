use notan::math::{Affine2, Mat3, Vec2};

/*
Note: Glam docs say Affine2 is much faster than Mat3 for 2D work.
*/

#[derive(Clone)]
pub struct Transform {
    translation: Vec2,
    /// Radians
    angle: f32,
    /// Default is (1.0, 1.0). Scale affects drawn size only, not position or rotation.
    scale: Vec2,
    /// Default is centered (0.5, 0.5). Children of this transform do not inherit
    /// the parent's offset.
    offset: Vec2,
    /// Default (0.0, 0.0). Set to full image size. With most draw() implementations, this
    /// does not affect the drawn size. Use set_scale() or set_scale_from_size() to adjust
    /// drawn size.
    full_size: Vec2,
    /// Stores the Affine2 representing the above fields. If None, the affine is calculated
    /// and then stored. Setting the other fields to new values sets the affine to None again.
    affine: Option<Affine2>,
    /// Stores the Mat3 representing the above fields. If None, the matrix is calculated
    /// and then stored. Setting the other fields to new values sets the matrix to None again.
    matrix: Option<Mat3>,

    pub drawn_size: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vec2::ZERO,
            angle: 0.0,
            scale: Vec2::ONE,
            offset: Vec2::new(0.5, 0.5),
            full_size: Vec2::ZERO,
            affine: None,
            matrix: None,
            drawn_size: Vec2::ZERO,
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
    pub fn from_translation_angle_full_size(
        translation: Vec2,
        angle: f32,
        full_size: Vec2,
    ) -> Self {
        Transform {
            translation,
            angle,
            full_size,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn translation(&self) -> Vec2 {
        self.translation
    }

    #[allow(dead_code)]
    pub fn set_translation(&mut self, translation: Vec2) {
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
        self.scale = scale;
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    /// Sets the  scale to achieve the given desired size when drawn.
    pub fn set_scale_from_size(&mut self, size: Vec2) {
        self.scale = Vec2::new(size.x / self.full_size.x, size.y / self.full_size.y);
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    pub fn scaled_size(&self) -> Vec2 {
        Vec2::new(
            self.full_size.x * self.scale.x,
            self.full_size.y * self.scale.y,
        )
    }

    #[allow(dead_code)]
    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    #[allow(dead_code)]
    pub fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
        self.affine = None;
        self.matrix = None;
    }

    #[allow(dead_code)]
    pub fn full_size(&self) -> Vec2 {
        self.full_size
    }

    #[allow(dead_code)]
    /// This should be set to the full image size. With most draw() implementations,
    /// use set_scale() or set_scale_from_size() to change drawn size.
    pub fn set_full_size(&mut self, full_size: Vec2) {
        self.full_size = full_size;
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
                // let offset = Vec2::new(
                //     -self.full_size.x * self.offset.x,
                //     -self.full_size.y * self.offset.y,
                // );
                let offset = Vec2::new(
                    -self.drawn_size.x * self.offset.x,
                    -self.drawn_size.y * self.offset.y,
                );
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

    pub fn transform_screen_point(
        &mut self,
        screen_pt: Vec2,
        parent_affine: Option<&Affine2>,
    ) -> Vec2 {
        let combined_affine = match parent_affine {
            Some(parent) => *parent * self.affine2(),
            None => self.affine2(),
        };
        combined_affine.inverse().transform_point2(screen_pt)
    }

    #[allow(dead_code)]
    pub fn contains_screen_point(
        &mut self,
        screen_pt: Vec2,
        parent_affine: Option<&Affine2>,
    ) -> bool {
        let pt = self.transform_screen_point(screen_pt, parent_affine);
        // pt.x >= 0.0 && pt.y >= 0.0 && pt.x <= self.full_size.x && pt.y <= self.full_size.y
        pt.x >= 0.0 && pt.y >= 0.0 && pt.x <= self.drawn_size.x && pt.y <= self.drawn_size.y
    }
}
