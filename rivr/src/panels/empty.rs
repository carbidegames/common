use {
    cassowary::{Constraint},

    attributes::{PanelSize, PanelBox},
    input::{FrameCollision},
    panels::{Panel},
    rendering::{Renderer},
    Ui, PanelId, Error, PanelVariables, PanelLayout,
};

pub struct EmptyPanel {
    size: PanelSize,
    panel_box: PanelBox,
}

impl EmptyPanel {
    pub fn new(size: PanelSize, panel_box: PanelBox) -> Self {
        EmptyPanel {
            size,
            panel_box,
        }
    }
}

impl Panel for EmptyPanel {
    fn constraints(&self, _ui: &Ui, this: &PanelVariables) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        self.size.add_constraints(&mut constraints, this);
        constraints
    }

    fn render(
        &self,
        _ui: &Ui, renderer: &mut Renderer,
        this_id: PanelId, this_layout: &PanelLayout,
        _frame: &mut FrameCollision,
    ) -> Result<(), Error> {
        self.panel_box.render(renderer, this_id, this_layout, false)?;

        Ok(())
    }
}
