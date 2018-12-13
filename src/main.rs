use std::env;
use std::path::Path;

mod alphamap;
mod ground_plane;
mod gui;
mod heightmap;
mod origin_model;
mod ortho_view;
mod terrain_mode;

use crate::alphamap::Alphamap;
use crate::gui::Gui;
use crate::heightmap::Heightmap;

fn main() {
    // TODO - add cli/opts lib to handle args/etc
    // - point to a src image file, construct the rest
    let app_name = env::args()
        .nth(0)
        .unwrap_or(String::from("heli-x-scene3d-tool"));
    let usage = format!("Usage: {} /path/to/project/res/", app_name);
    let resource_root: String = if let Some(p) = env::args().nth(1) {
        p
    } else {
        panic!("Invalid arguments\n{}", usage);
    };
    let resource_root_path = Path::new(&resource_root);

    if !resource_root_path.exists() {
        panic!(
            "Path {} does not exist\n{}",
            resource_root_path.display(),
            usage
        );
    }

    if !resource_root_path.is_dir() {
        panic!(
            "Path {} is not a directory\n{}",
            resource_root_path.display(),
            usage
        );
    }

    let hmap_file = resource_root_path.join("heightmap.png");
    let amap_file = resource_root_path.join("alphamap.png");
    let some_amap_file = if amap_file.exists() {
        Some(&amap_file)
    } else {
        None
    };

    let hmap = Heightmap::from_png_file(&hmap_file).expect("Failed to create Heightmap");

    let (dw, dh) = hmap.dimensions();
    let amap = Alphamap::from_png_file(some_amap_file, dw, dh).expect("Failed to create Alphamap");

    let mut gui = Gui::new(hmap, amap);

    while gui.render() {}
}
