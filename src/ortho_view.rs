// TODO - some marker inidcating view/camera position and orientation

use kiss3d::scene::PlanarSceneNode;
use kiss3d::window::Window;
use nalgebra::{Point2, Point3, Translation2, UnitComplex, Vector2, Vector3};
use std::f32;

pub struct OrthoView {
    size: Point2<f32>,
    scale: Point2<f32>,
    hmap_rect: PlanarSceneNode,
    origin_rect: PlanarSceneNode,
    cam_geom: PlanarSceneNode,
    //loc: Translation2,
}

impl OrthoView {
    pub fn new(win: &mut Window, size: Point2<f32>, scale: Point2<f32>) -> Self {
        // TODO - configs, check texture coordinates
        let mut hmap_rect = win.add_rectangle(size.x, size.y);
        hmap_rect.append_translation(&Translation2::new(0.0, 0.0));
        hmap_rect.set_surface_rendering_activation(true);
        hmap_rect.set_points_size(0.0);
        hmap_rect.set_lines_width(0.0);
        hmap_rect.set_texture_with_name("heightmap");
        hmap_rect.modify_uvs(&mut |v| {
            v[0].x = 1.0;
            v[0].y = 0.0;
            v[1].x = 0.0;
            v[1].y = 1.0;
            v[2].x = 0.0;
            v[2].y = 0.0;
            v[3].x = 1.0;
            v[3].y = 1.0;
        });
        hmap_rect.set_visible(true);

        let ratio_x = 1.0 / scale.x;
        let ratio_y = 1.0 / scale.y;

        let mut origin_rect = hmap_rect.add_rectangle(10.0 * ratio_x, 10.0 * ratio_y);
        origin_rect.set_color(0.77, 1.0, 0.22);
        origin_rect.set_surface_rendering_activation(true);
        origin_rect.set_points_size(1.0);
        origin_rect.set_lines_width(1.0);
        origin_rect.set_visible(true);

        let dist = 20.0;
        let points: Vec<Point2<f32>> = vec![
            Point2::new(-dist * ratio_x, 0.7 * dist * ratio_y),
            Point2::new(0.0, 0.0),
            Point2::new(dist * ratio_x, 0.7 * dist * ratio_y),
        ];
        let mut cam_geom = hmap_rect.add_convex_polygon(points, Vector2::new(1.0, 1.0));
        cam_geom.set_color(0.0, 0.0, 1.0);
        cam_geom.set_surface_rendering_activation(true);
        cam_geom.set_points_size(1.0);
        cam_geom.set_lines_width(1.0);
        cam_geom.set_visible(true);

        Self {
            size,
            scale,
            hmap_rect,
            origin_rect,
            cam_geom,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.hmap_rect.is_visible()
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.hmap_rect.set_visible(visible);
        self.origin_rect.set_visible(visible);
        self.cam_geom.set_visible(visible);
    }

    pub fn set_origin_position(&mut self, p: &Point3<f32>) {
        let (x, y) = self.constrained_scale_xy(p.x, p.z);
        self.origin_rect
            .set_local_translation(Translation2::new(x, y));
    }

    pub fn set_cam_position(&mut self, p: &Point3<f32>) {
        let (x, y) = self.constrained_scale_xy(p.x, p.z);
        self.cam_geom.set_local_translation(Translation2::new(x, y));
    }

    pub fn set_cam_orientation(&mut self, eye_dir: &Vector3<f32>) {
        let angle = eye_dir.z.atan2(eye_dir.x) - f32::consts::FRAC_PI_2;
        self.cam_geom.set_local_rotation(UnitComplex::new(-angle));
    }

    fn constrained_scale_xy(&self, x: f32, y: f32) -> (f32, f32) {
        let mx = x * (self.size.x / self.scale.x);
        let my = y * (self.size.y / self.scale.y);

        let hx = (self.size.x / self.scale.x) * (self.scale.x / 2.0);
        let hy = (self.size.y / self.scale.y) * (self.scale.y / 2.0);

        (constrain(-mx, -hx, hx), constrain(my, -hy, hy))
    }
}

fn constrain(v: f32, min: f32, max: f32) -> f32 {
    if v <= min {
        min
    } else if v >= max {
        max
    } else {
        v
    }
}
