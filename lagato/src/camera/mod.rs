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
}
