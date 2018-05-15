use {
    nalgebra::{Point2},
    palette::{Srgba},
    lyon::{
        math::rect,
        tessellation as lt,
    },

    rendering::{Renderer},
    Error, PanelId, PanelLayout,
};

#[derive(Clone)]
pub struct PanelBox {
    pub background: Option<Srgba>,
    pub background_hovering: Option<Srgba>,
    pub border_radius: f32,
}

impl Default for PanelBox {
    fn default() -> Self {
        PanelBox {
            background: None,
            background_hovering: None,
            border_radius: 0.0,
        }
    }
}

impl PanelBox {
    pub fn render(
        &self,
        renderer: &mut Renderer, this_id: PanelId, this_layout: &PanelLayout, hovering: bool,
    ) -> Result<(), Error> {
        let background = if !hovering {
            self.background
        } else {
            self.background_hovering.or(self.background)
        };

        if let Some(background) = background {
            if self.border_radius == 0.0 {
                // Simple rectangle fast path
                renderer.render_vertices(this_id, &[
                    Point2::new(0.0, 0.0),
                    Point2::new(0.0, this_layout.size.y),
                    Point2::new(this_layout.size.x, this_layout.size.y),
                    Point2::new(this_layout.size.x, 0.0),
                ], &[0, 1, 3, 2, 3, 1], background)?;
            } else {
                // Generate the rounded rectangle
                let mut geometry = lt::VertexBuffers::new();
                let options = lt::FillOptions::tolerance(0.1);
                lt::basic_shapes::fill_rounded_rectangle(
                    &rect(0.0, 0.0, this_layout.size.x, this_layout.size.y),
                    &lt::basic_shapes::BorderRadii {
                        top_left: self.border_radius,
                        top_right: self.border_radius,
                        bottom_left: self.border_radius,
                        bottom_right: self.border_radius,
                    },
                    &options,
                    &mut lt::geometry_builder::simple_builder(&mut geometry),
                );

                // Send it over to the renderer
                let vertices: Vec<_> = geometry.vertices.into_iter()
                    .map(|v| Point2::new(v.position.x, v.position.y)).collect();
                renderer.render_vertices(
                    this_id,
                    &vertices,
                    &geometry.indices,
                    background,
                )?;
            }
        }

        Ok(())
    }
}
