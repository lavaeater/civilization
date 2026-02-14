use bevy::prelude::*;
use crate::civilization::components::GameCamera;

/// Resource that holds a queue of positions the camera should focus on
#[derive(Resource, Default)]
pub struct CameraFocusQueue {
    /// Queue of (position, hold_duration) pairs
    pub targets: Vec<CameraFocusTarget>,
    /// Current focus state
    pub state: CameraFocusState,
}

#[derive(Clone, Debug)]
pub struct CameraFocusTarget {
    pub position: Vec3,
    pub hold_duration: f32,
    pub description: String,
}

#[derive(Default, Clone, Debug)]
pub enum CameraFocusState {
    #[default]
    Idle,
    MovingTo {
        target: CameraFocusTarget,
        start_position: Vec3,
        elapsed: f32,
        move_duration: f32,
    },
    Holding {
        target: CameraFocusTarget,
        elapsed: f32,
    },
}

impl CameraFocusQueue {
    pub fn add_focus(&mut self, position: Vec3, hold_duration: f32, description: impl Into<String>) {
        self.targets.push(CameraFocusTarget {
            position,
            hold_duration,
            description: description.into(),
        });
    }
    
    pub fn is_busy(&self) -> bool {
        !matches!(self.state, CameraFocusState::Idle) || !self.targets.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.targets.clear();
        self.state = CameraFocusState::Idle;
    }
}

/// System that smoothly moves the camera to focus targets
pub fn process_camera_focus(
    time: Res<Time>,
    mut focus_queue: ResMut<CameraFocusQueue>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };
    
    let dt = time.delta_secs();
    
    match &mut focus_queue.state {
        CameraFocusState::Idle => {
            if let Some(target) = focus_queue.targets.first().cloned() {
                focus_queue.targets.remove(0);
                let start_position = camera_transform.translation;
                let distance = start_position.distance(target.position);
                let move_duration = (distance / 500.0).clamp(0.3, 1.5);
                
                info!("[CAMERA] Focusing on: {}", target.description);
                focus_queue.state = CameraFocusState::MovingTo {
                    target,
                    start_position,
                    elapsed: 0.0,
                    move_duration,
                };
            }
        }
        CameraFocusState::MovingTo { target, start_position, elapsed, move_duration } => {
            *elapsed += dt;
            let t = (*elapsed / *move_duration).min(1.0);
            let eased_t = ease_out_cubic(t);
            
            camera_transform.translation = start_position.lerp(target.position, eased_t);
            
            if t >= 1.0 {
                camera_transform.translation = target.position;
                focus_queue.state = CameraFocusState::Holding {
                    target: target.clone(),
                    elapsed: 0.0,
                };
            }
        }
        CameraFocusState::Holding { target, elapsed } => {
            *elapsed += dt;
            if *elapsed >= target.hold_duration {
                focus_queue.state = CameraFocusState::Idle;
            }
        }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
