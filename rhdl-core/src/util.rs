use std::fmt::Display;

pub fn splice<T: Display>(elems: &[T], sep: &str) -> String {
    elems
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}

#[derive(Default, Debug)]
pub struct IndentingFormatter {
    buffer: String,
    indent: i32,
}

impl IndentingFormatter {
    pub fn buffer(self) -> String {
        self.buffer
    }
    pub fn location(&self) -> usize {
        self.buffer.len()
    }
    pub fn write(&mut self, s: &str) {
        // Write s to the internal buffer.
        // If s contains a left brace, then increase the indent
        // if s contains a right brace, then decrease the indent
        // if s contains a newline, then add the indent
        // if s contains a semicolon, then add a newline
        // otherwise, just write the string
        for c in s.chars() {
            match c {
                '{' => {
                    self.buffer.push(c);
                    self.indent += 1;
                }
                '}' => {
                    let backup = self
                        .buffer
                        .chars()
                        .rev()
                        .take_while(|x| *x == ' ')
                        .take(3)
                        .count();
                    self.buffer.truncate(self.buffer.len() - backup);
                    self.indent -= 1;
                    self.buffer.push(c);
                }
                '\n' => {
                    self.buffer.push(c);
                    for _ in 0..self.indent {
                        self.buffer.push_str("   ");
                    }
                }
                _ => {
                    self.buffer.push(c);
                }
            }
        }
    }
}

#[test]
fn test_indenting_formatter() {
    let mut f = IndentingFormatter::default();
    f.write("hello {\n");
    f.write("let a = 2;\n");
    f.write("let b = 3;\n");
    f.write("}\n");
    println!("{}", f.buffer());
}

pub fn binary_string(x: &[bool]) -> String {
    x.iter().rev().map(|b| if *b { '1' } else { '0' }).collect()
}
