use {
    ggez::{
        graphics,
        Context, GameResult,
    },
    gfx::{
        traits::{FactoryExt},
        self, PipelineState, VertexBuffer, ConstantBuffer, TextureSampler,
        RenderTarget, DepthTarget,
    },
    gfx_device_gl::{Resources},
    nalgebra::{Vector2, Matrix4},

    lagato::{camera::{RenderCamera}},

    Object, Texture,
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
    data: pipe::Data<Resources>,
    pso: PipelineState<Resources, pipe::Meta>,
}

impl Renderer {
    pub fn new(ctx: &mut Context, block_texture: &Texture) -> Self {
        let (factory, _device, _encoder, depth_view, color_view) =
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

        // Bundle all the data together
        let data = pipe::Data {
            vbuf: factory.create_vertex_buffer(&[]),
            locals: factory.create_constant_buffer(1),
            texture: (block_texture.view.clone(), block_texture.sampler.clone()),
            out_color: color_view,
            out_depth: depth_view,
        };

        Renderer {
            data,
            pso,
        }
    }

    pub fn draw(
        &mut self, ctx: &mut Context,
        camera: &RenderCamera, objects: &Vec<Object>,
    ) -> GameResult<()> {
        graphics::set_background_color(ctx, (10, 10, 15).into());
        graphics::clear(ctx);
        let (window_width, window_height) = graphics::get_size(ctx);

        {
            let (_factory, device, encoder, _depthview, _colorview) =
                graphics::get_gfx_objects(ctx);
            encoder.clear(&self.data.out_color, [0.1, 0.1, 0.1, 1.0]);
            encoder.clear_depth(&self.data.out_depth, 1.0);

            let camera = camera.world_to_clip_matrix(Vector2::new(window_width, window_height));

            for object in objects {
                self.data.vbuf = object.mesh.vbuf.clone();

                let model = Matrix4::new_translation(&object.position.coords);
                let transform = camera * model;
                let locals = Locals {
                    transform: transform.into(),
                };
                encoder.update_constant_buffer(&self.data.locals, &locals);

                encoder.draw(&object.mesh.slice, &self.pso, &self.data);
            }

            encoder.flush(device);
        }

        graphics::present(ctx);

        Ok(())
    }
}
