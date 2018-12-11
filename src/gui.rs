use crate::alphamap::Alphamap;
use crate::heightmap::Heightmap;
use kiss3d::camera::FirstPerson;
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::resource::{MeshManager, Texture, TextureManager};
use kiss3d::scene::{PlanarSceneNode, SceneNode};
use kiss3d::text::Font;
use kiss3d::window::Window;
use nalgebra::{Point2, Point3, Translation2, Translation3, Vector3};
use std::rc::Rc;

pub struct Gui {
    hmap: Heightmap,
    amap: Alphamap,
    win: Window,
    cam: FirstPerson,
    terrain_nodes: Vec<SceneNode>,
    terrain_mode: TerrainMode,
    height_scale: f32,
    height_offset: f32,
    default_texture: Rc<Texture>,
    hmap_src_rect: PlanarSceneNode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TerrainMode {
    /// Colored points and lines
    Wireframe,
    /// Colored points
    Points,
    /// Solid, colored material
    Solid,
    /// Solid, color provided by the heightmap image itself
    Textured,
    /// Solid, color provided by the alpha map channels
    Alphamap,
    /* TODO - need to load and blend the alphamap channels
     * Filled, color provided by the color channels in the alpha map texture
     * file
     * Channels? */
}

impl TerrainMode {
    pub fn next(&self) -> Self {
        match *self {
            TerrainMode::Wireframe => TerrainMode::Points,
            TerrainMode::Points => TerrainMode::Solid,
            TerrainMode::Solid => TerrainMode::Textured,
            TerrainMode::Textured => TerrainMode::Alphamap,
            TerrainMode::Alphamap => TerrainMode::Wireframe,
        }
    }
}

impl Gui {
    pub fn new(hmap: Heightmap, amap: Alphamap) -> Self {
        // TODO - mesh scale and offset configs
        let height_scale = 10.0;
        let height_offset = 0.0;
        let mesh_scale = Vector3::new(1.0, height_scale, 1.0);

        let mut win = Window::new("Heli-X Scene3D Tool");

        // TODO - which lighting is better?
        win.set_light(Light::StickToCamera);
        //win.set_light(Light::Absolute(Point3::new(0.0, 500.0, 0.0)));

        // Load textures
        TextureManager::get_global_manager(|tm| {
            tm.add_image(hmap.src_texture(), "heightmap");
            tm.add_image(amap.src_texture(), "alphamap_src");
        });

        // Load all of the GPU mesh tiles that make up the heightmap terrain
        let mut terrain_nodes = Vec::new();
        let tiles = MeshManager::get_global_manager(|mm| hmap.populate_mesh_manager(mm));
        for tile in tiles {
            let m = win
                .add_geom_with_name(tile.name(), mesh_scale)
                .expect("Failed to add mesh tile");
            terrain_nodes.push(m);
        }

        // TODO - toggle/size/location/etc
        let mut hmap_src_rect = win.add_rectangle(150.0, 150.0);
        let tx = win.width() as f32 / 2.0 - 400.0;
        let ty = win.height() as f32 / 2.0 - 300.0;
        hmap_src_rect.append_translation(&Translation2::new(tx, ty));
        hmap_src_rect.set_surface_rendering_activation(true);
        hmap_src_rect.set_points_size(0.0);
        hmap_src_rect.set_lines_width(0.0);
        hmap_src_rect.set_texture_with_name("heightmap");
        hmap_src_rect.modify_uvs(&mut |v| {
            v[0].x = 0.0;
            v[0].y = 0.0;
            v[1].x = 1.0;
            v[1].y = 1.0;
            v[2].x = 0.0;
            v[2].y = 1.0;
            v[3].x = 1.0;
            v[3].y = 0.0;
        });
        hmap_src_rect.set_visible(false);

        let mut gui = Self {
            hmap,
            amap,
            win,
            cam: FirstPerson::new(Point3::new(1.0, 1.0, 1.0), Point3::origin()),
            terrain_nodes,
            terrain_mode: TerrainMode::Textured,
            height_scale,
            height_offset,
            default_texture: TextureManager::get_global_manager(|tm| tm.get_default()),
            hmap_src_rect,
        };

        gui.set_terrain_mode(gui.terrain_mode.clone());
        gui.reset_camera();

        gui
    }

    pub fn render(&mut self) -> bool {
        let keep_rendering = self.win.render_with_camera(&mut self.cam);

        if keep_rendering {
            // TODO - break apart event handling
            for mut event in self.win.events().iter() {
                match event.value {
                    WindowEvent::Key(button, Action::Press, _) => {
                        if button == Key::Return {
                            self.reset_camera();
                        } else if button == Key::T {
                            self.set_terrain_mode(self.terrain_mode.next());
                        } else if button == Key::I {
                            self.set_terrain_height_scale(self.height_scale + 1.0);
                        } else if button == Key::K {
                            self.set_terrain_height_scale(self.height_scale - 1.0);
                        } else if button == Key::O {
                            self.set_terrain_height_offset(self.height_offset + 1.0);
                        } else if button == Key::L {
                            self.set_terrain_height_offset(self.height_offset - 1.0);
                        } else if button == Key::Y {
                            self.hmap_src_rect
                                .set_visible(!self.hmap_src_rect.is_visible());
                        }

                        // Override the default keyboard handler
                        event.inhibited = true
                    }
                    _ => {}
                }
            }

            self.render_scene_info_text();
        }

        keep_rendering
    }

    fn reset_camera(&mut self) {
        let eye = Point3::new(-20.0, 20.0, -20.0);
        let at = Point3::new(0.0, 0.0, 0.0);
        self.cam.look_at(eye, at);
        self.cam.set_move_step(1.0);

        // Rebind arrow key movement to ASDW keys
        self.cam.rebind_up_key(Some(Key::W));
        self.cam.rebind_down_key(Some(Key::S));
        self.cam.rebind_left_key(Some(Key::A));
        self.cam.rebind_right_key(Some(Key::D));
    }

    fn set_terrain_mode(&mut self, mode: TerrainMode) {
        // TODO - configs/lines/points/sizes/etc
        let point_size = 3.0;
        let line_width = 1.0;

        for mesh_tile in &mut self.terrain_nodes {
            match mode {
                TerrainMode::Wireframe => {
                    mesh_tile.set_color(1.0, 1.0, 1.0);
                    mesh_tile.enable_backface_culling(false);
                    mesh_tile.set_surface_rendering_activation(false);
                    mesh_tile.set_points_size(point_size);
                    mesh_tile.set_lines_width(line_width);
                    mesh_tile.set_material_with_name("object");
                    mesh_tile.set_texture(self.default_texture.clone());
                }
                TerrainMode::Points => {
                    mesh_tile.set_color(1.0, 1.0, 1.0);
                    mesh_tile.enable_backface_culling(false);
                    mesh_tile.set_surface_rendering_activation(false);
                    mesh_tile.set_points_size(point_size);
                    mesh_tile.set_lines_width(0.0);
                    mesh_tile.set_material_with_name("object");
                    mesh_tile.set_texture(self.default_texture.clone());
                }
                TerrainMode::Solid => {
                    mesh_tile.set_color(1.0, 1.0, 1.0);
                    mesh_tile.enable_backface_culling(true);
                    mesh_tile.set_surface_rendering_activation(true);
                    mesh_tile.set_points_size(0.0);
                    mesh_tile.set_lines_width(0.0);
                    mesh_tile.set_material_with_name("object");
                    mesh_tile.set_texture(self.default_texture.clone());
                }
                TerrainMode::Textured => {
                    mesh_tile.set_color(1.0, 1.0, 1.0);
                    mesh_tile.enable_backface_culling(true);
                    mesh_tile.set_surface_rendering_activation(true);
                    mesh_tile.set_points_size(0.0);
                    mesh_tile.set_lines_width(0.0);
                    mesh_tile.set_texture_with_name("heightmap");
                }
                TerrainMode::Alphamap => {
                    mesh_tile.set_color(1.0, 1.0, 1.0);
                    mesh_tile.enable_backface_culling(true);
                    mesh_tile.set_surface_rendering_activation(true);
                    mesh_tile.set_points_size(0.0);
                    mesh_tile.set_lines_width(0.0);
                    mesh_tile.set_texture_with_name("alphamap_src");
                }
            }
        }

        self.terrain_mode = mode;
    }

    fn set_terrain_height_scale(&mut self, scale: f32) {
        let scale = if scale < 1.0 { 1.0 } else { scale };

        for mesh_tile in &mut self.terrain_nodes {
            mesh_tile.set_local_scale(1.0, scale, 1.0);
        }

        self.height_scale = scale;
    }

    fn set_terrain_height_offset(&mut self, offset: f32) {
        for mesh_tile in &mut self.terrain_nodes {
            mesh_tile.set_local_translation(Translation3::new(0.0, offset, 0.0));
        }

        self.height_offset = offset;
    }

    fn render_scene_info_text(&mut self) {
        // TODO - configs
        let font_size = 35.0;
        let next_font = font_size + 5.0;
        let mut font_pos = Point2::new(10.0, 10.0);
        let font_color = Point3::new(1.0, 1.0, 0.0);

        self.win.draw_text(
            &format!("Terrain Mode: {:?}", self.terrain_mode),
            &font_pos,
            font_size,
            &Font::default(),
            &font_color,
        );

        font_pos.y += next_font;
        self.win.draw_text(
            &format!("Height Scale: {}", self.height_scale),
            &font_pos,
            font_size,
            &Font::default(),
            &font_color,
        );

        font_pos.y += next_font;
        self.win.draw_text(
            &format!("Height Offset: {}", self.height_offset),
            &font_pos,
            font_size,
            &Font::default(),
            &font_color,
        );
    }
}
