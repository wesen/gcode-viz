use crate::helpers::PopIf;

pub enum DisplayLine<'a> {
    Comment(gcode::Comment<'a>),
    GCode(String, gcode::GCode),
}

pub struct LineIterator<'input, I>
where
    I: Iterator<Item = gcode::Line<'input>>,
{
    s: I,
    current_line: Option<gcode::Line<'input>>,
    comments: Vec<gcode::Comment<'input>>,
    gcodes: Vec<gcode::GCode>,
}

impl<'input, I> LineIterator<'input, I>
where
    I: Iterator<Item = gcode::Line<'input>>,
{
    pub fn new(mut lines: I) -> Self {
        let mut res = LineIterator {
            s: lines,
            current_line: None,
            comments: Vec::new(),
            gcodes: Vec::new(),
        };
        res.next_line();

        res
    }

    fn next_line(&mut self) {
        self.current_line = self.s.next();
        if let Some(s) = &self.current_line {
            self.comments.extend(s.comments().iter().cloned());
            self.gcodes.extend(s.gcodes().iter().cloned());
        }
    }
}

impl<'input, I> Iterator for LineIterator<'input, I>
where
    I: Iterator<Item = gcode::Line<'input>> + 'input,
{
    type Item = DisplayLine<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(l) = &self.current_line {
            let first_gcode_line: usize = self
                .gcodes
                .get(0)
                .and_then(|x| Some(x.span().line))
                .unwrap_or(0);

            // if there are still comments for previous lines, emit
            if let Some(x) = self.comments.pop_if(|c| c.span.line <= first_gcode_line) {
                return Some(DisplayLine::Comment(x.clone()));
            }

            // emit all gcodes buffered up
            if let Some(i) = self.gcodes.pop() {
                let opcode = match (i.mnemonic(), i.major_number(), i.minor_number()) {
                    (m, major, 0) => format!("{}{}", m, major),
                    (m, major, minor) => format!("{}{}.{}", m, major, minor),
                };
                return Some(DisplayLine::GCode(opcode, i.clone()));
            }

            // all gcodes emitted, get next line
            self.next_line();
        }

        None
    }
}
