use {
    nalgebra::{Point2},
    cassowary::{
        WeightedRelation::*,
        strength::{STRONG},
        Expression, Variable, Constraint,
    },

    attributes::{PanelSize, PanelBox, Orientation},
    input::{FrameCollision},
    panels::{Panel},
    rendering::{Renderer},
    Ui, PanelId, Error, PanelVariables, PanelLayout,
};


pub struct StackPanel {
    size: PanelSize,
    panel_box: PanelBox,
    orientation: Orientation,
    margin: f32,

    children: Vec<PanelId>,
}

impl StackPanel {
    pub fn new(
        size: PanelSize, panel_box: PanelBox, orientation: Orientation, margin: f32
    ) -> Self {
        StackPanel {
            size,
            panel_box,
            orientation,
            margin,

            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, panel: PanelId) {
        self.children.push(panel);
    }

    fn constrain_axis_to_children<F1, F2>(
        &self, constraints: &mut Vec<Constraint>, ui: &Ui,
        this: &PanelVariables, major_axis_map: F1, minor_axis_map: F2,
    ) where
        F1: Fn(&PanelVariables) -> Variable,
        F2: Fn(&PanelVariables) -> Variable,
    {
        let mut major_total_width = Expression::from_constant(self.margin as f64);
        let mut major_total_margin = 0.0;

        if self.children.len() != 0 {
            for child_id in &self.children {
                let child = ui.get(*child_id).unwrap();
                let child_variables = &child.variables;

                // Add this child to our total size expression
                major_total_width = major_total_width + major_axis_map(child_variables);
                major_total_margin += self.margin;

                // We need to size our minor axis to be bigger than the size of children + margin
                constraints.push(
                    minor_axis_map(this)
                    |GE(STRONG)|
                    minor_axis_map(child_variables) + (self.margin * 2.0)
                );
            }
        } else {
            // If we don't have any children at all, we need to do some corrections to still get
            // a valid size based on the margins
            if self.margin != 0.0 {
                major_total_margin += self.margin;
                constraints.push(
                    minor_axis_map(this) |GE(STRONG)| self.margin * 2.0
                );
            }
        }

        // This panel's size should be at least the size of all children combined
        constraints.push(
            major_axis_map(this) |GE(STRONG)| major_total_width + major_total_margin
        );
    }
}

impl Panel for StackPanel {
    fn visible_children(&self) -> Option<&Vec<PanelId>> { Some(&self.children) }

    fn constraints(&self, ui: &Ui, this: &PanelVariables) -> Vec<Constraint> {
        let mut constraints = Vec::new();

        self.size.add_constraints(&mut constraints, this);

        // Prefer a size that at least contains all children
        match self.orientation {
            Orientation::Horizontal => self.constrain_axis_to_children(
                &mut constraints, ui, this, |c| c.width, |c| c.height,
            ),
            Orientation::Vertical => self.constrain_axis_to_children(
                &mut constraints, ui, this, |c| c.height, |c| c.width,
            ),
        }

        constraints
    }

    fn render(
        &self,
        ui: &Ui, renderer: &mut Renderer,
        this_id: PanelId, this_layout: &PanelLayout,
        frame: &mut FrameCollision,
    ) -> Result<(), Error> {
        self.panel_box.render(renderer, this_id, this_layout, false)?;

        let mut stack_position = self.margin;
        for child_id in &self.children {
            let child = ui.get(*child_id).unwrap();

            let position = match self.orientation {
                Orientation::Horizontal => {
                    let position = Point2::new(stack_position, self.margin);
                    stack_position += child.layout.size.x + self.margin;
                    position
                },
                Orientation::Vertical => {
                    let position = Point2::new(self.margin, stack_position);
                    stack_position += child.layout.size.y + self.margin;
                    position
                },
            };

            renderer.render_panel(this_id, *child_id, position)?;
            frame.set(*child_id, position, child.layout.size);
        }

        Ok(())
    }

    fn is_capturing_cursor(&self) -> bool {
        self.panel_box.background.is_some()
    }
}
