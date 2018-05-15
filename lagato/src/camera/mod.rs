use {
    alga::linear::{Transformation},
    nalgebra::{Point3, Vector3, Matrix4, UnitQuaternion},
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
}
