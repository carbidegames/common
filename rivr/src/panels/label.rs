use {
    nalgebra::{Vector2},
    palette::{Srgba},
    cassowary::{
        WeightedRelation::*,
        strength::{STRONG, REQUIRED},
        Constraint,
    },
    rusttype::{Font, Scale, PositionedGlyph, point},

    input::{FrameCollision},
    panels::{Panel},
    rendering::{Renderer},
    Ui, PanelId, Error, FontId, PanelVariables, PanelLayout,
};

pub struct LabelPanel {
    text: String,
    text_scale: Scale,
    font_id: FontId,

    text_bounds: Vector2<f32>,
}

impl LabelPanel {
    pub fn new<S: Into<String>>(
        ui: &Ui, text: S, font_id: FontId, text_scale: f32,
    ) -> Result<Self, Error> {
        let text: String = text.into();
        let text_scale = Scale::uniform(text_scale);

        // Calculate sizing data for this label from the text we got
        let font = ui.resources.font(font_id)?;
        let (_glyphs, mut text_bounds) = layout_text_line(&text, font, text_scale);
        text_bounds.x = text_bounds.x.ceil();
        text_bounds.y = text_bounds.y.ceil();

        Ok(LabelPanel {
            text,
            text_scale,
            font_id,

            text_bounds,
        })
    }
}

impl Panel for LabelPanel {
    fn constraints(&self, _ui: &Ui, this: &PanelVariables) -> Vec<Constraint> {
        vec!(
            // Must be non-negative size
            this.width |GE(REQUIRED)| 0.0,
            this.height |GE(REQUIRED)| 0.0,

            // Prefer to contain its contents
            this.width |EQ(STRONG)| self.text_bounds.x as f64,
            this.height |EQ(STRONG)| self.text_bounds.y as f64,
        )
    }

    fn render(
        &self,
        ui: &Ui, renderer: &mut Renderer,
        this_id: PanelId, _this_layout: &PanelLayout,
        _frame: &mut FrameCollision,
    ) -> Result<(), Error> {
        // First, layout the glyphs
        let font = ui.resources.font(self.font_id)?;
        let (glyphs, text_scale) = layout_text_line(
            &self.text, font, self.text_scale,
        );

        // Prepare the image data to render the glyphs to
        let width_px = ::std::cmp::max(text_scale.x.ceil() as usize, 1);
        let height_px = ::std::cmp::max(text_scale.y.ceil() as usize, 1);
        let bytes_per_pixel = 4;
        let mut pixel_data = vec![0; width_px * height_px * bytes_per_pixel];
        let pitch = width_px * bytes_per_pixel;

        // Render the glyphs to the image data
        for g in glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                // v is the amount of the pixel covered
                // by the glyph, in the range 0.0 to 1.0
                g.draw(|x, y, v| {
                    let c = (v * 255.0) as u8;
                    let x = x as i32 + bb.min.x;
                    let y = y as i32 + bb.min.y;
                    // There's still a possibility that the glyph clips the boundaries of the
                    // bitmap
                    if x >= 0 && x < width_px as i32 && y >= 0 && y < height_px as i32
                    {
                        let x = x as usize * bytes_per_pixel;
                        let y = y as usize;
                        pixel_data[(x + y * pitch)] = 255;
                        pixel_data[(x + y * pitch + 1)] = 255;
                        pixel_data[(x + y * pitch + 2)] = 255;
                        pixel_data[(x + y * pitch + 3)] = c;
                    }
                })
            }
        }

        // Render the image data to the component cache
        renderer.render_raw(
            this_id,
            &pixel_data, Vector2::new(width_px, height_px),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        )?;

        Ok(())
    }
}

fn layout_text_line<'a>(
    text: &str, font: &'a Font, scale: Scale,
) -> (Vec<PositionedGlyph<'a>>, Vector2<f32>) {
    let mut positioned_glyphs = Vec::new();

    let v_metrics = font.v_metrics(scale);
    //let new_line_height = v_metrics.ascent + -v_metrics.descent + v_metrics.line_gap;

    let mut caret = point(0.0, v_metrics.ascent);
    let mut last_glyph_id = None;

    for c in text.chars() {
        // Skip control characters in single-line drawing
        if c.is_control() {
            continue;
        }

        // Look up the glyph for this character
        let base_glyph = font.glyph(c);

        // Add the kerning needed for the last glyph next to this glyph
        if let Some(id) = last_glyph_id.take() {
            caret.x += font.pair_kerning(scale, id, base_glyph.id());
        }
        last_glyph_id = Some(base_glyph.id());

        // Position the glyph for this character
        let glyph = base_glyph.scaled(scale).positioned(caret);
        caret.x += glyph.unpositioned().h_metrics().advance_width;
        positioned_glyphs.push(glyph);
    }

    (positioned_glyphs, Vector2::new(
        caret.x,
        caret.y + -v_metrics.descent,
    ))
}
