// https://github.com/sebcrozet/kiss3d/issues/130
// - split mesh into parts, add to MeshManager
// http://kiss3d.org/doc/kiss3d/resource/struct.MeshManager.html
//
// https://www.ncollide.org/mesh_generation/#mesh-generation
// https://docs.rs/ncollide/0.14.1/ncollide/procedural/struct.TriMesh.html

use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::resource::MeshManager;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, Vector3};
use std::env;

mod heightmap;

use crate::heightmap::Heightmap;

fn main() {
    let file_path = if let Some(p) = env::args().nth(1) {
        p
    } else {
        panic!("Expected a file path to heightmap.png");
    };

    let hmap = Heightmap::from_png_file(&file_path).expect("Failed to create Heightmap");

    let eye = Point3::new(0.0, 20.0, 0.0);
    let at = Point3::new(0.0, 0.0, 0.0);
    //let at = Point3::origin();
    let mut first_person_cam = FirstPerson::new(eye, at);
    //let mut first_person_cam = FirstPerson::new_with_frustrum(50.0, 1.0, 400.0,
    // eye, at);
    first_person_cam.set_move_step(10.0);

    let mut window = Window::new("Heli-X Scene3D Tool");
    window.set_light(Light::StickToCamera);

    let tiles = MeshManager::get_global_manager(|mm| hmap.populate_mesh_manager(mm));

    let mesh_scale = Vector3::new(1.0, 1.0, 1.0);
    for tile in tiles {
        let mut m = window
            .add_geom_with_name(tile.name(), mesh_scale)
            .expect("Failed to add mesh tile");

        m.set_color(1.0, 0.0, 0.0);
        //m.enable_backface_culling(false);
        //m.set_surface_rendering_activation(true);

        // Wireframe
        m.set_surface_rendering_activation(false);
        m.set_points_size(3.0);
        m.set_lines_width(1.0);
    }

    let mut c = window.add_cube(1.0, 1.0, 1.0);
    c.set_color(0.0, 0.0, 1.0);

    let mut b = window.add_cube(1.0, 1.0, 1.0);
    b.set_color(0.0, 1.0, 1.0);
    b.append_translation(&Translation3::new(-128.0, 0.0, 0.0));
    let mut b = window.add_cube(1.0, 1.0, 1.0);
    b.set_color(0.0, 1.0, 1.0);
    b.append_translation(&Translation3::new(-256.0, 0.0, 0.0));

    while window.render_with_camera(&mut first_person_cam) {}
}
