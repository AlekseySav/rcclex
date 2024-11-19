use std::ops::{BitAnd, BitOr};

/*
 * Charset may contain ascii 7-bit characters
 * as well as up to 126 auxiliary chars
 */

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charset {
    map: u128,
    cmd: u8,
}

pub struct CharsetIter {
    set: Charset,
    i: u8,
}

impl Charset {
    pub fn new() -> Charset {
        return Charset { map: 0, cmd: 0 };
    }

    pub fn char(c: u8) -> Charset {
        return Charset {
            map: 1u128 << c,
            cmd: 0,
        };
    }

    pub fn range(a: u8, b: u8) -> Charset {
        let mut s = Charset { map: 0u128, cmd: 0 };
        for c in a..=b {
            s.map |= 1u128 << c;
        }
        return s;
    }

    pub fn parse(s: &[u8]) -> LexerResult<Charset> {
        let mut res = Charset::new();
        let mut lex = Lexer {
            iter: s.iter(),
            charset: Charset::new().invert(),
            peekc: 0,
        };
        loop {
            match lex.token()? {
                None => {
                    return Ok(res);
                }
                Some(Token::Char(c)) => {
                    res = res | c;
                }
                _ => {
                    return Err(LexerError::BadCharset);
                }
            }
        }
    }

    pub fn invert(self) -> Charset {
        return Charset {
            map: self.map ^ std::u128::MAX,
            cmd: self.cmd,
        };
    }

    pub fn contains(self, c: u8) -> bool {
        if c < 128 {
            return (self.map & (1u128 << c)) != 0;
        }
        return c - 128 < self.cmd;
    }

    pub fn empty(self) -> bool {
        return self.map == 0 && self.cmd == 0;
    }

    pub fn ischar(self) -> bool {
        let mut it = self.chars();
        if it.next() == None {
            return false;
        }
        return it.next() == None;
    }

    pub fn addcmd(&mut self) -> u8 {
        assert!(self.cmd < 127);
        self.cmd += 1;
        return self.cmd + 127;
    }

    pub fn chars(self) -> CharsetIter {
        return CharsetIter { set: self, i: 0 };
    }
}

impl BitOr for Charset {
    type Output = Charset;
    fn bitor(self, rhs: Charset) -> Self::Output {
        return Charset {
            map: self.map | rhs.map,
            cmd: self.cmd,
        };
    }
}

impl BitAnd for Charset {
    type Output = Charset;
    fn bitand(self, rhs: Charset) -> Self::Output {
        return Charset {
            map: self.map & rhs.map,
            cmd: self.cmd,
        };
    }
}

impl std::fmt::Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in self.chars() {
            if c < 32 || c > 126 {
                write!(f, "\\{c:o}")?;
            } else {
                write!(f, "{}", c as char)?;
            }
        }
        Ok(())
    }
}

impl Iterator for CharsetIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i < 255 {
            self.i += 1;
            if self.set.contains(self.i - 1) {
                return Some(self.i - 1);
            }
        }
        return None;
    }
}

#[cfg(test)]
mod charset {
    use super::*;

    #[test]
    fn just_works() {
        let mut c = Charset::char(b'a');
        c = c | Charset::char(b'c');
        assert!(c.contains(b'a'));
        assert!(c.contains(b'c'));
        c = c.invert() & Charset::range(b'a', b'c');
        assert!(!c.contains(b'a'));
        assert!(!c.contains(b'c'));
        assert!(c.contains(b'b'));
        assert!(!c.contains(b'd'));
        for _ in 0..=126 {
            let cmd = c.addcmd();
            assert!(c.contains(cmd));
        }
    }

    #[test]
    fn iterators() {
        let s = Charset::range(b'a', b'z');
        for (i, c) in s.chars().enumerate() {
            assert_eq!(i + usize::from(b'a'), c.into());
        }
        let mut c = Charset::char(b'a');
        let cmd = c.addcmd();
        let mut it = c.chars();
        assert_eq!(it.next(), Some(b'a'));
        assert_eq!(it.next(), Some(cmd));
    }

    #[test]
    fn properties() {
        let mut s = Charset { map: 0, cmd: 0 };
        assert!(s.empty());
        assert!(!s.ischar());
        s = s | Charset::char(b'a');
        assert!(!s.empty());
        assert!(s.ischar());
        s = s | Charset::char(b'b');
        assert!(!s.empty());
        assert!(!s.ischar());
    }

    #[test]
    fn parse() {
        assert_eq!(
            Charset::parse(b"[a-d]").unwrap(),
            Charset::range(b'a', b'd')
        );
    }
}
