// TODO - support more than 8-bit depths

use kiss3d::resource::{Mesh, MeshManager};
use nalgebra::{normalize, Point3, Vector3};
use png::{BitDepth, ColorType, Decoder, DecodingError};
use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidFile,
    FileNotSupported,
}

#[derive(Debug)]
pub struct Heightmap {
    src_data: Vec<u8>,
    width: usize,
    height: usize,
    height_scale: f32,
    height_offset: f32,
    tiles: Vec<Tile>,
}

const TILE_SIZE: usize = 128;

#[derive(Debug)]
pub struct Tile {
    name: String,
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
}

impl Tile {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Heightmap {
    pub fn from_png_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
        let f = File::open(file_path)?;

        let decoder = Decoder::new(f);
        let (info, mut reader) = decoder.read_info().unwrap();

        assert_eq!(
            info.color_type,
            ColorType::Grayscale,
            "Only 8-bit grayscale is supported"
        );

        assert_eq!(
            info.bit_depth,
            BitDepth::Eight,
            "Only 8-bit grayscale is supported"
        );

        // TODO - update these
        // Enforce mesh tiling constraints
        assert_eq!(
            info.width as usize % TILE_SIZE,
            0,
            "Only mod {} dimensions are supported",
            TILE_SIZE
        );
        assert_eq!(
            info.height as usize % TILE_SIZE,
            0,
            "Only mod {} dimensions are supported",
            TILE_SIZE
        );

        // Allocate and fill the output buffer
        let mut buf: Vec<u8> = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;

        // Split up the grid into TILE_SIZE x TILE_SIZE meshes
        let num_tiles_x = info.width as usize / TILE_SIZE;
        let num_tiles_y = info.width as usize / TILE_SIZE;

        let mut tiles: Vec<Tile> = Vec::new();
        for ty in 0..num_tiles_y {
            for tx in 0..num_tiles_x {
                let tile = Tile {
                    name: format!("{} {}", tx, ty),
                    start_x: (tx * TILE_SIZE),
                    start_y: (ty * TILE_SIZE),
                    width: TILE_SIZE,
                    height: TILE_SIZE,
                };
                tiles.push(tile);
            }
        }

        Ok(Self {
            src_data: buf,
            width: info.width as _,
            height: info.height as _,
            height_scale: 5.0,
            height_offset: 0.0,
            tiles,
        })
    }

    pub fn populate_mesh_manager(&self, mm: &mut MeshManager) -> &[Tile] {
        for tile in &self.tiles {
            let mesh = self.generate_mesh(tile);
            //mm.add(Rc::new(RefCell::new(mesh)).clone(), tile.name());
            mm.add(Rc::new(RefCell::new(mesh)), tile.name());
        }

        &self.tiles
    }

    fn generate_mesh(&self, tile: &Tile) -> Mesh {
        // TODO - need to juggle the boundary conditions to correctly stitch the tiles
        //let twidth = (self.width +- 1) as f32;
        //let theight = (self.height +- 1) as f32;
        //
        let twidth = (self.width - 1) as f32;
        let theight = (self.height - 1) as f32;
        let half_twidth = twidth / 2_f32;
        let half_theight = theight / 2_f32;

        // For each quad in the mesh, generate two triangles
        // Each triangle has three indices
        let mut vertices: Vec<Point3<f32>> = vec![];
        let mut normals: Vec<Vector3<f32>> = vec![];
        let mut indices: Vec<Point3<u16>> = vec![];

        // Generate the vertices for the vbo
        for y in tile.start_y..(tile.start_y + tile.height) {
            for x in tile.start_x..(tile.start_x + tile.width) {
                let index = (y * self.width) + x;

                let elevation: f32 = self.src_data[index] as f32 / 255.0;

                let s = x as f32 / twidth;
                let t = y as f32 / theight;

                // TODO - clean this up
                // Stitch the tile edges together
                let px = if x == tile.start_x {
                    x as f32 - (self.width / 2) as f32
                } else if x == (tile.start_x + tile.width - 1) {
                    (x + 1) as f32 - (self.width / 2) as f32
                } else {
                    (s * twidth) - half_twidth
                };

                let pz = if y == tile.start_y {
                    y as f32 - (self.height / 2) as f32
                } else if y == (tile.start_y + tile.height - 1) {
                    (y + 1) as f32 - (self.height / 2) as f32
                } else {
                    (t * theight) - half_theight
                };

                // align coordinate frame, Y-up
                vertices.push(Point3::new(
                    // image width mapped to X axis
                    px,
                    // depth/elevation mapped to Y axis
                    (elevation * self.height_scale) + self.height_offset,
                    // image height mapped to Y axis
                    pz,
                ));

                // Push empty normal vector
                normals.push(Vector3::new(0_f32, 0_f32, 0_f32));
            }
        }

        // Generate the indices for the ibo
        for y in 0..(tile.height - 1) {
            for x in 0..(tile.width - 1) {
                let index = (y * tile.width) + x;

                // top triangle T0 v0->v1->v2
                indices.push(Point3::new(
                    index as _,
                    index as u16 + tile.width as u16 + 1,
                    index as u16 + 1,
                ));

                // bottom triangle T1 v0->v1->v2
                indices.push(Point3::new(
                    index as _,
                    index as u16 + tile.width as u16,
                    index as u16 + tile.width as u16 + 1,
                ));
            }
        }

        // Generate the normals for the nbo
        for ibo_idx in (0..indices.len()).step_by(3) {
            let v0 = vertices[indices[ibo_idx].x as usize];
            let v1 = vertices[indices[ibo_idx].y as usize];
            let v2 = vertices[indices[ibo_idx].z as usize];

            let a = v1 - v0;
            let b = v2 - v0;

            let normal = normalize(&a.cross(&b));

            normals[indices[ibo_idx].x as usize] += normal;
            normals[indices[ibo_idx].y as usize] += normal;
            normals[indices[ibo_idx].z as usize] += normal;
        }

        // Normalize the normals
        for n in normals.iter_mut() {
            *n = normalize(n);
        }

        Mesh::new(vertices, indices, Some(normals), None, false)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        match e {
            _ => Error::InvalidFile,
        }
    }
}

impl From<DecodingError> for Error {
    fn from(e: DecodingError) -> Error {
        match e {
            _ => Error::FileNotSupported,
        }
    }
}
