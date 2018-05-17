use {
    alga::linear::{Transformation},
    nalgebra::{Vector2, Point3, Vector3, Matrix4, UnitQuaternion, Perspective3},
};

pub struct PitchYawCamera {
    pub pitch: f32,
    pub yaw: f32,
}

impl PitchYawCamera {
    pub fn new(pitch: f32, yaw: f32) -> Self {
        PitchYawCamera {
            pitch,
            yaw,
        }
    }

    pub fn handle_mouse_motion(&mut self, relative: Vector2<i32>) {
        let sensitivity = 0.0025;

        self.yaw += relative.x as f32 * -sensitivity;
        self.pitch += relative.y as f32 * -sensitivity;

        let limit = ::std::f32::consts::PI * 0.475;
        self.pitch = self.pitch.max(-limit).min(limit);
    }

    pub fn to_render_camera(&self, position: Point3<f32>) -> RenderCamera {
        RenderCamera::new(
            position,
            UnitQuaternion::from_euler_angles(self.pitch, self.yaw, 0.0),
        )
    }
}

pub struct OrbitingCamera {
    pub focus: Point3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
}

impl OrbitingCamera {
    pub fn new(focus: Point3<f32>, pitch: f32, yaw: f32, distance: f32) -> Self {
        OrbitingCamera {
            focus,
            pitch,
            yaw,
            distance,
        }
    }

    pub fn to_render_camera(&self) -> RenderCamera {
        let rotation = Matrix4::from_euler_angles(self.pitch, self.yaw, 0.0);
        let distance = rotation.transform_vector(&Vector3::new(0.0, 0.0, self.distance));

        RenderCamera::new(
            self.focus + distance,
            UnitQuaternion::from_euler_angles(self.pitch, self.yaw, 0.0),
        )
    }
}

pub struct RenderCamera {
    position: Point3<f32>,
    rotation: UnitQuaternion<f32>,
}

impl RenderCamera {
    pub fn new(position: Point3<f32>, rotation: UnitQuaternion<f32>) -> Self {
        RenderCamera {
            position,
            rotation,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.position.coords) * self.rotation.to_homogeneous()
    }

    pub fn world_to_clip_matrix(&self, window_size: Vector2<u32>) -> Matrix4<f32> {
        let h_fov = ::std::f32::consts::PI / 2.0; // 90 deg
        let fov_ratio = window_size.y as f32 / window_size.x as f32;
        let v_fov = 2.0 * ((h_fov/2.0).tan() * fov_ratio).atan();

        // Aspect ratio, FOV, znear, zfar
        let ratio = window_size.x as f32 / window_size.y as f32;
        let projection = Perspective3::new(ratio, v_fov, 0.2, 1000.0);
        let view = self.view_matrix();
        let transform = projection.as_matrix() * view.try_inverse().unwrap();

        transform
    }
}
