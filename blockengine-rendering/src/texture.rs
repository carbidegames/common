use {
    std::io::{Read},

    ggez::{
        graphics,
        Context, GameResult,
    },
    gfx::{
        texture::{SamplerInfo, Kind, Mipmap, AaMode, FilterMethod, WrapMode},
        handle::{ShaderResourceView, Sampler},
        self, Factory,
    },
    gfx_device_gl::{Resources},
    image,
};

pub struct Texture {
    pub view: ShaderResourceView<Resources, [f32; 4]>,
    pub sampler: Sampler<Resources>,
}

impl Texture {
    pub fn load(ctx: &mut Context, path: &str) -> GameResult<Self> {
        let mut buffer = Vec::new();
        let mut reader = ctx.filesystem.open(path)?;
        reader.read_to_end(&mut buffer).unwrap();

        let (factory, _device, _encoder, _depth_view, _color_view) =
            graphics::get_gfx_objects(ctx);

        // Create a texture for the voxels
        let image = image::load_from_memory(&buffer).unwrap().to_rgba();
        let image_dimensions = image.dimensions();

        let data: [&[u8]; 1] = [&image.into_raw()];
        let (_, view) = factory
            .create_texture_immutable_u8::<gfx::format::Srgba8>(
                Kind::D2(image_dimensions.0 as u16, image_dimensions.1 as u16, AaMode::Single),
                Mipmap::Provided,
                &data,
            )
            .unwrap();

        let sinfo = SamplerInfo::new(FilterMethod::Bilinear, WrapMode::Clamp);
        let sampler = factory.create_sampler(sinfo);

        Ok(Texture {
            view,
            sampler,
        })
    }
}
