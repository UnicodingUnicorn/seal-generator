use crate::GeneratorError;
use crate::svgbuilder::SVGBuilder;
use crate::triangles::{ Line, Triangles };
use ttf_parser::Face;

#[derive(Debug, Copy, Clone)]
pub struct CharacterPosition {
    width: i16,
    height: i16,
    x: i16,
    y: i16,
}
impl CharacterPosition {
    pub fn square(side:i16, x:i16, y:i16) -> Self {
        Self {
            width: side,
            height: side,
            x,
            y,
        }
    }

    pub fn squares(side:i16) -> [Self; 4] {
        [
            Self::square(side / 2, 0, 0),
            Self::square(side / 2, 0, side / 2),
            Self::square(side / 2, side / 2, 0),
            Self::square(side / 2, side / 2, side / 2)
        ]
    }
}

#[derive(Debug, Clone)]
pub struct PositionedCharacter {
    d: String,
    translate_x: f32,
    translate_y: f32,
    scale: f32,
}
impl PositionedCharacter {
    pub fn new(ch:char, font:&Face, position: &CharacterPosition) -> Option<Self> {
        let id = font.glyph_index(ch)?;
        let mut builder = SVGBuilder::new();
        let bbox = font.outline_glyph(id, &mut builder)?;

        let width:f32 = (bbox.x_max - bbox.x_min).abs().into();
        let height:f32 = (bbox.y_max - bbox.y_min).abs().into();

        let scale = match width > height {
            true => (position.width as f32) / width,
            false => (position.height as f32) / height,
        };

        Some(Self {
            d: builder.into(),
            translate_x: position.x.into(),
            translate_y: position.y.into(),
            scale,
        })
    }
    
    pub fn to_triangles(&self, resolution:u64) -> Result<Vec<Triangles>, GeneratorError> {
        // Convert points to a series of lines
        let mut lines = svg_path_parser::parse_with_resolution(&self.d, resolution)
            .map(|(_, points)| {
                let points = points.iter()
                    .map(|(x, y)| ((*x as f32) * -self.scale + self.translate_x, (*y as f32) * -self.scale + self.translate_y))
                    .collect::<Vec<(f32, f32)>>();
                Line::new(points)
            })
            .collect::<Vec<Line>>();

        // Sort it so the biggest shapes are first
        lines.sort_by(|a, b| a.area().partial_cmp(&b.area()).unwrap());

        // Separate standalone objects
        let mut groups:Vec<(Line, Vec<Line>)> = Vec::new();
        while let Some(line) = lines.pop() {
            if let Some((_, children)) = groups.iter_mut().find(|(g, _)| g.contains(&line)) {
                children.push(line);
            } else {
                groups.push((line, Vec::new()));
            }
        }
        
        // Consolidate everything into one array
        let objects = groups.drain(0..)
            .map(|(g, mut children)| vec![g]
                .drain(0..)
                .chain(children.drain(0..))
                .collect::<Vec<Line>>())
            .collect::<Vec<Vec<Line>>>();

        // Map to triangles
        objects.iter()
            .map(|object| Triangles::from_lines(object))
            .collect::<Result<Vec<Triangles>, earcutr::Error>>()
            .map_err(|e| GeneratorError::TriangulationError(e))
    }
}