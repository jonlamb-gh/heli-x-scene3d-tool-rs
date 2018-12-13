// TODO - verify channel depth, hard-coded to 8 bits

use image::{DynamicImage, GenericImage, ImageError, Pixel};
use kiss3d::resource::{Mesh, MeshManager};
use nalgebra::{normalize, Point2, Point3, Vector3};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidFile,
    FileNotSupported,
}

pub struct Heightmap {
    src_img: DynamicImage,
    width: usize,
    height: usize,
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
    pub fn from_png_file(file_path: &Path) -> Result<Self, Error> {
        let src_img = image::open(file_path)?;

        let (src_width, src_height) = src_img.dimensions();

        // TODO - make this not necessary
        // Enforce mesh tiling constraints
        assert_eq!(
            src_width as usize % TILE_SIZE,
            0,
            "Only mod {} dimensions are supported",
            TILE_SIZE
        );
        assert_eq!(
            src_height as usize % TILE_SIZE,
            0,
            "Only mod {} dimensions are supported",
            TILE_SIZE
        );

        // Split up the grid into TILE_SIZE x TILE_SIZE meshes
        let num_tiles_x = src_width as usize / TILE_SIZE;
        let num_tiles_y = src_height as usize / TILE_SIZE;

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
            src_img,
            width: src_width as _,
            height: src_height as _,
            tiles,
        })
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /*
    pub fn src_image(&self) -> &DynamicImage {
        &self.src_img
    }
    */

    pub fn src_texture(&self) -> DynamicImage {
        DynamicImage::ImageRgb8(self.src_img.to_rgb())
    }

    pub fn populate_mesh_manager(&self, mm: &mut MeshManager) -> &[Tile] {
        for tile in &self.tiles {
            let mesh = self.generate_mesh(tile);
            mm.add(Rc::new(RefCell::new(mesh)), tile.name());
        }

        &self.tiles
    }

    fn generate_mesh(&self, tile: &Tile) -> Mesh {
        let twidth = (self.width - 1) as f32;
        let theight = (self.height - 1) as f32;
        let half_twidth = twidth / 2_f32;
        let half_theight = theight / 2_f32;

        // For each quad in the mesh, generate two triangles
        // Each triangle has three indices
        let mut vertices: Vec<Point3<f32>> = vec![];
        let mut normals: Vec<Vector3<f32>> = vec![];
        let mut indices: Vec<Point3<u16>> = vec![];
        let mut uvs: Vec<Point2<f32>> = vec![];

        // Generate the vertices for the vbo
        for y in tile.start_y..(tile.start_y + tile.height) {
            for x in tile.start_x..(tile.start_x + tile.width) {
                let pixel = self.src_img.get_pixel(x as _, y as _);
                let elevation: f32 = pixel.to_luma().data[0] as f32 / 255.0;

                let s = x as f32 / twidth;
                let t = y as f32 / theight;

                // TODO - clean this up, it's just moving the edges closer together,
                // not a proper stitch
                //
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
                    px,        // depth/elevation mapped to Y axis
                    elevation, // image height mapped to Y axis
                    pz,
                ));

                // Construct uv texture coordinates
                uvs.push(Point2::new(1.0 - s, 1.0 - t));

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

        Mesh::new(vertices, indices, Some(normals), Some(uvs), false)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        match e {
            _ => Error::InvalidFile,
        }
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Error {
        match e {
            _ => Error::FileNotSupported,
        }
    }
}
