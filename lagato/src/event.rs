#[derive(Deserialize, Serialize)]
pub struct Event {
    raised: bool,
}

impl Event {
    pub fn new() -> Self {
        Event {
            raised: false,
        }
    }

    pub fn raise(&mut self) {
        self.raised = true
    }

    pub fn check(&mut self) -> bool {
        let value = self.raised;
        self.raised = false;
        value
    }
}
