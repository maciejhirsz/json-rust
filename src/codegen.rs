use JsonValue;

pub trait Generator {
    fn new_line(&mut self) {}

    fn write(&mut self, slice: &str);

    fn write_min(&mut self, slice: &str, minslice: &str);

    fn write_char(&mut self, ch: char);

    fn indent(&mut self) {}

    fn dedent(&mut self) {}

    fn write_json(&mut self, json: &JsonValue) {
        match *json {
            JsonValue::String(ref string) => {
                self.write_char('"');

                for ch in string.chars() {
                    match ch {
                        '\\' | '"' => {
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

    fn consume(self) -> String;
}

pub struct DumpGenerator {
    code: String,
}

impl DumpGenerator {
    pub fn new() -> Self {
        DumpGenerator {
            code: String::with_capacity(1024),
        }
    }
}

impl Generator for DumpGenerator {
    fn write(&mut self, slice: &str) {
        self.code.push_str(slice);
    }

    fn write_min(&mut self, _: &str, minslice: &str) {
        self.write(minslice);
    }

    fn write_char(&mut self, ch: char) {
        self.code.push(ch);
    }

    fn consume(self) -> String {
        self.code
    }
}

pub struct PrettyGenerator {
    code: String,
    dent: u16,
    spaces_per_indent: u16,
}

impl PrettyGenerator {
    pub fn new(spaces: u16) -> Self {
        PrettyGenerator {
            code: String::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces
        }
    }
}

impl Generator for PrettyGenerator {
    fn new_line(&mut self) {
        self.code.push('\n');
        for _ in 0..(self.dent * self.spaces_per_indent) {
            self.code.push(' ');
        }
    }

    fn write(&mut self, slice: &str) {
        self.code.push_str(slice);
    }

    fn write_min(&mut self, slice: &str, _: &str) {
        self.write(slice);
    }

    fn write_char(&mut self, ch: char) {
        self.code.push(ch);
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }

    fn consume(self) -> String {
        self.code
    }
}
