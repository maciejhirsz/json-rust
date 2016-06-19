use JsonValue;

pub struct Generator<'a> {
    pub minify: bool,
    code: String,
    dent: u16,
    indent: &'a str,
}

impl<'a> Generator<'a> {
    pub fn new(indent: Option<&'a str>) -> Self {
        Generator {
            minify: indent.is_none(),
            code: String::new(),
            dent: 0,
            indent: indent.unwrap_or("")
        }
    }

    pub fn new_line(&mut self) {
        if !self.minify {
            self.code.push('\n');
            for _ in 0..(self.dent) {
                self.code.push_str(self.indent);
            }
        }
    }

    pub fn write_json(&mut self, json: &JsonValue) {
        match *json {
            JsonValue::String(ref string) => {
                self.write_char('"');

                for ch in string.chars() {
                    match ch {
                        '\\' | '/' | '"' => {
                            self.write_char('\\');
                            self.write_char(ch);
                        },
                        '\n'       => self.write("\\n"),
                        '\r'       => self.write("\\r"),
                        '\t'       => self.write("\\t"),
                        '\u{000C}' => self.write("\\f"),
                        '\u{0008}' => self.write("\\b"),
                        _          => self.write_char(ch)
                    }
                }

                self.write_char('"');
            },
            JsonValue::Number(ref number) => self.write(&number.to_string()),
            JsonValue::Boolean(ref value) => self.write(if *value { "true" } else { "false" }),
            JsonValue::Null               => self.write("null"),
            JsonValue::Array(ref array)   => {
                self.write_char('[');
                self.indent();
                let mut first = true;
                for item in array {
                    if first {
                        first = false;
                        self.new_line();
                    } else {
                        self.write(",");
                        self.new_line();
                    }
                    self.write_json(item);
                }
                self.dedent();
                self.new_line();
                self.write_char(']');
            },
            JsonValue::Object(ref object) => {
                self.write_char('{');
                self.indent();
                let mut first = true;
                for (key, value) in object.iter() {
                    if first {
                        first = false;
                        self.new_line();
                    } else {
                        self.write(",");
                        self.new_line();
                    }
                    self.write(&format!("{:?}", key));
                    self.write_min(": ", ":");
                    self.write_json(value);
                }
                self.dedent();
                self.new_line();
                self.write_char('}');
            }
        }
    }

    pub fn write(&mut self, slice: &str) {
        self.code.push_str(slice);
    }

    pub fn write_min(&mut self, slice: &str, minslice: &str) {
        if self.minify {
            self.write(minslice);
        } else {
            self.write(slice);
        }
    }

    pub fn write_char(&mut self, ch: char) {
        self.code.push(ch);
    }

    pub fn indent(&mut self) {
        self.dent += 1;
    }

    pub fn dedent(&mut self) {
        self.dent -= 1;
    }

    pub fn consume(self) -> String {
        self.code
    }
}
