use crate::ChargenError;
use crate::characters::Characters;
use crate::utils;
use crate::svg_data::SVGData;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::Chars;
use usvg::{ Align, AspectRatio, Group, Node, NodeKind, NormalizedValue, Options, Rect, Size, Svg, Transform, Tree, ViewBox };

pub trait IdeographicDescriptionCharacter {
    fn chars_required(&self) -> usize;
    fn consume(&self, chars:&[Rc<SVGData>]) -> Result<Vec<Transform>, ChargenError>;
}

pub struct IDC<'a> {
    size: Size,
    viewbox: ViewBox,
    idcs: HashMap<char, Box<dyn IdeographicDescriptionCharacter>>,
    characters: &'a Characters,
}
impl<'a> IDC<'a> {
    pub fn new(width:f64, height:f64, characters:&'a Characters) -> Result<Self, ChargenError> {
        match Self::optioned_new(width, height, characters) {
            Some(idc) => Ok(idc),
            None => Err(ChargenError::InvalidWidthAndHeight),
        }
    }

    fn optioned_new(width:f64, height:f64, characters:&'a Characters) -> Option<Self> {
        let rect = Rect::new(0.0, 0.0, width, height)?;
        let aspect = AspectRatio {
            slice: false,
            align: Align::None,
            defer: false,
        };

        let viewbox = ViewBox {
            rect,
            aspect,
        };

        let size = Size::new(width, height)?;

        let mut idcs:HashMap<char, Box<dyn IdeographicDescriptionCharacter>> = HashMap::new();
        idcs.insert('⿰', Box::new(LeftToRight::new()));

        Some(Self {
            size,
            viewbox,
            idcs,
            characters,
        })
    }

    pub fn get_chars_required(&self, idc:char) -> usize {
        match self.idcs.get(&idc) {
            Some(idc) => idc.chars_required(),
            None => 0,
        }
    }

    pub fn get_svg_data(&self, ch:char) -> Option<&SVGData> {
        self.characters.get(ch)
    }

    pub fn consume(&self, idc:char, chars:&mut Vec<Rc<SVGData>>) -> Result<SVGData, ChargenError> {
        let idc = match self.idcs.get(&idc) {
            Some(idc) => idc,
            None => return Err(ChargenError::IDCUnsupported(idc)),
        };

        let svg = Svg {
            size: self.size,
            view_box: self.viewbox,
        };
        let master_tree = Tree::create(svg);

        let transforms = idc.consume(chars)?;
        for transform in transforms {
            let data = match chars.pop() {
                Some(data) => data,
                None => return Err(ChargenError::IDCNotEnough),
            };

            let g = Group {
                id: String::new(),
                transform,
                opacity: NormalizedValue::new(1.0),
                clip_path: None,
                mask: None,
                filter: None,
                filter_fill: None,
                filter_stroke: None,
                enable_background: None,
            };

            let mut node = Node::new(NodeKind::Group(g));
            let tree = Tree::from_str(data.svg(), &Options::default())?;
            for child in tree.root().children() {
                node.append(child);
            }

            master_tree.root().append(node);
        }

        Ok(SVGData::new(&master_tree))
    }
}

fn reverse_index<'a>(index:usize, chars:&[Rc<SVGData>]) -> Result<Rc<SVGData>, ChargenError> {
    if chars.len() < index + 1 {
        return Err(ChargenError::IDCNotEnough);
    }

    Ok(chars[chars.len() - index - 1].clone())
}

pub struct IDCer<'a> {
    idc: &'a IDC<'a>,
    chars: Chars<'a>,
    stack: Vec<Rc<SVGData>>,
}
impl<'a> IDCer<'a> {
    pub fn new(idc:&'a IDC<'a>, mapping:&'a str) -> Self {
        Self {
            idc,
            chars: mapping.chars(),
            stack: vec![],
        }
    }

    fn step(&mut self) -> Result<bool, ChargenError> {
        let ch = match self.chars.next() {
            Some(ch) => ch,
            None => return Ok(false),
        };

        if !utils::is_ids_char(ch) {
            let ch = match self.idc.get_svg_data(ch) {
                Some(ch) => ch,
                None => return Err(ChargenError::Unsupported(ch)),
            };

            self.stack.push(Rc::new(ch.clone()));
        } else {
            for _ in 0..self.idc.get_chars_required(ch) {
                if !self.step()? {
                    return Err(ChargenError::IDCNotEnough);
                }
            }

            let data = self.idc.consume(ch, &mut self.stack)?;
            self.stack.push(Rc::new(data));
        }

        Ok(true)
    }
}
impl<'a> Iterator for IDCer<'a> {
    type Item = Result<Rc<SVGData>, ChargenError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.step() {
            Ok(true) => match self.stack.len() {
                1 => Some(Ok(self.stack[0].clone())),
                _ => Some(Err(ChargenError::IDCIncompleteReduction)),
            },
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct LeftToRight {}
impl LeftToRight {
    pub fn new() -> Self {
        Self {}
    }
}
impl IdeographicDescriptionCharacter for LeftToRight {
    fn chars_required(&self) -> usize {
        2
    }

    fn consume(&self, chars:&[Rc<SVGData>]) -> Result<Vec<Transform>, ChargenError> {
        let b = reverse_index(0, chars)?;
        let a = reverse_index(1, chars)?;
        let (a_scale, b_scale) = a.scale_y(&b);

        let mut a_transform = Transform::new_translate(-a.bounding_box().x * a_scale, -a.bounding_box().y * a_scale);
        let mut b_transform = Transform::new_translate(-b.bounding_box().x * b_scale + a.bounding_box().w * a_scale, -b.bounding_box().y * b_scale);

        a_transform.scale(a_scale, a_scale);
        b_transform.scale(b_scale, b_scale);

        Ok(vec![b_transform, a_transform])
    }
}
