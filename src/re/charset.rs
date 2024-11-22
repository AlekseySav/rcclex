/*
 * Regex Engine for rcclex
 */

#[derive(Debug, PartialEq)]
pub struct Charset {
    c: u128,
}

pub struct CharsetIter {
    c: u128,
    fwd: u8,
    bwd: u8,
}

impl Charset {
    pub const ALL: Charset = Charset { c: std::u128::MAX };

    pub fn new() -> Charset {
        Charset { c: 0 }
    }

    pub fn iter(&self) -> CharsetIter {
        CharsetIter {
            c: self.c,
            fwd: 0,
            bwd: 0,
        }
    }

    pub fn inv(mut self) -> Charset {
        self.c ^= std::u128::MAX;
        return self;
    }

    pub fn add(&mut self, s: &Charset) {
        self.c |= s.c;
    }

    pub fn add_char(&mut self, c: u8) {
        assert!(c <= 127);
        self.c |= 1u128 << c;
    }

    pub fn add_range(&mut self, a: u8, b: u8) {
        assert!(a <= b);
        for c in a..=b {
            self.add_char(c);
        }
    }

    pub fn empty(&self) -> bool {
        self.c == 0
    }
}

impl std::fmt::Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for c in self.iter() {
            match c {
                b'-' | b']' => write!(f, "\\{}", c as char),
                b'\n' => write!(f, "\\n"),
                b'\t' => write!(f, "\\t"),
                32..=126 => write!(f, "{}", c as char),
                _ => write!(f, "\\x{:02x}", c),
            }?;
        }
        write!(f, "]")
    }
}

impl Iterator for CharsetIter {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        while self.fwd <= 127 {
            self.fwd += 1;
            if (self.c & 1u128 << (self.fwd - 1)) != 0 {
                return Some(self.fwd - 1);
            }
        }
        None
    }
}

impl DoubleEndedIterator for CharsetIter {
    fn next_back(&mut self) -> Option<u8> {
        while self.bwd <= 127 {
            self.bwd += 1;
            if (self.c & 1u128 << (128 - self.bwd)) != 0 {
                return Some(128 - self.bwd);
            }
        }
        None
    }
}

#[macro_export]
macro_rules! charset {
    () => { $crate::re::Charset::new() };
    ( $( [$a:expr, $b:expr] ),+ ) => {
        {
            let mut s = $crate::re::Charset::new();
            $( s.add_range($a, $b); )*
            s
        }
    };
    ( $( [$a:expr, $b:expr] ),+; $( $x:expr ),+ ) => {
        {
            let mut s = $crate::re::Charset::new();
            $( s.add_char($x); )*
            $( s.add_range($a, $b); )*
            s
        }
    };
    ( $( $x:expr ),+ ) => {
        {
            let mut s = $crate::re::Charset::new();
            $( s.add_char($x); )*
            s
        }
    };
}

#[cfg(test)]
mod test_charset {
    #[test]
    fn construct() {
        assert_eq!(charset!(0).c, 1);
        assert_eq!(charset!(0, 2).c, 5);
        assert_eq!(charset!(0, 1, 2).c, 7);
        assert_eq!(charset!([0, 2]; 5).c, 7 | 32);
        assert_eq!(charset!([0, 127]).c, std::u128::MAX);
    }

    #[test]
    fn methods() {
        let mut s = charset!([1, 2]);
        assert_eq!(s.c, 6);
        s.add(&charset!(5));
        assert_eq!(s.c, 38);
        assert_eq!(s.inv().c, std::u128::MAX - 38);
    }

    #[test]
    fn iter() {
        let s = charset!([0, 127]);
        let mut it = s.iter();
        for i in 0..=127 {
            assert_eq!(it.next(), Some(i));
        }
        assert_eq!(it.next(), None);

        let s = charset!([8, 27], [50, 52]; 6);
        let mut it = s.iter();
        assert_eq!(it.next(), Some(6));
        for i in 8..=27 {
            assert_eq!(it.next(), Some(i));
        }
        for i in 50..=52 {
            assert_eq!(it.next(), Some(i));
        }
        assert_eq!(it.next(), None);
    }
}
