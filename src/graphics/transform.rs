//! Some common transformation matrices that I couldn't find in the cgmath library.

use cgmath::{Matrix3, One, Vector2, Zero};

pub fn translate<T: One + Zero>(offset: Vector2<T>) -> Matrix3<T> {
    Matrix3::new(
        T::one(), T::zero(), T::zero(),
        T::zero(), T::one(), T::zero(),
        offset.x, offset.y, T::one(),
    )
}

pub fn scale<T: One + Zero>(scale: Vector2<T>) -> Matrix3<T> {
    Matrix3::new(
        scale.x, T::zero(), T::zero(),
        T::zero(), scale.y, T::zero(),
        T::zero(), T::zero(), T::one()
    )
}



#[test]
fn test_translate() {
    let t = translate(Vector2::new(7.0, -3.0f32));
    let x = Vector2::new(5.0, 2.0);
    let tx = Vector2::new(12.0, -1.0);
    assert_eq!((t * x.extend(1.0)).xy(), tx);
}

#[test]
fn test_scale() {
    let t = scale(Vector2::new(2.0, -4.0f32));
    let x = Vector2::new(-3.0, 2.0);
    let tx = Vector2::new(-6.0, -8.0);
    assert_eq!((t * x.extend(1.0)).xy(), tx);
}
