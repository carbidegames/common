use {
    std::any::{Any},

    cassowary::{Constraint},

    input::{FrameCollision},
    rendering::{Renderer},
    Ui, PanelId, Error, PanelLayout, PanelVariables,
};

pub trait Panel: Any {
    /// Returns a vector of the children that need to be layouted, and rendered.
    fn visible_children(&self) -> Option<&Vec<PanelId>> { None }

    fn constraints(&self, ui: &Ui, this: &PanelVariables) -> Vec<Constraint>;

    fn render(
        &self,
        ui: &Ui, renderer: &mut Renderer,
        this_id: PanelId, this_layout: &PanelLayout,
        frame: &mut FrameCollision,
    ) -> Result<(), Error>;

    fn is_capturing_cursor(&self) -> bool { false }

    /// If returns true, component will be re-rendered.
    fn handle_hover_start(&mut self) -> bool { false }

    /// If returns true, component will be re-rendered.
    fn handle_hover_end(&mut self) -> bool { false }

    fn handle_pressed(&mut self) {}
}
