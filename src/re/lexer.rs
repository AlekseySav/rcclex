#[derive(Debug, PartialEq)]
pub enum Token {
    Close(bool),                // eof or (
    Open,                       // )
    Union,                      // |
    Group,                      // \Z
    Repeat((u32, Option<u32>)), // + * ? {...}
    Char(Charset),
}

pub struct Lexer<'a> {
    it: std::slice::Iter<'a, u8>,
    peekc: Option<u8>,
}

impl Lexer<'_> {
    pub fn new<'a>(s: &'a [u8]) -> Lexer<'a> {
        return Lexer {
            it: s.iter(),
            peekc: None,
        };
    }

    pub fn token(&mut self) -> RegexResult<Token> {
        match self.char() {
            Some(b'|') => Ok(Token::Union),
            Some(b'(') => Ok(Token::Open),
            Some(b')') => Ok(Token::Close(false)),
            Some(b'.') => Ok(Token::Char(Charset::ALL)),
            Some(b'*') => Ok(Token::Repeat((0, None))),
            Some(b'+') => Ok(Token::Repeat((1, None))),
            Some(b'?') => Ok(Token::Repeat((0, Some(1)))),
            Some(b'[') => Ok(Token::Char(self.charset()?)),
            Some(b'{') => self.repeat(),
            Some(b'\\') => self.escape(),
            Some(c) => Ok(Token::Char(charset!(c))),
            None => Ok(Token::Close(true)),
        }
    }

    fn char(&mut self) -> Option<u8> {
        match self.peekc {
            Some(c) => {
                self.peekc = None;
                Some(c)
            }
            None => self.it.next().copied(),
        }
    }

    fn repeat(&mut self) -> RegexResult<Token> {
        let min = self.atoi(10);
        let min_int = min.unwrap_or(0) as u32;
        match self.char() {
            Some(b',') => (),
            Some(b'}') if min != None => return Ok(Token::Repeat((min_int, Some(min_int)))),
            _ => return Err(RegexError::Repeat),
        };
        let max = self.atoi(10);
        if self.char() != Some(b'}') || max == Some(0) || min_int > max.unwrap_or(255) as u32 {
            return Err(RegexError::Repeat);
        }
        Ok(Token::Repeat((
            min_int,
            match max {
                Some(c) => Some(c as u32),
                None => None,
            },
        )))
    }

    fn charset(&mut self) -> RegexResult<Charset> {
        let mut s = charset!();
        let mut prev: Option<u8> = None;
        let inv = match self.char() {
            Some(b'^') => true,
            c => {
                self.peekc = c;
                false
            }
        };
        loop {
            match self.char() {
                None => return Err(RegexError::Charset),
                Some(b']') if !s.empty() => return Ok(if inv { s.inv() } else { s }),
                Some(b'\\') => {
                    let p = self.char_escape()?;
                    prev = p.iter().next();
                    s.add(&p);
                }
                Some(b'-') if prev != None => {
                    let end = match self.char() {
                        Some(b'\\') => self.char_escape()?.iter().next_back().unwrap(),
                        Some(c) => c,
                        None => return Err(RegexError::Charset),
                    };
                    let begin = prev.unwrap();
                    prev = None;
                    s.add_range(begin, end);
                }
                Some(c) => {
                    prev = Some(c);
                    s.add_char(c);
                }
            };
        }
    }

    fn char_escape(&mut self) -> RegexResult<Charset> {
        match self.escape()? {
            Token::Char(c) => Ok(c),
            _ => Err(RegexError::Escape),
        }
    }

    fn escape(&mut self) -> RegexResult<Token> {
        let c = self.char();
        match c {
            None => Err(RegexError::Escape),
            Some(b'Z') => Ok(Token::Group),
            Some(b't') => Ok(Token::Char(charset!(b'\t'))),
            Some(b'n') => Ok(Token::Char(charset!(b'\n'))),
            Some(b's') => Ok(Token::Char(charset!(b' ', b'\t', b'\r', b'\n'))),
            Some(b'S') => Ok(Token::Char(charset!(b' ', b'\t', b'\r', b'\n').inv())),
            Some(b'd') => Ok(Token::Char(charset!([b'0', b'9']))),
            Some(b'D') => Ok(Token::Char(charset!([b'0', b'9']).inv())),
            Some(b'w') => Ok(Token::Char(
                charset!([b'A', b'Z'], [b'a', b'z'], [b'0', b'9']; b'_'),
            )),
            Some(b'W') => Ok(Token::Char(
                charset!([b'A', b'Z'], [b'a', b'z'], [b'0', b'9']; b'_').inv(),
            )),
            Some(b'x') | Some(b'X') => match self.atoi(16) {
                None => Err(RegexError::Escape),
                Some(c) => Ok(Token::Char(charset!(c))),
            },
            Some(c) => Ok(Token::Char(charset!(c))),
        }
    }

    fn atoi(&mut self, base: u8) -> Option<u8> {
        let mut res: Option<u8> = None;
        loop {
            match self.char() {
                Some(c) if Self::digit(c) < base => {
                    res = Some(res.unwrap_or(0) * base + Self::digit(c))
                }
                Some(c) => {
                    self.peekc = Some(c);
                    return res;
                }
                None => return res,
            };
        }
    }

    fn digit(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'F' => c - b'A' + 10,
            b'a'..=b'f' => c - b'a' + 10,
            _ => 255,
        }
    }
}

#[cfg(test)]
mod test_lexer {
    use super::*;

    fn onetok(s: &[u8]) -> RegexResult<Token> {
        Lexer::new(s).token()
    }

    #[test]
    fn tokens() {
        let mut lex =
            Lexer::new(b"()\\Z{18}{1,59}{9,}{,8}*+?|[]].\\x00\\x7f\\x9\\s\\S\\d\\D\\w\\W\\t\\n");
        let ans = [
            Token::Open,
            Token::Close(false),
            Token::Group,
            Token::Repeat((18, Some(18))),
            Token::Repeat((1, Some(59))),
            Token::Repeat((9, None)),
            Token::Repeat((0, Some(8))),
            Token::Repeat((0, None)),
            Token::Repeat((1, None)),
            Token::Repeat((0, Some(1))),
            Token::Union,
            Token::Char(charset!(b']')),
            Token::Char(Charset::ALL),
            Token::Char(charset!(0)),
            Token::Char(charset!(0x7f)),
            Token::Char(charset!(0x9)),
            Token::Char(charset!(b'\n', b'\t', b' ', b'\r')),
            Token::Char(charset!(b'\n', b'\t', b' ', b'\r').inv()),
            Token::Char(charset!([b'0', b'9'])),
            Token::Char(charset!([b'0', b'9']).inv()),
            Token::Char(charset!([b'0', b'9'], [b'A', b'Z'], [b'a', b'z']; b'_')),
            Token::Char(charset!([b'0', b'9'], [b'A', b'Z'], [b'a', b'z']; b'_').inv()),
            Token::Char(charset!(b'\t')),
            Token::Char(charset!(b'\n')),
            Token::Close(true),
        ];
        for a in ans {
            assert_eq!(lex.token().unwrap(), a);
        }

        let mut v: Vec<u8> = vec![];
        for c in 0..128 {
            match c {
                b'(' | b')' | b'{' | b'*' | b'+' | b'?' | b'|' | b'[' | b'.' | b'\\' => {
                    v.push(b'\\');
                    v.push(c);
                }
                _ => v.push(c),
            }
        }
        let mut lex = Lexer::new(&v);
        for c in 0..128 {
            assert_eq!(lex.token().unwrap(), Token::Char(charset!(c)));
        }
        assert_eq!(lex.token().unwrap(), Token::Close(true));
    }

    #[test]
    fn charset() {
        let mut lex = Lexer::new(b"[][\\]][^]][\\d][^-\\x05][\\s-\\d][--]abc\\n]");
        let ans = [
            Token::Char(charset!(b'[', b']')),
            Token::Char(charset!(b']').inv()),
            Token::Char(charset!([b'0', b'9'])),
            Token::Char(charset!(b'-', 5).inv()),
            Token::Char(charset!([b'\t', b'9'])),
            Token::Char(charset!([b'-', b']'], [b'a', b'c']; b'\n')),
            Token::Close(true),
        ];
        for a in ans {
            assert_eq!(lex.token().unwrap(), a);
        }
    }

    #[test]
    fn errors() {
        assert_eq!(onetok(b"\\").unwrap_err(), RegexError::Escape);
        assert_eq!(onetok(b"[").unwrap_err(), RegexError::Charset);
        assert_eq!(onetok(b"[hello").unwrap_err(), RegexError::Charset);
        assert_eq!(onetok(b"[i-").unwrap_err(), RegexError::Charset);
        assert_eq!(onetok(b"{").unwrap_err(), RegexError::Repeat);
        assert_eq!(onetok(b"{a").unwrap_err(), RegexError::Repeat);
        assert_eq!(onetok(b"{}").unwrap_err(), RegexError::Repeat);
        assert_eq!(onetok(b"{0,0}").unwrap_err(), RegexError::Repeat);
        assert_eq!(onetok(b"\\xq").unwrap_err(), RegexError::Escape);
        assert_eq!(onetok(b"{a").unwrap_err(), RegexError::Repeat);
    }
}
