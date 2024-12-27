use glam::Vec3;
use miniquad::KeyCode;

pub struct Camera {
    pub position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub last_mouse_pos: (f32, f32),
    pub keys: [bool; 65535],
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vec3::new(50.0, 150.0, 3.0),
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            yaw: -90.0,
            pitch: 0.0,
            movement_speed: 0.5,
            mouse_sensitivity: 0.3,
            last_mouse_pos: (0.0, 0.0),
            keys: [false; 65535],
        }
    }

    pub fn process_input(&mut self) {
        if self.keys[KeyCode::W as usize] {
            self.position += self.front * self.movement_speed;
        }
        if self.keys[KeyCode::S as usize] {
            self.position -= self.front * self.movement_speed;
        }
        if self.keys[KeyCode::A as usize] {
            self.position -= self.front.cross(self.up).normalize() * self.movement_speed;
        }
        if self.keys[KeyCode::D as usize] {
            self.position += self.front.cross(self.up).normalize() * self.movement_speed;
        }
    }

    pub fn process_mouse(&mut self, xoffset: f32, yoffset: f32) {
        let xoffset = xoffset * self.mouse_sensitivity;
        let yoffset = yoffset * self.mouse_sensitivity;

        self.yaw += xoffset;
        self.pitch += yoffset;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        let yaw_radians = self.yaw.to_radians();
        let pitch_radians = self.pitch.to_radians();

        self.front = Vec3::new(
            yaw_radians.cos() * pitch_radians.cos(),
            pitch_radians.sin(),
            yaw_radians.sin() * pitch_radians.cos(),
        )
        .normalize();
    }
}