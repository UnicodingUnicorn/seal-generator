use random_color::RandomColor;

pub struct PointsGroup {
    points: Vec<Vec<f64>>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    area: f64,
}
impl PointsGroup {
    pub fn new(points:Vec<Vec<f64>>) -> Self {
        let (x_min, x_max, y_min, y_max) = points.iter().fold((f64::MAX, f64::MIN, f64::MAX, f64::MIN), |(x_min, x_max, y_min, y_max), p| {
            let x_min = x_min.min(p[0]);
            let x_max = x_max.max(p[0]);
            let y_min = y_min.min(p[1]);
            let y_max = y_max.max(p[1]);
            (x_min, x_max, y_min, y_max)
        });

        Self {
            points,
            x_min,
            x_max,
            y_min,
            y_max,
            area: (x_max - x_min).abs() * (y_max - y_min).abs(),
        }
    }

    pub fn area(&self) -> f64 {
        self.area
    }

    pub fn contains(&self, other:&Self) -> bool {
        self.x_min <= other.x_min && self.x_max >= other.x_max && self.y_min <= other.y_min && self.y_max >= other.y_max
    }

    pub fn into_points(self) -> Vec<Vec<f64>> {
        self.points
    }
}

pub struct Triangles {
    triangles: Vec<[(f64, f64); 3]>,
}
impl Triangles {
    pub fn from_points(points:&Vec<Vec<Vec<f64>>>) -> Result<Self, earcutr::Error> {
        let (vertices, holes, dimensions) = earcutr::flatten(points);
        let triangles = earcutr::earcut(&vertices, &holes, dimensions)?
            .array_chunks()
            .map(|tp:&[usize; 3]| tp.map(|i| (vertices[i * 2], vertices[i * 2 + 1])))
            .collect::<Vec<[(f64, f64); 3]>>();

        Ok(Self {
            triangles,
        })
    }

    pub fn svg(&self, colour:bool) -> String {
        self.triangles.iter()
            .map(|triangle| {
                let colour = match colour {
                    true => RandomColor::new().to_rgb_string(),
                    false => String::from("black"),
                };
                let path = triangle.iter().map(|(x, y)| format!("{},{} ", x, y)).collect::<String>();
                format!("<path fill=\"{}\" d=\"M{}Z\" />", colour, path)
            })
            .intersperse(String::from("\n"))
            .collect::<String>()
    }
}