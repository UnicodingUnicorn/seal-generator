use std::fmt::Write;
use ttf_parser::OutlineBuilder;

pub struct SVGBuilder(String);
impl SVGBuilder {
    pub fn new() -> Self {
        Self(String::new())
    }
}

impl OutlineBuilder for SVGBuilder {
    fn move_to(&mut self, x:f32, y:f32) {
        write!(&mut self.0, "M {} {}", x, y).unwrap();
    }

    fn line_to(&mut self, x:f32, y:f32) {
        write!(&mut self.0, "L {} {}", x, y).unwrap();
    }

    fn quad_to(&mut self, x1:f32, y1:f32, x:f32, y:f32) {
        write!(&mut self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
    }

    fn curve_to(&mut self, x1:f32, y1:f32, x2:f32, y2:f32, x:f32, y:f32) {
        write!(&mut self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap();
    }

    fn close(&mut self) {
        write!(&mut self.0, "Z ").unwrap();
    }
}

impl Into<String> for SVGBuilder {
    fn into(self) -> String {
        self.0
    }
}
