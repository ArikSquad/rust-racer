use raylib::prelude::*;

pub struct InputState {
    pub accel: bool,
    pub brake: bool,
    pub steer_left: bool,
    pub steer_right: bool,
    pub handbrake: bool,
}

impl InputState {
    pub fn read(rl: &RaylibHandle) -> Self {
        Self {
            accel: rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP),
            brake: rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN),
            steer_left: rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT),
            steer_right: rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT),
            handbrake: rl.is_key_down(KeyboardKey::KEY_SPACE),
        }
    }
}
