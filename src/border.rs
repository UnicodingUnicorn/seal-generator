use crate::GeneratorError;
use crate::triangles::{ Line, Triangles };

fn square_path(x1: f32, y1: f32, x2:f32, y2:f32) -> String {
    format!("M{} {} {} {} {} {} {} {} {} {} Z", x1, y1, x2, y1, x2, y2, x1, y2, x1, y1)
}

fn centered_square_path(size:f32) -> String {
    square_path(-size / 2.0, -size / 2.0, size / 2.0, size / 2.0)
}

pub struct Border {
    outer_d: String,
    inner_d: String,
}
impl Border {
    pub fn new(size:f32, thickness: f32) -> Self {
        let outer_d = centered_square_path(size);
        let inner_d = centered_square_path(size - thickness * 2.0);

        Self {
            outer_d,
            inner_d,
        }
    }

    pub fn to_triangles(&self, resolution:u64) -> Result<Triangles, GeneratorError> {
        let (_, outer_points) = svg_path_parser::parse_with_resolution(&self.outer_d, resolution)
            .next().unwrap(); // Guaranteed to be there
        let outer_line = Line::new(outer_points.iter().map(|(x, y)| (*x as f32, *y as f32)).collect::<Vec<(f32, f32)>>());

        let (_, inner_points) = svg_path_parser::parse_with_resolution(&self.inner_d, resolution)
            .next().unwrap(); // Guaranteed to be there
        let inner_line = Line::new(inner_points.iter().map(|(x, y)| (*x as f32, *y as f32)).collect::<Vec<(f32, f32)>>());

        let lines = vec![outer_line, inner_line];
        Triangles::from_lines(&lines)
            .map_err(|e| GeneratorError::TriangulationError(e))
    }
}