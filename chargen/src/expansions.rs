pub trait Expansions {
    fn new_blank() -> Self;
    fn from_character(ch:char) -> Self;
    fn push_char(&mut self, ch:char);
    fn push_string(&mut self, string:&str);
    fn merge_strings(&mut self, strings:&[String]);
}
impl Expansions for Vec<String> {
    fn new_blank() -> Self {
        vec![ String::new() ]
    }

    fn from_character(ch:char) -> Self {
        vec![ String::from(ch) ]
    }

    fn push_char(&mut self, ch:char) {
        for expansion in self.iter_mut() {
            expansion.push(ch);
        }
    }

    fn push_string(&mut self, string:&str) {
        for expansion in self.iter_mut() {
            expansion.push_str(string);
        }
    }

    fn merge_strings(&mut self, strings:&[String]) {
        let mut new = self.drain(..)
            .map(|expansion| strings
                 .iter()
                 .map(move |extension| format!("{}{}", expansion, extension))
            )
            .flatten()
            .collect::<Vec<String>>();
        
        self.append(&mut new);
    }
}
