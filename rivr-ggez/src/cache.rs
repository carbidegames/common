use {
    ggez::{
        Context,
        conf::{NumSamples},
        graphics::{self, Canvas},
    },
    metrohash::{MetroHashMap},

    rivr::{
        Error, PanelId,
        attributes::{Vector2},
    },

    egtr,
};

/// A persistent resource cache for the ggez markedly renderer.
pub struct GgezRivrCache {
    panels: MetroHashMap<PanelId, Canvas>,
}

impl GgezRivrCache {
    pub fn new() -> Self {
        GgezRivrCache {
            panels: MetroHashMap::default(),
        }
    }

    pub fn canvas_for_panel(&self, panel_id: PanelId) -> Option<&Canvas> {
        self.panels.get(&panel_id)
    }

    pub fn create_resize_cache(
        &mut self, ctx: &mut Context, panel_id: PanelId, size: Vector2<u32>
    ) -> Result<bool, Error> {
        // If we have a cached canvas and it's of the right size, we only have to clear
        if let Some(canvas) = self.panels.get(&panel_id) {
            if canvas.get_image().width() == size.x &&
                canvas.get_image().height() == size.y {
                return Ok(false)
            }
        }

        // We don't have what we need so create a new canvas
        let canvas = Canvas::new(ctx, size.x, size.y, NumSamples::One).map_err(egtr)?;
        self.panels.insert(panel_id, canvas);

        Ok(true)
    }

    pub fn clear_cache(&mut self, ctx: &mut Context, panel_id: PanelId) -> Result<(), Error> {
        let canvas = self.panels.get(&panel_id).unwrap();
        graphics::set_canvas(ctx, Some(canvas));
        graphics::set_background_color(ctx, (255, 255, 255, 0).into());
        graphics::clear(ctx);

        Ok(())
    }
}
