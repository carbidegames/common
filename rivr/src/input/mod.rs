use {
    std::collections::hash_map::{Entry},

    nalgebra::{Point2, Vector2},
    metrohash::{MetroHashMap},

    Ui, PanelId, Error,
};

/// The collision data for a single (rendered) UI frame.
#[derive(Debug)]
pub struct FrameCollision {
    panels: MetroHashMap<PanelId, PanelCollision>,
}

impl FrameCollision {
    pub fn new() -> Self {
        FrameCollision {
            panels: MetroHashMap::default(),
        }
    }

    pub fn set(&mut self, panel_id: PanelId, position: Point2<f32>, size: Vector2<f32>) {
        match self.panels.entry(panel_id) {
            Entry::Occupied(mut o) => {
                let v = o.get_mut();
                v.position = position;
                v.size = size;
            },
            Entry::Vacant(v) => { v.insert(PanelCollision { position, size }); }
        };
    }
}

#[derive(Debug)]
pub struct PanelCollision {
    position: Point2<f32>,
    size: Vector2<f32>,
}

pub struct PcInputHandler {
    hovering_over: Option<PanelId>,
}

impl PcInputHandler {
    /// Creates a new UI input handler.
    pub fn new() -> Self {
        PcInputHandler {
            hovering_over: None,
        }
    }

    /// Returns true if the cursor is currently over a panel that captures cursor movement.
    pub fn is_cursor_over_ui(&self) -> bool {
        self.hovering_over.is_some()
    }

    /// Handles cursor movement.
    pub fn handle_cursor_moved(
        &mut self, position: Point2<f32>, ui: &mut Ui, frame: &FrameCollision,
    ) -> Result<(), Error> {
        let new_hovering = find_at_position(
            position, ui, frame, ui.root_id()?, Point2::new(0.0, 0.0),
        );

        if let Some(new_hovering) = new_hovering {
            // If the thing we're hovering over is a new thing, we need to notify it
            if self.hovering_over.map(|v| v != new_hovering).unwrap_or(true) {
                let panel_entry = ui.get_mut(new_hovering).unwrap();
                panel_entry.needs_rendering |= panel_entry.panel.handle_hover_start();
            }
        }

        if let Some(hovering_over) = self.hovering_over {
            // If the thing we're hovering over is a new thing, we need to notify the old one
            if new_hovering.map(|v| v != hovering_over).unwrap_or(true) {
                let panel_entry = ui.get_mut(hovering_over).unwrap();
                panel_entry.needs_rendering |= panel_entry.panel.handle_hover_end();
            }
        }

        self.hovering_over = new_hovering;

        Ok(())
    }

    /// Handles the start of a cursor or touch drag.
    pub fn handle_drag_started(
        &mut self, _position: Point2<f32>, _ui: &mut Ui, _frame: &FrameCollision,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Handles the end of a cursor or touch drag.
    pub fn handle_drag_ended(
        &mut self, position: Point2<f32>, ui: &mut Ui, frame: &FrameCollision,
    ) -> Result<(), Error> {
        if let Some(panel_id) = find_at_position(
            position, ui, frame, ui.root_id()?, Point2::new(0.0, 0.0),
        ) {
            let panel_entry = ui.get_mut(panel_id).unwrap();
            panel_entry.panel.handle_pressed();
        }

        Ok(())
    }
}

fn find_at_position(
    cursor_position: Point2<f32>,
    ui: &Ui, frame: &FrameCollision,
    panel_id: PanelId, parent_position: Point2<f32>,
) -> Option<PanelId> {
    let panel_entry = ui.get(panel_id).unwrap();

    // If there's no entry for this panel in the frame, it wasn't rendered and thus not relevant
    let frame_entry = if let Some(frame_entry) = frame.panels.get(&panel_id) {
        frame_entry
    } else {
        return None
    };

    let panel_position = parent_position + frame_entry.position.coords;
    let panel_size = frame_entry.size;

    // If the position isn't over us, it also won't be over any children, so just return none
    if cursor_position.x < panel_position.x ||
        cursor_position.y < panel_position.y ||
        cursor_position.x > panel_position.x + panel_size.x ||
        cursor_position.y > panel_position.y + panel_size.y {
        return None
    }

    // If this component doesn't capture input, we still need to check children, but we can't
    // return this one.
    let mut found_id = if panel_entry.panel.is_capturing_cursor() {
        Some(panel_id)
    } else {
        None
    };

    // Go through all children, if any of them find a hit, replace the ID we found, we want to find
    // the last one that matches because it's the one rendered on top. The function will
    // recursively find the deepest matching child like this.
    if let Some(children) = panel_entry.panel.visible_children() {
        for child_id in children {
            if let Some(id) = find_at_position(
                cursor_position, ui, frame, *child_id, panel_position,
            ) {
                found_id = Some(id);
            }
        }
    }

    found_id
}
