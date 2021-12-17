use std::path::Path;
use serde::{ Deserialize, Serialize };
use usvg::{ self, NodeExt, Tree, Options, XmlOptions };

// Basically cos usvg::Rect cannot be deserialised
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
impl Rect {
    pub fn new(x:f64, y:f64, w:f64, h:f64) -> Self {
        Self {
            x,
            y,
            w,
            h,
        }
    }

    pub fn coordinates(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.x + self.w, self.y + self.h)
    }
}
impl From<usvg::Rect> for Rect {
    fn from(r:usvg::Rect) -> Self {
        Self::new(r.x(), r.y(), r.width(), r.height())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SVGData {
    svg_string: String,
    bounding_box: Rect,
}
impl SVGData {
    pub fn from_file<P: AsRef<Path>>(filepath:P) -> Result<Self, usvg::Error> {
        let tree = Tree::from_file(filepath, &Options::default())?;
        Ok(Self::new(&tree))
    }

    pub fn new(tree:&Tree) -> Self {
        let svg_string = tree.to_string(XmlOptions::default());
        let bounding_box = tree.root()
            .calculate_bbox()
            .map_or_else(|| Rect::new(0.0, 0.0, 0.0, 0.0), |r| Rect::from(r));

        Self {
            svg_string,
            bounding_box,
        }
    }

    pub fn bounding_box(&self) -> Rect {
        self.bounding_box
    }

    pub fn svg(&self) -> &str {
        &self.svg_string
    }

    pub fn scale_y(&self, b:&Self) -> (f64, f64) {
        if self.bounding_box().h < b.bounding_box().h {
            (b.bounding_box().h / self.bounding_box().h, 1.0)
        } else {
            (1.0, self.bounding_box().h / b.bounding_box().h)
        }
    }
}
