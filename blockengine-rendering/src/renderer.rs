use {
    ggez::{
        graphics,
        Context, GameResult,
    },
    gfx::{
        traits::{FactoryExt},
        handle::{Buffer},
        self, PipelineState, VertexBuffer, ConstantBuffer, TextureSampler, RenderTarget,
        DepthTarget,
    },
    gfx_device_gl::{Resources},
    cgmath::{EuclideanSpace, Matrix4},

    lagato::{camera::{RenderCamera}},

    Object,
};

type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_pos",
        tex_coord: [f32; 2] = "a_tex_coord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_transform",
    }

    pipeline pipe {
        vbuf: VertexBuffer<Vertex> = (),
        locals: ConstantBuffer<Locals> = "Locals",
        texture: TextureSampler<[f32; 4]> = "u_texture",
        out_color: RenderTarget<ColorFormat> = "o_color",
        out_depth: DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub struct Renderer {
    pso: PipelineState<Resources, pipe::Meta>,
    locals: Buffer<Resources, Locals>,
}

impl Renderer {
    pub fn new(ctx: &mut Context) -> Self {
        let (factory, _device, _encoder, _depth_view, _color_view) =
            graphics::get_gfx_objects(ctx);

        // Create pipeline state object
        let vs = include_bytes!("s_vertex.glsl");
        let fs = include_bytes!("s_fragment.glsl");
        let set = factory.create_shader_set(vs, fs).unwrap();
        let pso = factory.create_pipeline_state(
            &set,
            gfx::Primitive::TriangleList,
            gfx::state::Rasterizer::new_fill().with_cull_back(),
            pipe::new()
        ).unwrap();

        let locals = factory.create_constant_buffer(1);

        Renderer {
            pso,
            locals,
        }
    }

    pub fn draw(
        &mut self, ctx: &mut Context,
        camera: &RenderCamera, objects: &Vec<Object>,
    ) -> GameResult<()> {
        graphics::set_background_color(ctx, (10, 10, 15).into());
        graphics::clear(ctx);

        {
            let (_factory, device, encoder, depth_view, color_view) =
                graphics::get_gfx_objects(ctx);
            encoder.clear(&color_view, [0.1, 0.1, 0.1, 1.0]);
            encoder.clear_depth(&depth_view, 1.0);

            let camera = camera.model_view_matrix();

            for object in objects {
                if !object.visible {
                    continue
                }

                let model = Matrix4::from_translation(object.position.to_vec());
                let transform = camera * model;
                let locals = Locals {
                    transform: transform.into(),
                };
                encoder.update_constant_buffer(&self.locals, &locals);

                let data = pipe::Data {
                    vbuf: object.mesh.vbuf.clone(),
                    locals: self.locals.clone(),
                    texture: (object.texture.view.clone(), object.texture.sampler.clone()),
                    out_color: color_view.clone(),
                    out_depth: depth_view.clone(),
                };
                encoder.draw(&object.mesh.slice, &self.pso, &data);
            }

            encoder.flush(device);
        }

        graphics::present(ctx);

        Ok(())
    }
}
