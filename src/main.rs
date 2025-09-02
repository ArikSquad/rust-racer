use raylib::ffi;
use raylib::prelude::*;

mod car;
mod input;
mod track;

use car::Car;
use input::InputState;
use track::build_track;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Raylib racing")
        .msaa_4x()
        .build();

    rl.set_target_fps(120);

    let mut car = Car::new(Vector3::new(0.0, 0.4, 20.0));
    let (walls, obstacles, checkpoints) = build_track();
    let mut next_cp = 0usize;
    let mut lap: u32 = 0;
    let mut lap_start_time = rl.get_time();

    let cam_back = car.forward() * -8.5 + Vector3::new(0.0, 5.0, 0.0);
    let mut camera = Camera3D::perspective(
        car.pos + cam_back,
        car.pos + Vector3::new(0.0, 0.8, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().max(1.0 / 1000.0);
        let input = InputState::read(&rl);

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            car = Car::new(Vector3::new(0.0, 0.4, 20.0));
            next_cp = 0;
            lap_start_time = rl.get_time();
        }

        car.update(dt, &input);

        // collisions
        let half = car.size / 2.0;
        let mut p = car.pos;
        let mut v = car.vel;
        for r in walls.iter().chain(obstacles.iter()) {
            let left = r.x - half.x;
            let right = r.x + r.width + half.x;
            let top = r.y - half.z;
            let bottom = r.y + r.height + half.z;

            if p.x > left && p.x < right && p.z > top && p.z < bottom {
                let dx = (right - p.x).min(p.x - left);
                let dz = (bottom - p.z).min(p.z - top);
                if dx < dz {
                    if (p.x - left) < (right - p.x) {
                        p.x = left;
                    } else {
                        p.x = right;
                    }
                    v.x = 0.0;
                } else {
                    if (p.z - top) < (bottom - p.z) {
                        p.z = top;
                    } else {
                        p.z = bottom;
                    }
                    v.z = 0.0;
                }
            }
        }
        car.pos = p;
        car.vel = v;

        // checkpoints
        let cp = &checkpoints[next_cp];
        if car.pos.distance_to(cp.pos) <= cp.radius {
            next_cp = (next_cp + 1) % checkpoints.len();
            if next_cp == 0 {
                lap += 1;
                let now = rl.get_time();
                println!("Lap {} in {:.3}s", lap, now - lap_start_time);
                lap_start_time = now;
            }
        }

        // camera
        let cam_back = car.forward() * -8.5 + Vector3::new(0.0, 5.0, 0.0);
        let desired_pos = car.pos + cam_back;
        camera.position = camera.position.lerp(desired_pos, 1.0 - f32::exp(-6.0 * dt));
        camera.target = camera.target.lerp(
            car.pos + Vector3::new(0.0, 0.8, 0.0),
            1.0 - f32::exp(-8.0 * dt),
        );

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::SKYBLUE);

        {
            let mut d3 = d.begin_mode3D(camera);

            // render car
            unsafe { ffi::rlPushMatrix() };
            unsafe { ffi::rlTranslatef(car.pos.x, car.pos.y, car.pos.z) };
            unsafe { ffi::rlRotatef(car.yaw.to_degrees(), 0.0, 1.0, 0.0) };
            d3.draw_cube(
                Vector3::zero(),
                car.size.x,
                car.size.y,
                car.size.z,
                Color::RED,
            );
            d3.draw_cube_wires(
                Vector3::zero(),
                car.size.x,
                car.size.y,
                car.size.z,
                Color::BLACK,
            );
            unsafe { ffi::rlPopMatrix() };

            // render indicator
            let nose_pos = car.pos
                + car.forward() * (car.size.z * 0.6)
                + Vector3::new(0.0, car.size.y * 0.3, 0.0);
            d3.draw_sphere(nose_pos, 0.15, Color::YELLOW);

            // render floor grid
            d3.draw_grid(10, 1.0);

            // render walls (grey) and obstacles (orange)
            for r in &walls {
                let center = Vector3::new(r.x + r.width / 2.0, 0.0, r.y + r.height / 2.0);
                let size = Vector3::new(r.width, 1.0, r.height);
                d3.draw_cube(
                    center + Vector3::new(0.0, 0.5, 0.0),
                    size.x,
                    size.y,
                    size.z,
                    Color::GRAY,
                );
            }
            for r in &obstacles {
                let center = Vector3::new(r.x + r.width / 2.0, 0.0, r.y + r.height / 2.0);
                let size = Vector3::new(r.width, 1.0, r.height);
                d3.draw_cube(
                    center + Vector3::new(0.0, 0.5, 0.0),
                    size.x,
                    size.y,
                    size.z,
                    Color::ORANGE,
                );
            }

            // render checkpoints
            for (i, cp) in checkpoints.iter().enumerate() {
                let color = if i == next_cp {
                    Color::GREEN
                } else {
                    Color::DARKGREEN
                };
                d3.draw_cylinder_wires(
                    cp.pos + Vector3::new(0.0, 0.1, 0.0),
                    cp.radius,
                    cp.radius,
                    0.2,
                    20,
                    color,
                );
            }
        }

        let speed_kmh = (car.speed() * 3.6).abs();
        d.draw_text(
            &format!("Speed: {:>5.1} km/h", speed_kmh),
            20,
            20,
            24,
            Color::WHITE,
        );
        d.draw_text(
            &format!("Lap: {}  Next CP: {}", lap, next_cp + 1),
            20,
            52,
            24,
            Color::WHITE,
        );
        d.draw_text(
            "W/S accel & brake, A/D steer, SPACE handbrake, R reset",
            20,
            84,
            20,
            Color::LIGHTGRAY,
        );
    }
}
