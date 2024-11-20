pub type RegexResult<T> = std::result::Result<T, RegexError>;

#[derive(Debug, Clone, PartialEq)]
pub enum RegexError {
    BadChar(u8),
    BadEof,
    BadCharset,
    BadExpr,
    NoMatch,
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadChar(c) => write!(f, "character not in charset: '{}'", Charset::char(*c)),
            Self::BadCharset => write!(f, "bad charset syntax"),
            Self::BadEof => write!(f, "unexpected (eof)"),
            Self::BadExpr => write!(f, "bad expression syntax"),
            Self::NoMatch => write!(f, "pattern does not match"),
        }
    }
}
