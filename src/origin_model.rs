use kiss3d::camera::Camera;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::{Point2, Point3, Translation3, UnitQuaternion, Vector2, Vector3};
use std::f32;

pub struct OriginModel {
    cube: SceneNode,
    loc: Point3<f32>,
    screen_size: Vector2<f32>,
    screen_point: Point2<f32>,
}

impl OriginModel {
    pub fn new(win: &mut Window) -> Self {
        let mut cube = win.add_cube(1.0, 1.0, 1.0);
        cube.set_color(0.77, 1.0, 0.22);
        cube.set_surface_rendering_activation(true);
        cube.set_points_size(0.0);
        cube.set_lines_width(0.0);
        cube.set_visible(true);

        let axis_line_len = 3.0;
        let axis_line_radius = 0.25;
        let mut x_line = cube.add_cylinder(axis_line_radius, axis_line_len);
        x_line.set_color(0.0, 0.0, 1.0);
        x_line.set_local_rotation(UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            f32::consts::FRAC_PI_2,
        ));
        x_line.set_local_translation(Translation3::new(axis_line_len / 2.0, 0.0, 0.0));

        let mut z_line = cube.add_cylinder(axis_line_radius, axis_line_len);
        z_line.set_color(1.0, 0.0, 0.0);
        z_line.set_local_rotation(UnitQuaternion::from_axis_angle(
            &Vector3::x_axis(),
            f32::consts::FRAC_PI_2,
        ));
        z_line.set_local_translation(Translation3::new(0.0, 0.0, axis_line_len / 2.0));

        Self {
            cube,
            // Make the cube rest on the surface
            loc: Point3::new(0.0, 0.5, 0.0),
            screen_size: Vector2::new(win.width() as _, win.height() as _),
            screen_point: Point2::new(0.0, 0.0),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.cube.is_visible()
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.cube.set_visible(visible);
    }

    pub fn position(&self) -> &Point3<f32> {
        &self.loc
    }

    pub fn set_static_height(&mut self, h: f32) {
        self.loc.y = h;
        self.cube.set_local_translation(self.translation());
    }

    pub fn set_screen_size<T: Camera>(&mut self, cam: &T, w: f32, h: f32) {
        self.screen_size.x = w;
        self.screen_size.y = h;

        self.recompute_loc(cam);
    }

    pub fn set_position<T: Camera>(&mut self, cam: &T, x: f32, y: f32) {
        self.screen_point.x = x;
        self.screen_point.y = y;

        self.recompute_loc(cam);
    }

    fn recompute_loc<T: Camera>(&mut self, cam: &T) {
        let (rp, rv) = cam.unproject(&self.screen_point, &self.screen_size);

        // Plane normal
        let pn = Vector3::new(0.0, 1.0, 0.0);
        // Point on the plane
        let pp = Vector3::new(1.0, 0.0, 1.0);

        if let Some(v) = ground_plane_intersection(&rv, &rp.coords, &pn, &pp) {
            self.loc.coords.x = v.x;
            self.loc.coords.z = v.z;
            self.cube.set_local_translation(self.translation());
        }
    }

    fn translation(&self) -> Translation3<f32> {
        Translation3::new(self.loc.x, self.loc.y, self.loc.z)
    }
}

fn ground_plane_intersection(
    rv: &Vector3<f32>,
    rp: &Vector3<f32>,
    pn: &Vector3<f32>,
    pp: &Vector3<f32>,
) -> Option<Vector3<f32>> {
    if pn.dot(rv) == 0.0 {
        // Does not intersect
        None
    } else {
        let diff = rp - pp;
        let prod1 = diff.dot(pn);
        let prod2 = rv.dot(pn);
        let prod3 = prod1 / prod2;
        Some(rp - (rv * prod3))
    }
}
