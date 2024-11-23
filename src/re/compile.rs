pub mod re {
    pub type RegexResult<T> = std::result::Result<T, RegexError>;

    #[derive(Debug, Clone, PartialEq)]
    pub enum RegexError {
        Charset,
        Escape,
        Repeat,
        Balance,
        Group,
        Union,
        Empty,
        NoMatch,
    }

    impl std::fmt::Display for RegexError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Self::Charset => write!(f, "bad charset syntax"),
                Self::Escape => write!(f, "invalid escape sequence"),
                Self::Repeat => write!(f, "bad repeat syntax"),
                Self::Balance => write!(f, "bad () balance"),
                Self::Group => write!(f, "attempted to define empty expr as a group"),
                Self::Union => write!(f, "invalid usage of '|'"),
                Self::Empty => write!(f, "empty expression or sub-expression"),
                Self::NoMatch => write!(f, "pattern does not match"),
            }
        }
    }

    include!("charset.rs");
    include!("lexer.rs");
    include!("build_nfa_uncooked.rs");
    include!("build_nfa.rs");
}
