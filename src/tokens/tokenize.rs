use std::str::FromStr;
use std::str::pattern::Pattern;
use std::u128;
use crate::source::{OnParseErr, ParseError, ParseET, Source, SourceIter, Span};
use crate::tokens::tokens::{Literal, NumLit, NumLitTy, Token, TokenType};

pub(crate) fn tokenize(source: Source) -> Result<Vec<Token>, ParseError>{
    let mut iter = SourceIter::new(source);
    let mut tokens = vec![];
    while iter.left() > 0 {
        match iter.this()? {
            '"' => {
                let (string, span) = collect_until(&mut iter, true, true,
                                                   |c| c.map(|c|c != '"').unwrap_or(false))
                    .e_when("tokenizing string literal".to_string())?;
                tokens.push(TokenType::Literal(Literal::String(string)).at(span));
            }
            '/' => {
                iter.next();
                let r: Result<(), ParseError> = try {
                    match iter.this()? {
                        '/' => {
                            let _comment = collect_until(&mut iter, true, true,
                                                         |c| c.map(|c|c != '\n').unwrap_or(false))
                                .e_when("tokenizing single line comment".to_string())?;
                        },
                        '*' => {
                            loop {
                                let _comment = collect_until(&mut iter, true, true,
                                                             |c| c.map(|c|c != '*').unwrap_or(false))
                                    .e_when("tokenizing single line comment".to_string())?;
                                iter.next();
                                if iter.this()? == '/' {
                                    break
                                }
                            }
                        }
                        _ => { // was just normal division slash or sth other
                            iter.index -= 1;
                            tokens.push(TokenType::Particle('/', if let Ok(t) = iter.peekn(-1) {
                                !(t.is_ascii_alphanumeric() || t == '_' || t == ' ')
                            } else {false}).at(iter.here().span()))
                        }
                    }
                };
                r.e_when(String::from("tokenizing comment"))?;
            }
            '\'' => {
                let (char_src, span) = collect_until(&mut iter, true, true,
                                                   |c| c.map(|c|c != '\'').unwrap_or(false))
                    .e_when("tokenizing char literal".to_string())?;
                if char_src.len() != 1 {
                    return Err(ParseET::TokenizationError(format!("Expected char, found: '{}'", char_src)).at(span))
                }
                let char = char_src.chars().nth(0).unwrap();
                tokens.push(TokenType::Literal(Literal::Char(char)).at(span));
            }
            c if c.is_whitespace() => {
                // pass
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let (ident, span) = collect_until(&mut iter, false, false,
                                                  |c| c.map(|c| c.is_ascii_alphanumeric() || c == '_').unwrap_or(false))
                                                      .e_when("tokenizing identifier".to_string())?;
                tokens.push(match ident {
                    ident if &ident == "true" => TokenType::Literal(Literal::Bool(true)),
                    ident if &ident == "false" => TokenType::Literal(Literal::Bool(false)),
                    ident => TokenType::Ident(ident)
                }.at(span));
            }
            c if c.is_ascii_digit() => {
                iter.index -= 1;
                let (num, span) = collect_until(&mut iter, true, true,
                                                   |c| c.map(|c| c.is_ascii_alphanumeric() || c == '_').unwrap_or(false))
                    .e_when("tokenizing number literal".to_string())?;
                iter.index -= 1;
                let (lit, ty) = str_to_num_lit(num).e_at(span.clone())?;
                tokens.push(TokenType::Literal(Literal::Number(lit, ty)).at(span));
            }
            c => tokens.push(TokenType::Particle(c, if let Ok(t) = iter.peekn(-1) {
                    !(t.is_ascii_alphanumeric() || t == '_' || t == ' ')
                } else {false}).at(iter.here().span()))
        }
        iter.next();
    }
    Ok(tokens)
}

fn collect_until(iter: &mut SourceIter, skip_first: bool, consume_break: bool, cond: fn(Option<char>) -> bool) -> Result<(String, Span), ParseError>{
    let start = iter.here();
    let mut result = String::new();
    if skip_first {
        iter.next();
    }
    while cond(iter.this().ok()) {
        result.push(iter.this()?);
        iter.next();
    }
    if consume_break {
        iter.next();
    }
    iter.index -= 1;
    Ok((result, Span::from_points(start, iter.here())))
}

pub(crate) fn str_to_num_lit(mut num: String) -> Result<(NumLit, Option<NumLitTy>), ParseError>{
    num = num.replace('_', "");
    let radix = if num.len() > 2 {
        if num.chars().nth(0).unwrap() == '0' {
            let r = match num.chars().nth(1).unwrap() {
                'b' => Some(0b10), // binary
                'q' => Some(4),    // quaternal
                'o' => Some(0o10), // octal
                'z' => Some(12),   // dozenal
                'x' => Some(0x10), // hexadecimal
                _ => None         // decimal (or invalid)
            };
            if let Some(r) = r {
                num.remove(0);
                num.remove(0);
                r
            } else { 10 }
        } else { 10 }
    } else { 10 };
    let float_like = num.contains('.');
    if float_like && radix != 10 {
        return Err(ParseET::ParseLiteralError(Literal::Number(NumLit::Float(0f64), None), format!("expected radix 10 for floating point literal, found {radix}")).error())
    }
    let mut float_like_ty = false;
    let ty = {
        let i = (|| {
            for (i, c) in num.chars().enumerate() {
                if c.is_numeric() || (float_like && c == '.') || (!float_like && c.is_contained_in("abcdefABCDEF") && (radix == 16 || && c != &&'f')) {
                    continue
                }
                return Some(i)
            }
            None
        })();
        if let Some(i) = i {
            let (n, t) = {
                let s = num.split_at(i);
                (s.0.to_string(), s.1.to_string())
            };
            num = n;
            let t = match t.as_str() {
                "u8" => NumLitTy::U8,
                "u16" => NumLitTy::U16,
                "u32" => NumLitTy::U32,
                "u64" => NumLitTy::U64,
                "u128" => NumLitTy::U128,
                "i8" => NumLitTy::I8,
                "i16" => NumLitTy::I16,
                "i32" => NumLitTy::I32,
                "i64" => NumLitTy::I64,
                "i128" => NumLitTy::I128,
                "f32" => { float_like_ty = true; NumLitTy::F32 },
                "f64" => { float_like_ty = true; NumLitTy::F64 },
                t => return Err(ParseET::ParseLiteralError(Literal::Number(if float_like {
                    NumLit::Float(0f64)
                } else {
                    NumLit::Integer(0)
                }, None), format!("unsupported type suffix: '{t}'")).error())
            };
            if float_like && !float_like_ty {
                return Err(ParseET::ParseLiteralError(Literal::Number(NumLit::Float(0f64), None), format!("expected floating point type for floating point literal, found '{t}'")).error())
            }
            Some(t)
        } else { None }
    };
    let lit = if float_like || float_like_ty {
        f64::from_str(&num).map(|f|NumLit::Float(f)).map_err(|_|
            ParseET::ParseLiteralError(Literal::Number(NumLit::Float(0f64), None), format!("invalid float literal")).error()
        )
    } else {
        u128::from_str_radix(&num, radix).map(|i|NumLit::Integer(i)).map_err(|_|
            ParseET::ParseLiteralError(Literal::Number(NumLit::Integer(0), None), format!("invalid integer literal")).error()
        )
    }?;
    Ok((lit, ty))
}