pub const MaxChar = 128;

pub const ParseError = error{
    BadChar,
    UnexpectedEnd,
    BadCharset,
    BadBraceBalance,
    BadExpr,
};
