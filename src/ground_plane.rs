use kiss3d::window::Window;
use nalgebra::Point3;

pub struct GroundPlane {
    visible: bool,
    color: Point3<f32>,
    lines: Vec<Line>,
}

struct Line {
    a: Point3<f32>,
    b: Point3<f32>,
}

impl GroundPlane {
    pub fn new(length: usize, delta: usize) -> Self {
        let half_length = length as f32 / 2.0;

        let mut lines = Vec::new();
        for r in (0..(length / 2)).step_by(delta) {
            lines.push(Line {
                a: Point3::new(r as _, 0.0, half_length),
                b: Point3::new(r as _, 0.0, -half_length),
            });

            if r != 0 {
                lines.push(Line {
                    a: Point3::new(-(r as f32), 0.0, half_length),
                    b: Point3::new(-(r as f32), 0.0, -half_length),
                });
            }

            lines.push(Line {
                a: Point3::new(half_length, 0.0, r as _),
                b: Point3::new(-half_length, 0.0, r as _),
            });

            if r != 0 {
                lines.push(Line {
                    a: Point3::new(half_length, 0.0, -(r as f32)),
                    b: Point3::new(-half_length, 0.0, -(r as f32)),
                });
            }
        }

        Self {
            visible: true,
            color: Point3::new(0.2, 0.2, 0.2),
            lines,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn draw(&self, win: &mut Window) {
        if self.is_visible() {
            for l in &self.lines {
                win.draw_line(&l.a, &l.b, &self.color);
            }
        }
    }
}
