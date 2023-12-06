use cgmath::{Vector2, Zero};

pub struct CursorInput {
    pub cursor_position: Vector2<f64>,
    pub last_cursor_position: Vector2<f64>,
}

impl CursorInput {
    pub fn new() -> Self{
        Self{
            cursor_position: Vector2::<f64>::zero(),
            last_cursor_position: Vector2::<f64>::zero(),
        }
    }
}