/*
 * API:
 * - parset_charset()
 * - lexer.token()
 */

pub type LexerResult<T> = std::result::Result<T, LexerError>;

#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    BadChar(u8),
    BadEof,
    BadCharset,
    BadExpr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Op(u8),
    Char(Charset),
}

pub struct Lexer<'a> {
    iter: std::slice::Iter<'a, u8>,
    charset: Charset,
    peekc: u8,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadChar(c) => write!(f, "character not in charset: '{}'", Charset::char(*c)),
            Self::BadCharset => write!(f, "bad charset syntax"),
            Self::BadEof => write!(f, "unexpected (eof)"),
            Self::BadExpr => write!(f, "bad expression syntax"),
        }
    }
}

impl Lexer<'_> {
    pub fn token(&mut self) -> LexerResult<Option<Token>> {
        let c = self.char();
        match c {
            0 => Ok(None),
            b'(' | b')' | b'*' | b'+' | b'|' => Ok(Some(Token::Op(c))),
            b'.' => Ok(Some(Token::Char(self.charset))),
            b'[' => {
                let s = self.getcharset()?;
                Ok(Some(self.char_token(s)?))
            }
            _ => {
                self.peekc = c;
                let s = self.getchar()?;
                Ok(Some(self.char_token(s)?))
            }
        }
    }

    fn char(&mut self) -> u8 {
        let c = self.peekc;
        if c != 0 {
            self.peekc = 0;
            return c;
        }
        return *self.iter.next().or(Some(&0)).unwrap();
    }

    fn char_token(&self, set: Charset) -> LexerResult<Token> {
        let bad = set & self.charset.invert();
        if !bad.empty() {
            return Err(LexerError::BadChar(bad.chars().next().unwrap()));
        }
        return Ok(Token::Char(set));
    }

    fn getchar(&mut self) -> LexerResult<Charset> {
        match self.char() {
            0 => Err(LexerError::BadEof),
            b'\\' => self.escaped(),
            c => Ok(Charset::char(c)),
        }
    }

    fn getcharset(&mut self) -> LexerResult<Charset> {
        let invert = match self.char() {
            b'^' => true,
            c => {
                self.peekc = c;
                false
            }
        };

        let mut set = Charset::new();
        let mut prev = Charset::new();
        loop {
            match self.char() {
                0 => {
                    return Err(LexerError::BadEof);
                }
                b']' => {
                    break;
                }
                b'-' => {
                    let c = self.getchar()?;
                    if !prev.ischar() || !c.ischar() {
                        return Err(LexerError::BadCharset);
                    }
                    let a = prev.chars().next().unwrap();
                    let b = c.chars().next().unwrap();
                    set = set | Charset::range(a, b);
                    prev = Charset::new();
                }
                c => {
                    self.peekc = c;
                    prev = self.getchar()?;
                    set = set | prev;
                }
            }
        }

        if invert {
            return Ok(set.invert() & self.charset);
        }
        return Ok(set);
    }

    fn escaped(&mut self) -> LexerResult<Charset> {
        let mut c = self.char();
        let mut a = 0u8;
        return match c {
            0 => Err(LexerError::BadEof),
            b'n' => Ok(Charset::char(b'\n')),
            b't' => Ok(Charset::char(b'\t')),
            b'r' => Ok(Charset::char(b'\r')),
            b'q' => Ok(Charset::char(b'\'')),
            b'd' => Ok(Charset::range(b'0', b'9')), // other builtin charsets to be implemented
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' => {
                loop {
                    a = a * 8 + c - b'0';
                    c = self.char();
                    if c < b'0' || c > b'9' {
                        self.peekc = c;
                        break;
                    }
                }
                Ok(Charset::char(a))
            }
            _ => Ok(Charset::char(c)),
        };
    }
}

#[cfg(test)]
mod lexer {
    use super::*;

    fn charset(s: &[u8]) -> Token {
        let mut r = Charset::new();
        for c in s.iter() {
            r = r | Charset::char(*c);
        }
        return Token::Char(r);
    }

    fn onetok(charset: Charset, s: &[u8]) -> LexerResult<Option<Token>> {
        let mut lex = Lexer {
            iter: s.iter(),
            charset,
            peekc: 0,
        };
        lex.token()
    }

    #[test]
    fn just_works() {
        let hello = b"he[acd-fq]+*|()\\+\\*\\|\\(\\)[+*|()][\\]][[-]][[\\-]\\040";
        let mut lex = Lexer {
            iter: hello.iter(),
            charset: Charset::range(b' ', b'~'),
            peekc: 0,
        };
        assert_eq!(lex.token().unwrap(), Some(charset(b"h")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"e")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"acdefq")));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'+')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'*')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'|')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'(')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b')')));
        assert_eq!(lex.token().unwrap(), Some(charset(b"+")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"*")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"|")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"(")));
        assert_eq!(lex.token().unwrap(), Some(charset(b")")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"+*|()")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"]")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"[\\]")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"[-")));
        assert_eq!(lex.token().unwrap(), Some(charset(b" ")));
        assert_eq!(lex.token().unwrap(), None);
    }

    #[test]
    fn operators() {
        let s = b"[^a].+*|()";
        let mut lex = Lexer {
            iter: s.iter(),
            charset: Charset::range(b'a', b'c'),
            peekc: 0,
        };
        assert_eq!(lex.token().unwrap(), Some(charset(b"bc")));
        assert_eq!(lex.token().unwrap(), Some(charset(b"abc")));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'+')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'*')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'|')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b'(')));
        assert_eq!(lex.token().unwrap(), Some(Token::Op(b')')));
        assert_eq!(lex.token().unwrap(), None);
    }

    #[test]
    fn errors() {
        let s = Charset::range(b'a', b'z');
        assert_eq!(onetok(s, b"\\").unwrap_err(), LexerError::BadEof);
        assert_eq!(onetok(s, b"[").unwrap_err(), LexerError::BadEof);
        assert_eq!(onetok(s, b"[hello").unwrap_err(), LexerError::BadEof);
        assert_eq!(onetok(s, b"[i-").unwrap_err(), LexerError::BadEof);
        assert_eq!(onetok(s, b"\\|").unwrap_err(), LexerError::BadChar(b'|'));
        assert_eq!(onetok(s, b"[\\d-i]").unwrap_err(), LexerError::BadCharset);
        assert_eq!(onetok(s, b"[i-\\d]").unwrap_err(), LexerError::BadCharset);
        assert_eq!(onetok(s, b"\\0").unwrap_err(), LexerError::BadChar(0));
    }
}
