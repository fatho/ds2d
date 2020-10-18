use cgmath::Vector2;

/// An axis-aligned rectangle
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: Copy> Rect<T> {
    pub fn position(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    pub fn set_position(&mut self, new_position: Vector2<T>) {
        self.x = new_position.x;
        self.y = new_position.y;
    }

    pub fn size(&self) -> Vector2<T> {
        Vector2::new(self.w, self.h)
    }
}

impl<T: PartialOrd> Rect<T> {}
