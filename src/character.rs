use crate::svgbuilder::SVGBuilder;
use ttf_parser::{ Face, Rect };

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

pub struct PositionedCharacter {
    d: String,
    translate_x: i16,
    translate_y: i16,
    scale: f64,
}
impl PositionedCharacter {
    pub fn new(d:&str, translate_x:i16, translate_y:i16, scale:f64) -> Self {
        Self {
            d: d.to_string(),
            translate_x,
            translate_y,
            scale,
        }
    }

    pub fn svg(&self) -> String {
        format!("<path fill=\"black\" transform=\"translate({} {}) scale(-{} -{})\" d=\"{}\" />", self.translate_x, self.translate_y, self.scale, self.scale, self.d)
    }
}

#[derive(Debug, Clone)]
pub struct Character {
    path: String,
    bbox: Rect,
}
impl Character {
    pub fn new(ch:char, font:&Face) -> Option<Self> {
        let id = font.glyph_index(ch)?;
        let mut builder = SVGBuilder::new();
        let bbox = font.outline_glyph(id, &mut builder)?;

        Some(Self {
            path: builder.into(),
            bbox,
        })
    }

    // TODO: Lifetimes for PositionedCharacter inheriting parent Character
    pub fn positioned(&self, position:&CharacterPosition) -> PositionedCharacter {
        let width:f64 = (self.bbox.x_max - self.bbox.x_min).abs().into();
        let height:f64 = (self.bbox.y_max - self.bbox.y_min).abs().into();

        let scaling_factor = match width > height {
            true => (position.width as f64) / width,
            false => (position.height as f64) / height,
        };

        PositionedCharacter::new(&self.path, position.x, position.y, scaling_factor)
    }
}
