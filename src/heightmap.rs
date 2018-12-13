// TODO - verify channel depth, hard-coded to 8 bits

use crate::terrain_mode::TerrainMode;
use image::{DynamicImage, GenericImage, ImageError, Pixel};
use kiss3d::resource::{Mesh, MeshManager, TextureManager};
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::{normalize, Point2, Point3, Translation3, Vector3};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidFile,
    FileNotSupported,
}

pub struct Heightmap {
    src_path: PathBuf,
    src_img: DynamicImage,
    width: usize,
    height: usize,
    height_scale: f32,
    height_offset: f32,
    terrain_mode: TerrainMode,
    tiles: Vec<Tile>,
}

const TILE_SIZE: usize = 128;

pub struct Tile {
    name: String,
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
    mesh_node: Option<SceneNode>,
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

        Ok(Self {
            src_path: file_path.to_path_buf(),
            src_img,
            width: src_width as _,
            height: src_height as _,
            height_scale: 10.0,
            height_offset: 0.0,
            terrain_mode: TerrainMode::Textured,
            tiles: Vec::new(),
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

    pub fn height_scale(&self) -> f32 {
        self.height_scale
    }

    pub fn height_offset(&self) -> f32 {
        self.height_offset
    }

    pub fn set_height_scale(&mut self, scale: f32) {
        let scale = if scale < 1.0 { 1.0 } else { scale };

        for tile in &mut self.tiles {
            if let Some(ref mut node) = tile.mesh_node {
                node.set_local_scale(1.0, scale, 1.0);
            }
        }

        self.height_scale = scale;
    }

    pub fn set_height_offset(&mut self, offset: f32) {
        for tile in &mut self.tiles {
            if let Some(ref mut node) = tile.mesh_node {
                node.set_local_translation(Translation3::new(0.0, offset, 0.0));
            }
        }

        self.height_offset = offset;
    }

    pub fn terrain_mode(&self) -> TerrainMode {
        self.terrain_mode
    }

    pub fn set_terrain_mode(&mut self, mode: TerrainMode) {
        // TODO - configs/lines/points/sizes/etc
        let point_size = 3.0;
        let line_width = 1.0;
        let default_texture = TextureManager::get_global_manager(|tm| tm.get_default());
        self.terrain_mode = mode;

        for tile in &mut self.tiles {
            if let Some(ref mut node) = tile.mesh_node {
                match mode {
                    TerrainMode::Wireframe => {
                        node.set_color(1.0, 1.0, 1.0);
                        node.enable_backface_culling(false);
                        node.set_surface_rendering_activation(false);
                        node.set_points_size(point_size);
                        node.set_lines_width(line_width);
                        node.set_material_with_name("object");
                        node.set_texture(default_texture.clone());
                    }
                    TerrainMode::Points => {
                        node.set_color(1.0, 1.0, 1.0);
                        node.enable_backface_culling(false);
                        node.set_surface_rendering_activation(false);
                        node.set_points_size(point_size);
                        node.set_lines_width(0.0);
                        node.set_material_with_name("object");
                        node.set_texture(default_texture.clone());
                    }
                    TerrainMode::Solid => {
                        node.set_color(1.0, 1.0, 1.0);
                        node.enable_backface_culling(true);
                        node.set_surface_rendering_activation(true);
                        node.set_points_size(0.0);
                        node.set_lines_width(0.0);
                        node.set_material_with_name("object");
                        node.set_texture(default_texture.clone());
                    }
                    TerrainMode::Textured => {
                        node.set_color(1.0, 1.0, 1.0);
                        node.enable_backface_culling(true);
                        node.set_surface_rendering_activation(true);
                        node.set_points_size(0.0);
                        node.set_lines_width(0.0);
                        node.set_texture_with_name("heightmap");
                    }
                    TerrainMode::Alphamap => {
                        node.set_color(1.0, 1.0, 1.0);
                        node.enable_backface_culling(true);
                        node.set_surface_rendering_activation(true);
                        node.set_points_size(0.0);
                        node.set_lines_width(0.0);
                        node.set_texture_with_name("alphamap_src");
                    }
                }
            }
        }
    }

    /// Add the generated mesh tiles to a window
    pub fn create_mesh_tile_scene_nodes(&mut self, win: &mut Window) {
        assert_ne!(
            self.tiles.len(),
            0,
            "Mesh tiles have not been generated yet?"
        );
        let mesh_scale = Vector3::new(1.0, self.height_scale, 1.0);

        for tile in &mut self.tiles {
            let node = win
                .add_geom_with_name(tile.name(), mesh_scale)
                .expect("Failed to add mesh tile");
            tile.mesh_node = Some(node);
        }

        // Set the initial terrain mode
        self.set_terrain_mode(self.terrain_mode);
    }

    /// Generate mesh tiles from the heightmap image
    pub fn generate_mesh_tiles(&mut self, mm: &mut MeshManager) {
        assert_eq!(self.tiles.len(), 0, "Should only call this once");

        // Split up the grid into TILE_SIZE x TILE_SIZE meshes
        let num_tiles_x = self.width / TILE_SIZE;
        let num_tiles_y = self.height / TILE_SIZE;

        for ty in 0..num_tiles_y {
            for tx in 0..num_tiles_x {
                // Create tile meta data
                let tile = Tile {
                    name: format!("{} {}", tx, ty),
                    start_x: (tx * TILE_SIZE),
                    start_y: (ty * TILE_SIZE),
                    width: TILE_SIZE,
                    height: TILE_SIZE,
                    mesh_node: None,
                };

                // Generate mesh from tile parameters
                let mesh = self.generate_mesh(&tile);
                let mesh = Rc::new(RefCell::new(mesh));

                // Add mesh to the global mesh manager
                mm.add(mesh.clone(), tile.name());

                self.tiles.push(tile);
            }
        }
    }

    pub fn reload(&mut self, mm: &mut MeshManager) {
        // Reload source image
        let new_img = if let Ok(img) = image::open(&self.src_path) {
            img
        } else {
            println!(
                "Failed to reload source heightmap image {}",
                self.src_path.display()
            );
            return;
        };

        // TODO - WIP, not working yet
        self.src_img = new_img;

        // Regenerate mesh tiles
        for tile in &self.tiles {
            if let Some(_) = mm.get(tile.name()) {
                // Remove old mesh
                mm.remove(tile.name());

                // TODO - need to update the texture too

                let new_mesh = self.generate_mesh(tile);
                let new_mesh = Rc::new(RefCell::new(new_mesh));

                // Add mesh to the global mesh manager
                mm.add(new_mesh.clone(), tile.name());
            }
        }

        // Reset terrain mode
        self.set_terrain_mode(self.terrain_mode);
    }

    fn generate_mesh(&self, tile: &Tile) -> Mesh {
        let mut vertices: Vec<Point3<f32>> = vec![];
        let mut normals: Vec<Vector3<f32>> = vec![];
        let mut indices: Vec<Point3<u16>> = vec![];
        let mut uvs: Vec<Point2<f32>> = vec![];

        self.generate_mesh_vectors(tile, &mut vertices, &mut normals, &mut indices, &mut uvs);

        Mesh::new(vertices, indices, Some(normals), Some(uvs), false)
    }

    // For each quad in the mesh, generate two triangles
    // Each triangle has three indices
    fn generate_mesh_vectors(
        &self,
        tile: &Tile,
        vertices: &mut Vec<Point3<f32>>,
        normals: &mut Vec<Vector3<f32>>,
        indices: &mut Vec<Point3<u16>>,
        uvs: &mut Vec<Point2<f32>>,
    ) {
        let twidth = (self.width - 1) as f32;
        let theight = (self.height - 1) as f32;
        let half_twidth = twidth / 2.0;
        let half_theight = theight / 2.0;

        // Generate the vertices for the vbo
        for y in tile.start_y..(tile.start_y + tile.height) {
            for x in tile.start_x..(tile.start_x + tile.width) {
                // Invert x/y to align uvs/vertices
                let pixel = self.src_img.get_pixel(
                    (self.src_img.width() - 1) - x as u32,
                    (self.src_img.height() - 1) - y as u32,
                );
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

                // Align coordinate frame, Y-up
                // Image width mapped to X axis
                // Depth/elevation mapped to Y axis
                // Image height mapped to Y axis
                vertices.push(Point3::new(px, elevation, pz));

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
