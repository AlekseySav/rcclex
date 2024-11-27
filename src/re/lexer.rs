#[derive(Debug, PartialEq)]
pub enum Token {
    Close(bool),                // eof or (
    Open,                       // )
    Union,                      // |
    StartGroup,                 // \A
    EndGroup,                   // \Z
    Repeat((u32, Option<u32>)), // + * ? {...}
    Char(Charset),
}

pub struct Lexer<'a> {
    it: std::slice::Iter<'a, u8>,
    peekc: Option<u8>,
    peek: Option<Token>,
    config: Config,
}

impl Lexer<'_> {
    pub fn new<'a>(s: &'a [u8], config: Config) -> Lexer<'a> {
        Lexer {
            it: s.iter(),
            peekc: None,
            peek: None,
            config,
        }
    }

    pub fn token(&mut self) -> Result<Token> {
        if self.peek.is_some() {
            return Ok(core::mem::replace(&mut self.peek, None).unwrap());
        }
        match self.char() {
            Some(b'(') => match self.config.auto_groups {
                true => {
                    self.peek = Some(Token::Open);
                    Ok(Token::StartGroup)
                }
                false => Ok(Token::Open),
            },
            Some(b')') => {
                if self.config.auto_groups {
                    self.peek = Some(Token::EndGroup);
                }
                Ok(Token::Close(false))
            }
            Some(b'|') => Ok(Token::Union),
            Some(b'.') => Ok(Token::Char(self.config.dot_charset.clone())),
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

    fn repeat(&mut self) -> Result<Token> {
        let min = self.atoi(10)?;
        let min_int = min.unwrap_or(0) as u32;
        match self.char() {
            Some(b',') => (),
            Some(b'}') if min != None && min != Some(0) => {
                return Ok(Token::Repeat((min_int, Some(min_int))))
            }
            _ => return Err(Error::Repeat),
        };
        let max = self.atoi(10)?;
        if self.char() != Some(b'}') || max == Some(0) || min_int > max.unwrap_or(255) as u32 {
            return Err(Error::Repeat);
        }
        Ok(Token::Repeat((
            min_int,
            match max {
                Some(c) => Some(c as u32),
                None => None,
            },
        )))
    }

    fn charset(&mut self) -> Result<Charset> {
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
                None => return Err(Error::Charset),
                Some(b']') if !s.empty() => return Ok(if inv { s.inv() } else { s }),
                Some(b'\\') => {
                    let p = self.char_escape()?;
                    prev = p.iter().next();
                    s.add(&p);
                }
                Some(b'-') if prev != None => {
                    let end = match self.char() {
                        Some(b'\\') => self.char_escape()?.iter().next_back().unwrap(),
                        Some(b']') => {
                            self.peekc = Some(b']');
                            s.add_char(b'-');
                            continue;
                        }
                        Some(c) => c,
                        None => return Err(Error::Charset),
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

    fn char_escape(&mut self) -> Result<Charset> {
        match self.escape()? {
            Token::Char(c) => Ok(c),
            _ => Err(Error::Escape),
        }
    }

    fn escape(&mut self) -> Result<Token> {
        match self.char() {
            None => Err(Error::Escape),
            Some(b'A') => Ok(Token::StartGroup),
            Some(b'Z') => Ok(Token::EndGroup),
            Some(b'x') | Some(b'X') => match self.atoi(16)? {
                None => Err(Error::Escape),
                Some(c) => Ok(Token::Char(charset!(c))),
            },
            Some(c) => match self.config.esc_charset.get(&c) {
                None => Ok(Token::Char(charset!(c))),
                Some(c) => return Ok(Token::Char(c.clone())),
            },
        }
    }

    fn atoi(&mut self, base: u8) -> Result<Option<u8>> {
        let mut res: Option<u8> = None;
        loop {
            match self.char() {
                Some(c) if Self::digit(c) < base => {
                    let r = res.unwrap_or(0);
                    if r as usize * base as usize > 255 {
                        return Err(Error::Overflow);
                    }
                    res = Some(r * base + Self::digit(c))
                }
                Some(c) => {
                    self.peekc = Some(c);
                    return Ok(res);
                }
                None => return Ok(res),
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

    fn lexer(s: &[u8]) -> Lexer {
        return Lexer::new(s, Config::default());
    }

    fn onetok(s: &[u8]) -> Result<Token> {
        lexer(s).token()
    }

    #[test]
    fn tokens() {
        let mut lex =
            lexer(b"()\\A\\Z{18}{1,59}{9,}{,8}*+?|[]].\\x00\\x7f\\x9\\s\\S\\d\\D\\w\\W\\t\\n");
        let ans = [
            Token::Open,
            Token::Close(false),
            Token::StartGroup,
            Token::EndGroup,
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
        let mut lex = lexer(&v);
        for c in 0..128 {
            assert_eq!(lex.token().unwrap(), Token::Char(charset!(c)));
        }
        assert_eq!(lex.token().unwrap(), Token::Close(true));
    }

    #[test]
    fn charset() {
        let mut lex = lexer(b"[][\\]][^]][\\d][^-\\x05][\\s-\\d][--][abc\\n]");
        let ans = [
            Token::Char(charset!(b'[', b']')),
            Token::Char(charset!(b']').inv()),
            Token::Char(charset!([b'0', b'9'])),
            Token::Char(charset!(b'-', 5).inv()),
            Token::Char(charset!([b'\t', b'9'])),
            Token::Char(charset!(b'-')),
            Token::Char(charset!([b'a', b'c']; b'\n')),
            Token::Close(true),
        ];
        for a in ans {
            assert_eq!(lex.token().unwrap(), a);
        }
    }

    #[test]
    fn errors() {
        assert_eq!(onetok(b"\\").unwrap_err(), Error::Escape);
        assert_eq!(onetok(b"[").unwrap_err(), Error::Charset);
        assert_eq!(onetok(b"[hello").unwrap_err(), Error::Charset);
        assert_eq!(onetok(b"[i-").unwrap_err(), Error::Charset);
        assert_eq!(onetok(b"{").unwrap_err(), Error::Repeat);
        assert_eq!(onetok(b"{a").unwrap_err(), Error::Repeat);
        assert_eq!(onetok(b"{}").unwrap_err(), Error::Repeat);
        assert_eq!(onetok(b"{0,0}").unwrap_err(), Error::Repeat);
        assert_eq!(onetok(b"\\xq").unwrap_err(), Error::Escape);
        assert_eq!(onetok(b"{a").unwrap_err(), Error::Repeat);
    }

    #[test]
    fn auto_groups() {
        let mut config = Config::default();
        config.auto_groups = true;
        let mut lex = Lexer::new(b"()", config);
        assert_eq!(lex.token(), Ok(Token::StartGroup));
        assert_eq!(lex.token(), Ok(Token::Open));
        assert_eq!(lex.token(), Ok(Token::Close(false)));
        assert_eq!(lex.token(), Ok(Token::EndGroup));
        assert_eq!(lex.token(), Ok(Token::Close(true)));
    }
}
