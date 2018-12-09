// https://github.com/sebcrozet/kiss3d/issues/130
// - split mesh into parts, add to MeshManager
// http://kiss3d.org/doc/kiss3d/resource/struct.MeshManager.html
//
// https://www.ncollide.org/mesh_generation/#mesh-generation
// https://docs.rs/ncollide/0.14.1/ncollide/procedural/struct.TriMesh.html

use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::resource::Mesh;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, Vector3};
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::rc::Rc;

fn main() {
    let source = match env::args().nth(1) {
        Some(file_name) => File::open(file_name).unwrap(),
        None => File::open("heightmap.png").unwrap(),
    };

    let decoder = png::Decoder::new(source);
    let (info, mut reader) = decoder.read_info().unwrap();
    assert_eq!(
        info.color_type,
        png::ColorType::Grayscale,
        "Only grayscale is supported"
    );

    println!("info.width {}", info.width);
    println!("info.height {}", info.height);
    println!("info.color_type {:#?}", info.color_type);
    println!("info.bit_depth {:#?}", info.bit_depth);
    println!("info.line_size {}", info.line_size);

    // Allocate the output buffer.
    let mut buf: Vec<u8> = vec![0; info.buffer_size()];

    reader.next_frame(&mut buf).unwrap();

    // f32/f64/GLfloat?
    let mut coords: Vec<Point3<f32>> = vec![];

    // GLuint?, kiss3d switched to u16 to support webgl,
    // need to split into sub-meshes
    let mut indices: Vec<Point3<u16>> = vec![];

    let height_scale = 5_f32;
    let height_offset = 0_f32;

    let twidth = (info.width - 1) as f32;
    let theight = (info.height - 1) as f32;
    let half_twidth = twidth / 2_f32;
    let half_theight = theight / 2_f32;

    for y in 0..info.height {
        for x in 0..info.width {
            let index = ((y * info.width) + x) as usize;

            let elevation: f32 = buf[index] as f32 / 255.0;

            let s = x as f32 / twidth;
            let t = y as f32 / theight;

            coords.push(Point3::new(
                (s * twidth) - half_twidth,
                (t * theight) - half_theight,
                (elevation * height_scale) + height_offset,
            ));
        }
    }

    for y in 0..(info.height - 1) {
        for x in 0..(info.width - 1) {
            let index = ((y * info.width) + x) as usize;

            // top triangle T0 v0->v1->v2
            indices.push(Point3::new(
                index as _,
                index as u16 + info.width as u16 + 1,
                index as u16 + 1,
            ));

            // bottom triangle T1 v0->v1->v2
            indices.push(Point3::new(
                index as _,
                index as u16 + info.width as u16,
                index as u16 + info.width as u16 + 1,
            ));
        }
    }

    // TODO - normals?

    let mesh = Rc::new(RefCell::new(Mesh::new(coords, indices, None, None, false)));

    let eye = Point3::new(0.0, 0.0, 20.0);
    let at = Point3::origin();
    let mut first_person_cam = FirstPerson::new(eye, at);
    //let mut first_person_cam = FirstPerson::new_with_frustrum(50.0, 1.0, 400.0,
    // eye, at);
    first_person_cam.set_move_step(10.0);

    let mut window = Window::new("Heli-X Scene3D Tool");
    window.set_light(Light::StickToCamera);

    let mesh_scale = Vector3::new(1.0, 1.0, 1.0);
    //let mesh_scale = Vector3::new(10.0, 10.0, 10.0);
    let mut m = window.add_mesh(mesh, mesh_scale);
    m.set_color(1.0, 0.0, 0.0);
    //m.enable_backface_culling(false);
    //m.set_surface_rendering_activation(true);

    // Wireframe
    m.set_surface_rendering_activation(false);
    m.set_points_size(5.0);
    m.set_lines_width(1.0);

    let mut c = window.add_cube(2.0, 2.0, 2.0);
    c.set_color(0.0, 0.0, 1.0);

    let mut cb = window.add_cube(2.0, 2.0, 2.0);
    cb.set_color(0.0, 1.0, 1.0);
    cb.append_translation(&Translation3::new(64.0, 0.0, 0.0));

    //while window.render() {}
    while window.render_with_camera(&mut first_person_cam) {}
}
