use bevy::prelude::{Component, Reflect, Vec3};

#[derive(Component, Debug, Reflect)]
pub struct PerformingMovement;

#[derive(Component, Debug, Reflect)]
pub struct HasJustMoved;

#[derive(Component, Debug, Reflect)]
pub struct TokenHasMoved;

/// Component for animating token movement from one position to another
#[derive(Component, Debug)]
pub struct TokenMoveAnimation {
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub elapsed: f32,
    pub duration: f32,
}

impl TokenMoveAnimation {
    pub fn new(start_pos: Vec3, end_pos: Vec3, duration: f32) -> Self {
        Self {
            start_pos,
            end_pos,
            elapsed: 0.0,
            duration,
        }
    }
    
    /// Returns the interpolated position based on elapsed time
    pub fn current_position(&self) -> Vec3 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        // Use smooth step for nicer easing
        let t = t * t * (3.0 - 2.0 * t);
        self.start_pos.lerp(self.end_pos, t)
    }
    
    /// Returns true if the animation is complete
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}
