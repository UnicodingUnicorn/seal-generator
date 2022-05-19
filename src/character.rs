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

    pub fn squares(side:i16) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(Self::square(side / 2, side / 2, 0));
        res.push(Self::square(side / 2, side / 2, side / 2));
        res.push(Self::square(side / 2, 0, 0));
        res.push(Self::square(side / 2, 0, side / 2));

        res
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

    pub fn svg(&self, position:&CharacterPosition) -> String {
        let width:f64 = (self.bbox.x_max - self.bbox.x_min).abs().into();
        let height:f64 = (self.bbox.y_max - self.bbox.y_min).abs().into();

        let scaling_factor = match width > height {
            true => (position.width as f64) / width,
            false => (position.height as f64) / height,
        };

        let correction_x = match width > height {
            true => 0.0,
            false => (height - width) / 2.0,
        } as i16;

        let correction_y = match width > height {
            true => (width - height) / 2.0,
            false => 0.0,
        } as i16;

        let scaled_x = ((position.x as f64) / (position.width as f64) * width).floor() as i16;
        let scaled_y = ((position.y as f64) / (position.height as f64) * height).floor() as i16;

        let translate_x = 0 - std::cmp::min(self.bbox.x_min, self.bbox.x_max) + scaled_x + correction_x;
        let translate_y = 0 - std::cmp::min(self.bbox.y_min, self.bbox.y_min) + scaled_y + correction_y;

        let transform = format!("transform=\"scale({}) translate({} {}) rotate(180)\"", scaling_factor, translate_x, translate_y);
        format!("<path fill=\"black\" {} d=\"{}\" />", transform, self.path)
    }
}
