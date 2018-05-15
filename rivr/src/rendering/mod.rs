use {
    nalgebra::{Point2, Vector2},
    palette::{Srgba},

    input::{FrameCollision},
    Error, RenderingError, Ui, PanelId,
};

pub trait Renderer {
    /// Gets the render target's size in UI units.
    fn target_size(&mut self) -> Vector2<f32>;

    /// Finishes rendering by displaying the root panel's cache on the target.
    fn finish_to_target(&mut self, root_id: PanelId) -> Result<(), Error>;

    /// Creates or resizes a cache for the panel, ensuring it's available.
    /// Returns true if the cache has been created or recreated, and thus is empty.
    fn create_resize_cache(
        &mut self, panel_id: PanelId, size: Vector2<u32>
    ) -> Result<bool, Error>;

    fn clear_cache(&mut self, panel_id: PanelId) -> Result<(), Error>;

    fn render_panel(
        &mut self,
        target_id: PanelId,
        source_id: PanelId,
        position: Point2<f32>,
    ) -> Result<(), Error>;

    fn render_vertices(
        &mut self,
        panel_id: PanelId,
        vertices: &[Point2<f32>], indices: &[u16], color: Srgba,
    ) -> Result<(), Error>;

    fn render_raw(
        &mut self,
        panel_id: PanelId,
        image_data: &Vec<u8>, image_size: Vector2<usize>,
        color: Srgba,
    ) -> Result<(), Error>;
}

pub fn render<R: Renderer>(
    ui: &mut Ui, renderer: &mut R, frame: &mut FrameCollision
) -> Result<(), Error> {
    let root_id = ui.root_id()?;

    // First re-layout the UI, we only need to do this during rendering, input should make use of
    // the cached information gathered here to be consistent with what's visible on screen
    let size = renderer.target_size();
    ui.solve_layout(size);

    // Insert the parent into the frame, this is needed because parents are responsible for adding
    // children to the frame, but the root has no parent
    frame.set(root_id, Point2::new(0.0, 0.0), size);

    // Make sure the root panel is rendered, then display it to the target
    render_panel(ui, renderer, root_id, frame)?;
    renderer.finish_to_target(root_id)?;

    // Mark everything as rendered, at this point everything should be there
    for (_, panel_entry) in &mut ui.entries {
        panel_entry.needs_rendering = false;
    }

    Ok(())
}

fn render_panel<R: Renderer>(
    ui: &Ui, renderer: &mut R,
    panel_id: PanelId, frame: &mut FrameCollision,
) -> Result<bool, Error> {
    let panel_entry = ui.get(panel_id).unwrap();
    let panel_size = panel_entry.layout.size;

    // If we got a zero (or less) size, skip this. It's not renderable
    // TODO: Clear the cache entry if this is the case
    if panel_size.x.ceil() < 1.0 || panel_size.y.ceil() < 1.0 {
        return Ok(false)
    }

    // If we got 1_000_000 or more, that means a panel has been told to maximize size without
    // anything to constrain it
    if panel_size.x >= 1_000_000.0 || panel_size.y >= 1_000_000.0 {
        return Err(Error::Rendering(RenderingError::PanelTooLarge))
    }

    // Make sure this panel's cache is created and of the correct size
    let cache_empty = renderer.create_resize_cache(panel_id, Vector2::new(
        panel_size.x.ceil() as u32, panel_size.y.ceil() as u32,
    ))?;

    // The parent's children need to be rendered first
    let mut child_rendered = false;
    if let Some(children) = panel_entry.panel.visible_children() {
        for child_id in children {
            child_rendered |= render_panel(ui, renderer, *child_id, frame)?
        }
    }

    // Render the component itself if we need to
    if cache_empty || child_rendered || panel_entry.needs_rendering {
        renderer.clear_cache(panel_id)?;

        panel_entry.panel.render(ui, renderer, panel_id, &panel_entry.layout, frame)?;

        Ok(true)
    } else {
        Ok(false)
    }
}
