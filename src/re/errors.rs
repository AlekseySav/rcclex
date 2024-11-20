pub type RegexResult<T> = std::result::Result<T, RegexError>;

#[derive(Debug, Clone, PartialEq)]
pub enum RegexError {
    BadChar(u8),
    BadEof,
    BadCharset,
    BadExpr,
}
