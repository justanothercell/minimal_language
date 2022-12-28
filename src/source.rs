use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Read};
use std::rc::Rc;
use crate::tokens::tokens::{Literal, NumLit};


/* !! no clone !! */
#[derive(PartialEq)]
pub(crate) struct Source {
    st: SourceType,
    source: String
}

impl Debug for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source({:?})", self.st)
    }
}

impl Source {
    pub(crate) fn from_file(path: String) -> Result<Self, ParseError> {
        Ok(Self {
            st: SourceType::File(path.clone()),
            source: {
                let mut f = File::open(path.clone())?;
                let mut buffer = String::new();
                f.read_to_string(&mut buffer)?;
                fn include(path: String, mut file: String) -> String{
                    let dir = if let Some((p, n)) = path.rsplit_once("/") {
                        p
                    } else {
                        "."
                    };
                    let mut includes = vec![];
                    for line in file.lines() {
                        if line.starts_with("#include ") {
                            includes.push(line.split_at(9).1.to_string())
                        }
                    }
                    for incl in includes {
                        let (path, name) = if let Some((p, n)) = incl.rsplit_once("/") {
                            (p.to_string(), n.to_string())
                        } else {
                            (".".to_string(), incl.clone())
                        };
                        let include_file = format!("{}/{}/{}.mi", dir, path, name);
                        let mut f = File::open(include_file.clone()).expect(&format!("error including file: {include_file}"));
                        let mut buffer = String::new();
                        f.read_to_string(&mut buffer).unwrap();
                        buffer = include(include_file, buffer);
                        file = file.replace(&format!("#include {}", incl), &buffer)
                    }
                    file
                }
                include(path, buffer)
            }
        })
    }

    pub(crate) fn from_string(source: String) -> Self{
        Self {
            st: SourceType::String,
            source
        }
    }
}

pub(crate) struct SourceIter {
    source: Rc<Source>,
    pub(crate) index: usize,
}

impl SourceIter {
    pub(crate) fn new(source: Source) -> Self {
        Self {
            source: Rc::new(source),
            index: 0,
        }
    }

    pub(crate) fn get(&self, index: usize) -> Result<char, ParseError> {
        if index >= self.source.source.len() {
            Err(ParseET::EOF.at(self.here().span()).when("getting char"))
        }
        else {
            Ok(self.source.source.as_bytes()[index] as char)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.source.source.len()
    }

    pub(crate) fn left(&self) -> usize {
        self.source.source.len() - self.index
    }

    pub(crate) fn this(&self) -> Result<char, ParseError> {
        self.get(self.index)
    }

    pub(crate) fn here(&self) -> CodePoint {
        CodePoint(self.source.clone(), self.index)
    }

    pub(crate) fn next(&mut self){
        self.index += 1;
    }

    pub(crate) fn peek(&self) -> Result<char, ParseError>{
        self.get(self.index + 1)
    }

    pub(crate) fn peekn(&self, n: isize) -> Result<char, ParseError>{
        self.get((self.index as isize + n) as usize)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SourceType {
    File(String),
    String,
}

impl Display for SourceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SourceType::File(f) =>  format!("{}", f),
            SourceType::String => format!("<string>")
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CodePoint(Rc<Source>, usize);

#[allow(non_camel_case_types)]
type line = usize;
#[allow(non_camel_case_types)]
type index_in_line = usize;

impl CodePoint {
    pub(crate) fn span(self) -> Span {
        Span::single(self)
    }

    pub(crate) fn pos(&self) -> (line, index_in_line){
        let first_part = &self.0.source[0..self.1];
        let mut lines_split = first_part.split("\n").collect::<Vec<&str>>();
        (lines_split.len(), lines_split.pop().unwrap().len())
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct Span {
    pub(crate) source: Rc<Source>,
    pub(crate) start: usize,
    pub(crate) end: usize
}

impl Span {
    pub(crate) fn single(p: CodePoint) -> Self{
        Self {
            source: p.0,
            start: p.1,
            end: p.1
        }
    }

    pub(crate) fn from_points(a: CodePoint, b: CodePoint) -> Self{
        assert!(Rc::ptr_eq(&a.0, &b.0), "CodePoints should be of same Source");
        Self {
            source: a.0.clone(),
            start: usize::min(a.1, b.1),
            end: usize::max(a.1, b.1)
        }
    }

    pub(crate) fn bounds(&self) -> (CodePoint, CodePoint) {
        (CodePoint(self.source.clone(), self.start),
         CodePoint(self.source.clone(), self.end))
    }

    pub(crate) fn start(&self) -> CodePoint {
        CodePoint(self.source.clone(), self.start)
    }

    pub(crate) fn end(&self) -> CodePoint {
        CodePoint(self.source.clone(), self.end)
    }

    pub(crate) fn extend(&mut self, p: CodePoint) {
        assert!(Rc::ptr_eq(&self.source, &p.0), "CodePoint should be of same Source as Span");
        self.start = usize::min(self.start, p.1);
        self.end = usize::max(self.end, p.1);
    }

    pub(crate) fn render_span_code(&self, line_pad: usize) -> String {
        let (sl, sp) = self.start().pos();
        let (el, ep) = self.end().pos();
        let lines_split = &self.source.source.split("\n").collect::<Vec<&str>>();
        let mut render = vec![];
        for i in usize::max(sl.saturating_sub(line_pad), 1)..=usize::min(el+line_pad, lines_split.len()) {
            render.push(format!("{i:3} | {}", lines_split[i-1]));
            if i == sl && i == el {
                render.push(format!("    | {}{}", " ".repeat(sp), "^".repeat(ep - sp + 1)));
            }
            else if i == sl {
                render.push(format!("    | {}{}", " ".repeat(sp), "^".repeat(lines_split[i-1].len() - sp + 1)));
            }
            else if i == el {
                render.push(format!("    | {}{}", "^".repeat(ep + 1), " ".repeat(lines_split[i-1].len() - ep)));
            }
            else if i > sl && i < el {
                render.push(format!("    | {}", "^".repeat(lines_split[i-1].len())));
            }
        }
        render.join("\n")
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug)]
pub(crate) struct ParseError {
    et: ParseET,
    loc: Option<Span>,
    context: Vec<String>
}

impl ParseError {
    pub(crate) fn when(mut self, reason: &str) -> Self{
        self.context.push(reason.to_string());
        self
    }
    pub(crate) fn at(mut self, loc: Span) -> Self{
        self.loc = Some(loc);
        self
    }
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseET::IOError(error).error().when("doing IO operation")
    }
}

#[derive(Debug)]
pub(crate) enum ParseET {
    EOF,
    EmptyInput,
    IOError(std::io::Error),
    TokenizationError(String),
    ParseError(String, String),
    ParseLiteralError(Literal, String),
    VariableError(String),
}

impl ParseET {
    pub(crate) fn error(self) -> ParseError{
        ParseError {
            et: self,
            loc: None,
            context: vec![]
        }
    }
    pub(crate) fn at(self, loc: Span) -> ParseError{
        ParseError {
            et: self,
            loc: Some(loc),
            context: vec![]
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}",
               match &self.et {
                   ParseET::EOF => format!("Input error:\n    reached end of file"),
                   ParseET::EmptyInput => format!("Input error:\n    input was empty"),
                   ParseET::IOError(e) => format!("IO error:\n    {}", e),
                   ParseET::TokenizationError(e) => format!("Tokenization error:\n    {}", e),
                   ParseET::ParseError(expected, found) => format!("Parsing error:\n    expected {expected} found {found}"),
                   ParseET::ParseLiteralError(lit, e) => format!("{} literal parsing error:\n    {}", match lit {
                       Literal::String(_) => "String",
                       Literal::Char(_) => "Char",
                       Literal::Number(NumLit::Integer(_), _) => "Integer",
                       Literal::Number(NumLit::Float(_), _) => "Float",
                       Literal::Bool(_) => "Float",
                   }, e),
                   ParseET::VariableError(e) => format!("cant find variable:\n    {e}"),
               },
               if self.context.len() > 0 {
                   format!("\n    while {}", self.context.join("\n    while "))
               } else {
                   String::new()
               },
               if let Some(loc) = &self.loc {
                   format!("{}\n{}",
                       if loc.start == loc.end {
                           let (l, p) = loc.start().pos();
                           format!("\n\nat: {}: {}:{}", loc.source.st, l, p)
                       } else {
                           let (sl, sp) = loc.start().pos();
                           let (el, ep) = loc.end().pos();
                           format!("\n\nat: {}: {}:{}..{}:{}", loc.source.st, sl, sp, el, ep)
                       },
                       loc.render_span_code(2)
                   )
               } else {
                   String::new()
               },
        )
    }
}

pub(crate) trait OnParseErr{
    fn e_when(self, reason: String) -> Self;
    fn e_at(self, loc: Span) -> Self;
}

impl<T> OnParseErr for Result<T, ParseError> {
    fn e_when(self, reason: String) -> Self {
        self.map_err(|err| err.when(&reason))
    }

    fn e_at(self, loc: Span) -> Self {
        self.map_err(|err| err.at(loc))
    }
}


