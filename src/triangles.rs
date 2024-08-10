use crate::stl::Triangle;

#[derive(Debug, Clone)]
pub struct Line {
    points: Vec<(f32, f32)>,
    clockwise: bool,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    area: f32,
}
impl Line {
    pub fn new(points:Vec<(f32, f32)>) -> Self {
        let (x_min, x_max, y_min, y_max, angle) = points.iter().fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN, 0.0), |(x_min, x_max, y_min, y_max, angle), (x, y)| {
            let x_min = x_min.min(*x);
            let x_max = x_max.max(*x);
            let y_min = y_min.min(*y);
            let y_max = y_max.max(*y);
            (x_min, x_max, y_min, y_max, angle + (*y / *x).atan())
        });

        Self {
            points,
            clockwise: angle >= 0.0,
            x_min,
            x_max,
            y_min,
            y_max,
            area: (x_max - x_min).abs() * (y_max - y_min).abs(),
        }
    }

    pub fn area(&self) -> f32 {
        self.area
    }

    pub fn contains(&self, other:&Self) -> bool {
        self.x_min <= other.x_min && self.x_max >= other.x_max && self.y_min <= other.y_min && self.y_max >= other.y_max
    }

    pub fn size(&self) -> usize {
        self.points.len()
    }

    pub fn is_clockwise(&self) -> bool {
        self.clockwise
    }

    pub fn points(&self) -> Vec<f32> {
        self.points.iter().map(|(x, y)| [*x, *y]).flatten().collect::<Vec<f32>>()
    }
}

pub struct Triangles {
    lines: Vec<Line>, // Necessary for extrusion
    triangles: Vec<[(f32, f32); 3]>,
}
impl Triangles {
    pub fn from_lines(lines:&Vec<Line>) -> Result<Self, earcutr::Error> {
        let mut vertices = Vec::new();
        let mut holes = Vec::new();
        let mut i = 0;
        for line in lines {
            if i != 0 {
                holes.push(i);
            }

            vertices.append(&mut line.points());
            i += line.size();
        }

        let triangles = earcutr::earcut(&vertices, &holes, 2)?
            .array_chunks()
            .map(|tp:&[usize; 3]| tp.map(|i| (vertices[i * 2], vertices[i * 2 + 1])))
            .collect::<Vec<[(f32, f32); 3]>>();

        Ok(Self {
            lines: lines.to_vec(),
            triangles,
        })
    }

    // Centred on 0.0, 0.0
    pub fn square(side:f32) -> Result<Self, earcutr::Error> {
        let (p1, p2) = (-side / 2.0, side / 2.0);
        let line = Line::new(vec![(p1, p1), (p2, p1), (p2, p2), (p1, p2)]);
        Self::from_lines(&vec![line])
    }

    pub fn extrude(&self, height:f32, offset:f32) -> Vec<Triangle> {
        // Flat triangles
        let mut triangles = self.triangles.iter()
            .map(|triangle| [
                Triangle::from_2d_points((0.0, 0.0, -1.0).into(), triangle, offset),
                Triangle::from_2d_points((0.0, 0.0, 1.0).into(), triangle, offset + height),
            ])
            .flatten()
            .collect::<Vec<Triangle>>();

        // Side triangles
        let mut t2 = self.lines.iter()
            .map(|line| line.points.windows(2).map(|p| {
                let (x1, y1) = match line.is_clockwise() {
                    true => p[0],
                    false => p[1],
                };
                let (x2, y2) = match line.is_clockwise() {
                    true => p[1],
                    false => p[0],
                };

                [
                    Triangle::from_vertices(
                        (x1, y1, offset).into(),
                        (x2, y2, offset).into(),
                        (x1, y1, height + offset).into(),
                    ),
                    Triangle::from_vertices(
                        (x1, y1, height + offset).into(),
                        (x2, y2, height + offset).into(),
                        (x2, y2, offset).into(),
                    ),
                ]
            }))
            .flatten().flatten()
            .collect::<Vec<Triangle>>();
        
        triangles.append(&mut t2);
        triangles
    }
}