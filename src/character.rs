use crate::svgbuilder::SVGBuilder;
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
    translate_x: f64,
    translate_y: f64,
    scale: f64,
}
impl PositionedCharacter {
    pub fn new(ch:char, font:&Face, position: &CharacterPosition) -> Option<Self> {
        let id = font.glyph_index(ch)?;
        let mut builder = SVGBuilder::new();
        let bbox = font.outline_glyph(id, &mut builder)?;

        let width:f64 = (bbox.x_max - bbox.x_min).abs().into();
        let height:f64 = (bbox.y_max - bbox.y_min).abs().into();

        let scale = match width > height {
            true => (position.width as f64) / width,
            false => (position.height as f64) / height,
        };

        Some(Self {
            d: builder.into(),
            translate_x: position.x.into(),
            translate_y: position.y.into(),
            scale,
        })
    }
    
    // pub fn svg(&self) -> String {
    //     format!("<path fill=\"black\" transform=\"translate({} {}) scale(-{} -{})\" d=\"{}\" />", self.translate_x, self.translate_y, self.scale, self.scale, self.d)
    // }

    pub fn to_points(&self, resolution:u64) -> Vec<Vec<Vec<f64>>> {
        svg_path_parser::parse_with_resolution(&self.d, resolution)
            .map(|(_, points)| points.iter()
                .map(|(x, y)| vec![*x * -self.scale + self.translate_x, *y * -self.scale + self.translate_y])
                .collect::<Vec<Vec<f64>>>())
            .collect::<Vec<Vec<Vec<f64>>>>()
    }
}