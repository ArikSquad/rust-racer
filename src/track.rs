use raylib::prelude::*;

#[derive(Clone)]
pub struct Checkpoint { pub pos: Vector3, pub radius: f32 }

pub fn build_track() -> (Vec<Rectangle>, Vec<Rectangle>, Vec<Checkpoint>) {
    let mut walls = Vec::new();
    let mut obstacles = Vec::new();

    let half_w = 30.0;
    let half_l = 50.0;
    let thickness = 2.0;

    // outer
    walls.push(Rectangle::new(-half_w, -half_l - thickness, half_w*2.0, thickness));
    walls.push(Rectangle::new(-half_w,  half_l,              half_w*2.0, thickness));
    walls.push(Rectangle::new(-half_w - thickness, -half_l, thickness, half_l*2.0));
    walls.push(Rectangle::new( half_w,             -half_l, thickness, half_l*2.0));

    // inner
    walls.push(Rectangle::new(-10.0, -5.0, 20.0, 2.0));
    walls.push(Rectangle::new(-10.0,  5.0, 20.0, 2.0));

    // obstacles
    obstacles.push(Rectangle::new(-5.0, -20.0, 4.0, 4.0));
    obstacles.push(Rectangle::new( 8.0, -10.0, 3.0, 3.0));
    obstacles.push(Rectangle::new( 0.0,   0.0, 2.5, 2.5));
    obstacles.push(Rectangle::new(-8.0,  15.0, 3.5, 3.5));
    for i in 0..5 {
        let z = -30.0 + i as f32 * 10.0;
        let x = if i % 2 == 0 { -12.0 } else { 12.0 };
        obstacles.push(Rectangle::new(x - 1.2, z - 1.2, 2.4, 2.4));
    }

    let cps = vec![
        Checkpoint { pos: Vector3::new(0.0, 0.0, -half_l + 6.0), radius: 5.5 },
        Checkpoint { pos: Vector3::new(half_w - 6.0, 0.0, 0.0), radius: 5.5 },
        Checkpoint { pos: Vector3::new(0.0, 0.0, half_l - 6.0), radius: 5.5 },
        Checkpoint { pos: Vector3::new(-half_w + 6.0, 0.0, 0.0), radius: 5.5 },
    ];

    (walls, obstacles, cps)
}
