use {
    cgmath::{
        EuclideanSpace, SquareMatrix, Rotation3, Transform, InnerSpace,
        Point2, Vector2, Point3, Vector3, Matrix4, Quaternion, PerspectiveFov, Rad,
    },
};

pub struct PitchYawCamera {
    pub pitch: Rad<f32>,
    pub yaw: Rad<f32>,
}

impl PitchYawCamera {
    pub fn new(pitch: Rad<f32>, yaw: Rad<f32>) -> Self {
        PitchYawCamera {
            pitch,
            yaw,
        }
    }

    pub fn handle_mouse_motion(&mut self, relative: Vector2<i32>) {
        let sensitivity = 0.0025;

        self.yaw.0 += relative.x as f32 * -sensitivity;
        self.pitch.0 += relative.y as f32 * -sensitivity;

        let limit = ::std::f32::consts::PI * 0.475;
        self.pitch.0 = self.pitch.0.max(-limit).min(limit);
    }

    pub fn to_rotation(&self) -> Quaternion<f32> {
        Quaternion::from_angle_y(self.yaw) *
        Quaternion::from_angle_x(self.pitch)
    }

    pub fn to_render_camera(
        &self, position: Point3<f32>, window_size: Vector2<u32>
    ) -> RenderCamera {
        RenderCamera::new(
            position,
            self.to_rotation(),
            window_size,
        )
    }
}

pub struct OrbitingCamera {
    pub focus: Point3<f32>,
    pub pitch: Rad<f32>,
    pub yaw: Rad<f32>,
    pub distance: f32,
}

impl OrbitingCamera {
    pub fn new(focus: Point3<f32>, pitch: Rad<f32>, yaw: Rad<f32>, distance: f32) -> Self {
        OrbitingCamera {
            focus,
            pitch,
            yaw,
            distance,
        }
    }

    pub fn to_position_rotation(&self) -> (Point3<f32>, Quaternion<f32>) {
        let rotation =
            Quaternion::from_angle_y(self.yaw) *
            Quaternion::from_angle_x(self.pitch);
        let distance = rotation * Vector3::new(0.0, 0.0, self.distance);
        (self.focus + distance, rotation)
    }

    pub fn to_render_camera(&self, window_size: Vector2<u32>) -> RenderCamera {
        let (position, rotation) = self.to_position_rotation();

        RenderCamera::new(
            position,
            rotation,
            window_size,
        )
    }
}

pub struct RenderCamera {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub window_size: Vector2<u32>,
}

impl RenderCamera {
    pub fn new(
        position: Point3<f32>, rotation: Quaternion<f32>, window_size: Vector2<u32>,
    ) -> Self {
        RenderCamera {
            position,
            rotation,
            window_size,
        }
    }

    pub fn view_matrix_inverse(&self) -> Matrix4<f32> {
        let rotation: Matrix4<f32> = self.rotation.into();
        Matrix4::from_translation(self.position.to_vec()) * rotation
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        let h_fov = ::std::f32::consts::PI / 2.0; // 90 deg
        let v_fov = horizontal_to_vertical_fov(h_fov, self.window_size);

        let ratio = self.window_size.x as f32 / self.window_size.y as f32;
        let projection = PerspectiveFov {
            fovy: Rad(v_fov),
            aspect: ratio,
            near: 0.2,
            far: 1000.0,
        };

        projection.into()
    }

    pub fn model_view_matrix(&self) -> Matrix4<f32> {
        let projection = self.projection_matrix();
        let view = self.view_matrix_inverse();
        let transform = projection * view.invert().unwrap();

        transform
    }

    pub fn pixel_to_ray(&self, pixel_position: Point2<i32>) -> Ray {
        let proj = self.projection_matrix().invert().unwrap();
        let view = self.view_matrix_inverse();

        // Get the clip position of the cursor
        let ray_clip = Vector3::new(
            (pixel_position.x as f32 / self.window_size.x as f32) * 2.0 - 1.0,
            1.0 - (pixel_position.y as f32 / self.window_size.y as f32) * 2.0,
            -1.0,
        );

        // Convert clip cursor to view cursor
        let mut ray_eye = proj.transform_vector(ray_clip);
        ray_eye = Vector3::new(ray_eye.x, ray_eye.y, -1.0);

        // Convert view cursor to world cursor
        let mut ray_world = view.transform_vector(ray_eye);
        ray_world = ray_world.normalize();

        Ray {
            origin: self.position,
            direction: ray_world,
        }
    }
}

fn horizontal_to_vertical_fov(h_fov: f32, window_size: Vector2<u32>) -> f32 {
    let fov_ratio = window_size.y as f32 / window_size.x as f32;
    2.0 * ((h_fov/2.0).tan() * fov_ratio).atan()
}

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}
