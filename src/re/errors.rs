pub type RegexResult<T> = std::result::Result<T, RegexError>;

#[derive(Debug, Clone, PartialEq)]
pub enum RegexError {
    Charset,
    Escape,
    Repeat,
    BadExpr,
    NoMatch,
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Charset => write!(f, "bad charset syntax"),
            Self::Escape => write!(f, "invalid escape sequence"),
            Self::Repeat => write!(f, "bad repeat syntax"),
            Self::BadExpr => write!(f, "bad expression syntax"),
            Self::NoMatch => write!(f, "pattern does not match"),
        }
    }
}
