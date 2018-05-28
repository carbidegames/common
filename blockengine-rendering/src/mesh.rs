use {
    ggez::{
        Context,
        graphics,
    },
    gfx::{
        traits::{FactoryExt},
        handle::{Buffer},
        Slice,
    },
    gfx_device_gl::{Resources},
    nalgebra::{Point3, Vector3},

    lagato::{grid::{Voxels}},

    renderer::{Vertex},
};

pub struct Mesh {
    pub vbuf: Buffer<Resources, Vertex>,
    pub slice: Slice<Resources>,
}

impl Mesh {
    pub fn new(ctx: &mut Context, vertices: &Vec<Vertex>) -> Self {
        let factory = graphics::get_factory(ctx);
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(vertices, ());

        Self {
            vbuf,
            slice,
        }
    }

    pub fn cube(ctx: &mut Context) -> Self {
        let mut vertices = Vec::new();
        add_cube_vertices(&mut vertices, Vector3::new(0.0, 0.0, 0.0));
        Self::new(ctx, &vertices)
    }
}

pub fn triangulate_voxels(voxels: &Voxels<bool>) -> Vec<Vertex> {
    // Add some cubes
    let mut vertices = Vec::new();
    for position in voxels.iter_pos() {
        if *voxels.get(position).unwrap() {
            add_cube_vertices(
                &mut vertices,
                Vector3::new(position.x as f32, position.y as f32, position.z as f32)
            );
        }
    }

    vertices
}

fn add_cube_vertices(vertices: &mut Vec<Vertex>, offset: Vector3<f32>) {
    let points = [
        [
            [
                Point3::new(0.0, 0.0, 0.0) + offset,
                Point3::new(0.0, 0.0, 1.0) + offset,
            ],
            [
                Point3::new(0.0, 1.0, 0.0) + offset,
                Point3::new(0.0, 1.0, 1.0) + offset,
            ],
        ],
        [
            [
                Point3::new(1.0, 0.0, 0.0) + offset,
                Point3::new(1.0, 0.0, 1.0) + offset,
            ],
            [
                Point3::new(1.0, 1.0, 0.0) + offset,
                Point3::new(1.0, 1.0, 1.0) + offset,
            ],
        ],
    ];

    // front (0, 0, 1)
    add_plane_vertices(vertices,
        points[0][0][1], points[1][0][1], points[0][1][1], points[1][1][1],
    );

    // back (0, 0, -1)
    add_plane_vertices(vertices,
        points[0][1][0], points[1][1][0], points[0][0][0], points[1][0][0],
    );

    // right (1, 0, 0)
    add_plane_vertices(vertices,
        points[1][0][0], points[1][1][0], points[1][0][1], points[1][1][1],
    );

    // left (-1, 0, 0)
    add_plane_vertices(vertices,
        points[0][1][0], points[0][0][0], points[0][1][1], points[0][0][1],
    );

    // top (0, 1, 0)
    add_plane_vertices(vertices,
        points[1][1][0], points[0][1][0], points[1][1][1], points[0][1][1],
    );

    // bottom (0, -1, 0)
    add_plane_vertices(vertices,
        points[0][0][0], points[1][0][0], points[0][0][1], points[1][0][1],
    );
}

fn add_plane_vertices(
    vertices: &mut Vec<Vertex>,
    lb: Point3<f32>, rb: Point3<f32>, lt: Point3<f32>, rt: Point3<f32>
) {
    vertices.push(Vertex { pos: nvtp(lb), tex_coord: [0.0, 0.0] });
    vertices.push(Vertex { pos: nvtp(rb), tex_coord: [1.0, 0.0] });
    vertices.push(Vertex { pos: nvtp(rt), tex_coord: [1.0, 1.0] });

    vertices.push(Vertex { pos: nvtp(lb), tex_coord: [0.0, 0.0] });
    vertices.push(Vertex { pos: nvtp(rt), tex_coord: [1.0, 1.0] });
    vertices.push(Vertex { pos: nvtp(lt), tex_coord: [0.0, 1.0] });
}

fn nvtp(v: Point3<f32>) -> [f32; 4] {
    [v.x, v.y, v.z, 1.0]
}
