pub type Expansions = Vec<String>;
pub trait ExpansionsMethods {
    fn new() -> Self;
    fn push_char(&mut self, ch:char);
    fn push_string(&mut self, string:&str);
    fn merge_strings(&mut self, strings:&[String]);
}
impl ExpansionsMethods for Expansions {
    fn new() -> Self {
        vec![ String::from("a") ]
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
        let mut new = vec![];
        for expansion in self.iter() {
            for extension in strings.iter() {
                new.push(format!("{}{}", expansion, extension));
            }
        }

        *self = new;
    }
}
