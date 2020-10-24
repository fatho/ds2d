use cgmath::prelude::*;
use cgmath::Vector2;

/// An axis-aligned rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect<S> {
    pub top_left: Vector2<S>,
    pub bottom_right: Vector2<S>,
}

impl<S: Copy> Rect<S> {
    /// Return the four corner coordinates in clockwise order starting at the top-left.
    pub fn corners(&self) -> [Vector2<S>; 4] {
        [
            self.top_left,
            Vector2::new(self.bottom_right.x, self.top_left.y),
            self.bottom_right,
            Vector2::new(self.top_left.x, self.bottom_right.y),
        ]
    }
}

impl<S: cgmath::BaseNum> Rect<S> {
    /// The unit square in the positive quadrant of the cartesian plane.
    pub fn unit_square() -> Self {
        Rect {
            top_left: Vector2::new(S::zero(), S::zero()),
            bottom_right: Vector2::new(S::one(), S::one()),
        }
    }

    /// In a coordinate system where negative Y points up and and negative X points left,
    /// create a rectangle from its center point and size.
    pub fn from_center_size(center: Vector2<S>, size: Vector2<S>) -> Rect<S> {
        let hsize = size / (S::one() + S::one());
        Rect {
            top_left: center - hsize,
            bottom_right: center + hsize,
        }
    }

    pub fn size(&self) -> Vector2<S> {
        self.bottom_right - self.top_left
    }

    pub fn center(&self) -> Vector2<S> {
        Vector2 {
            x: (self.top_left.x + self.bottom_right.x) / (S::one() + S::one()),
            y: (self.top_left.y + self.bottom_right.y) / (S::one() + S::one()),
        }
    }

    pub fn translated(self, offset: Vector2<S>) -> Rect<S> {
        Rect {
            top_left: self.top_left + offset,
            bottom_right: self.bottom_right + offset,
        }
    }

    pub fn scaled_around_center(self, factor: S) -> Rect<S> {
        let new_size = self.size() * factor;
        Self::from_center_size(self.center(), new_size)
    }

    /// Return a rectangle with the given size that is centered on the current rectangle.
    pub fn centered(&self, size: Vector2<S>) -> Self {
        Rect::from_center_size(self.center(), size)
    }
}

impl<S: cgmath::BaseFloat> Rect<S> {
    /// Return the four corner coordinates in clockwise order starting at the top-left,
    /// but rotated  by the angle implicitly given by its sine and cosine
    /// around the origin given in relative coordinates.
    pub fn rotated_corners(
        &self,
        origin: Vector2<S>,
        rotation_cos: S,
        rotation_sin: S,
    ) -> [Vector2<S>; 4] {
        let size = self.size();

        let origin_absolute = self.top_left + size.mul_element_wise(origin);
        let top_left_relative = -size.mul_element_wise(origin);
        let bottom_right_relative =
            size.mul_element_wise(Vector2::new(S::one(), S::one()) - origin);
        let top_right_relative = Vector2::new(bottom_right_relative.x, top_left_relative.y);
        let bottom_left_relative = Vector2::new(top_left_relative.x, bottom_right_relative.y);

        let transform =
            cgmath::Matrix2::new(rotation_cos, rotation_sin, -rotation_sin, rotation_cos);

        let top_left = origin_absolute + transform * top_left_relative;
        let bottom_right = origin_absolute + transform * bottom_right_relative;
        let top_right = origin_absolute + transform * top_right_relative;
        let bottom_left = origin_absolute + transform * bottom_left_relative;

        [top_left, top_right, bottom_right, bottom_left]
    }

    /// Return a transformation matrix that takes points from this rectangle
    /// to the corresponding point in the target rectangle.
    pub fn transform_to(&self, target_rect: Rect<S>) -> cgmath::Matrix4<S> {
        // TODO: add tests for this
        let self_to_origin = cgmath::Matrix4::from_translation(-self.center().extend(S::zero()));
        let self_size = self.size();
        let target_size = target_rect.size();
        let scale = cgmath::Matrix4::from_nonuniform_scale(
            target_size.x / self_size.x,
            target_size.y / self_size.y,
            S::one(),
        );
        let target_from_origin =
            cgmath::Matrix4::from_translation(target_rect.center().extend(S::zero()));

        target_from_origin * scale * self_to_origin
    }
}

impl<S> From<rusttype::Rect<S>> for Rect<S> {
    fn from(r: rusttype::Rect<S>) -> Self {
        Self {
            top_left: Vector2::new(r.min.x, r.min.y),
            bottom_right: Vector2::new(r.max.x, r.max.y),
        }
    }
}
