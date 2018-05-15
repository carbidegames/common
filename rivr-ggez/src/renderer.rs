use {
    ggez::{
        Context,
        graphics::{self, Mesh, Rect, Image},
    },

    rivr::{
        Error, PanelId,
        attributes::{Point2, Vector2, Srgba},
        rendering::{Renderer},
    },

    GgezRivrCache, egtr,
};


pub struct GgezRivrRenderer<'a> {
    ctx: &'a mut Context,
    cache: &'a mut GgezRivrCache,
}

impl<'a> GgezRivrRenderer<'a> {
    pub fn new(ctx: &'a mut Context, cache: &'a mut GgezRivrCache) -> Self {
        GgezRivrRenderer {
            ctx,
            cache,
        }
    }

    fn prepare_cache(&mut self, panel_id: PanelId) -> Result<(), Error> {
        let canvas = self.cache.canvas_for_panel(panel_id).unwrap();
        graphics::set_canvas(self.ctx, Some(canvas));
        graphics::set_screen_coordinates(self.ctx, Rect::new(
            0.0, 0.0,
            canvas.get_image().width() as f32, canvas.get_image().height() as f32,
        )).map_err(egtr)?;
        graphics::apply_transformations(self.ctx).map_err(egtr)?;

        Ok(())
    }
}

impl<'a> Renderer for GgezRivrRenderer<'a> {
    fn target_size(&mut self) -> Vector2<f32> {
        let target_size = graphics::get_size(self.ctx);
        Vector2::new(target_size.0 as f32, target_size.1 as f32)
    }

    fn finish_to_target(&mut self, root_id: PanelId) -> Result<(), Error> {
        graphics::set_canvas(self.ctx, None);

        // Set the coordinates to 1 pixel = 1 unit
        let target_size = graphics::get_size(self.ctx);
        graphics::set_screen_coordinates(self.ctx, Rect::new(
            0.0, 0.0,
            target_size.0 as f32, target_size.1 as f32,
        )).map_err(egtr)?;
        graphics::apply_transformations(self.ctx).map_err(egtr)?;

        // Render the root panel's cache to the screen if there is anything to draw
        if let Some(canvas) = self.cache.canvas_for_panel(root_id) {
            graphics::set_color(self.ctx, (255, 255, 255, 255).into()).map_err(egtr)?;
            graphics::draw(self.ctx, canvas, Point2::new(0.0, 0.0), 0.0).map_err(egtr)?;
        }

        Ok(())
    }

    fn create_resize_cache(
        &mut self, panel_id: PanelId, size: Vector2<u32>
    ) -> Result<bool, Error> {
        self.cache.create_resize_cache(self.ctx, panel_id, size)
    }

    fn clear_cache(&mut self, panel_id: PanelId) -> Result<(), Error> {
        self.cache.clear_cache(self.ctx, panel_id)
    }

    fn render_panel(
        &mut self,
        target_id: PanelId,
        source_id: PanelId,
        position: Point2<f32>,
    ) -> Result<(), Error> {
        self.prepare_cache(target_id)?;

        if let Some(source_canvas) = self.cache.canvas_for_panel(source_id) {
            graphics::set_color(self.ctx, (255, 255, 255, 255).into()).map_err(egtr)?;
            graphics::draw(self.ctx, source_canvas, Point2::new(
                position.x.round(),
                position.y.round(),
            ), 0.0).map_err(egtr)?;
        }

        Ok(())
    }

    fn render_vertices(
        &mut self,
        panel_id: PanelId,
        vertices: &[Point2<f32>], indices: &[u16], color: Srgba,
    ) -> Result<(), Error> {
        self.prepare_cache(panel_id)?;

        graphics::set_color(self.ctx, color_convert(color)).map_err(egtr)?;

        // Convert the vertices+indices to triangles and then a mesh
        let mut flattened_vertices = Vec::new();
        for index in indices {
            flattened_vertices.push(vertices[*index as usize]);
        }
        let mesh = Mesh::from_triangles(self.ctx, &flattened_vertices).map_err(egtr)?;

        graphics::draw(self.ctx, &mesh, Point2::new(0.0, 0.0), 0.0).map_err(egtr)?;

        Ok(())
    }

    fn render_raw(
        &mut self,
        panel_id: PanelId,
        image_data: &Vec<u8>, image_size: Vector2<usize>,
        color: Srgba,
    ) -> Result<(), Error> {
        self.prepare_cache(panel_id)?;

        let image = Image::from_rgba8(
            self.ctx,
            image_size.x as u16,
            image_size.y as u16,
            image_data,
        ).map_err(egtr)?;

        graphics::set_color(self.ctx, color_convert(color)).map_err(egtr)?;
        graphics::draw(self.ctx, &image, Point2::new(0.0, 0.0), 0.0).map_err(egtr)?;

        Ok(())
    }
}

fn color_convert(color: Srgba) -> ::ggez::graphics::Color {
    ::ggez::graphics::Color::new(color.red, color.green, color.blue, color.alpha)
}
