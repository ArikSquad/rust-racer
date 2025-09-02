use crate::input::InputState;
use raylib::prelude::*;

pub const MAX_ENGINE_FORCE: f32 = 45.0; // forward thrust
pub const BRAKE_FORCE: f32 = 60.0; // stronger opposing force
pub const STEER_RATE: f32 = 2.8; // radians/sec steering speed
pub const ANGULAR_DAMP: f32 = 3.0; // steering recenters
pub const LINEAR_DAMP: f32 = 0.9; // rolling friction (per second)
pub const DRAG_COEFF: f32 = 0.25; // quadratic drag ~ v^2
pub const GRIP: f32 = 6.0; // lateral grip (higher = less drift)
pub const HAND_BRAKE_GRIP: f32 = 1.6; // grip when handbrake held
pub const ROT_FRICTION: f32 = 2.5; // angular friction (yaw inertia damping)

#[derive(Clone, Copy, Debug)]
pub struct Car {
    pub pos: Vector3,
    pub yaw: f32,
    pub vel: Vector3,
    pub yaw_rate: f32,
    pub size: Vector3,
}

impl Car {
    pub fn new(pos: Vector3) -> Self {
        Self {
            pos,
            yaw: 0.0,
            vel: Vector3::zero(),
            yaw_rate: 0.0,
            size: Vector3::new(1.8, 0.8, 3.4),
        }
    }

    pub fn forward(&self) -> Vector3 {
        let sy = self.yaw.sin();
        let cy = self.yaw.cos();
        Vector3::new(-sy, 0.0, -cy)
    }

    pub fn right(&self) -> Vector3 {
        let f = self.forward();
        Vector3::new(-f.z, 0.0, f.x)
    }

    pub fn speed(&self) -> f32 {
        self.vel.dot(self.forward())
    }

    pub fn update(&mut self, dt: f32, input: &InputState) {
        let target_yaw_rate =
            (input.steer_left as i32 - input.steer_right as i32) as f32 * STEER_RATE;
        let diff = target_yaw_rate - self.yaw_rate;
        let max_step = ANGULAR_DAMP * dt;
        self.yaw_rate += diff.clamp(-max_step, max_step);

        // angular friction (yaw inertia)
        self.yaw_rate *= (1.0 - ROT_FRICTION * dt).max(0.0);
        self.yaw += self.yaw_rate * dt;

        let mut engine = 0.0;
        if input.accel {
            engine += MAX_ENGINE_FORCE;
        }
        if input.brake {
            engine -= BRAKE_FORCE;
        }

        let fwd = self.forward();
        let right = self.right();

        let v_long = self.vel.dot(fwd);
        let v_lat = self.vel.dot(right);

        let mut a_long = engine;
        a_long -= DRAG_COEFF * v_long.abs() * v_long.signum();

        let grip = if input.handbrake {
            HAND_BRAKE_GRIP
        } else {
            GRIP
        };
        let a_lat = -grip * v_lat;

        let acc = fwd * a_long + right * a_lat;

        self.vel += acc * dt;
        self.vel *= (1.0 - LINEAR_DAMP * dt).max(0.0);

        self.pos += self.vel * dt;
        self.pos.y = 0.4; // keep car on ground
    }
}
