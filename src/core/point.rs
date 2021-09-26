use libm::powf;

/// Type alias representing audio data.
/// There are no guarantees but ideally this value
/// should never exceed beyond `-1.0` or `1.0`.
pub type Point = f32;

/// Amplify a point using decibels
pub fn amplify(point: Point, db: f32) -> Point {
    if db == 0.0 {
        point
    } else {
        let ratio = powf(10.0, db / 20.0);
        (point * ratio).clamp(-1.0, 1.0)
    }
}
