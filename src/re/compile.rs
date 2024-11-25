pub mod re {
    use std::collections::HashSet;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        Charset,
        Escape,
        Repeat,
        Balance,
        Group,
        Union,
        Empty,
        Postfix,
        NoMatch,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Self::Charset => write!(f, "bad charset syntax"),
                Self::Escape => write!(f, "invalid escape sequence"),
                Self::Repeat => write!(f, "bad repeat syntax"),
                Self::Balance => write!(f, "bad () balance"),
                Self::Group => write!(f, "attempted to define empty expr as a group"),
                Self::Union => write!(f, "invalid usage of '|' or bad () balance"),
                Self::Empty => write!(f, "empty expression or sub-expression"),
                Self::Postfix => write!(f, "invalid usage of postfix operator"),
                Self::NoMatch => write!(f, "pattern does not match"),
            }
        }
    }

    pub trait Automation {
        fn nodes(&self) -> impl Iterator<Item = (HashSet<usize>, HashSet<usize>)>;
        fn edges(&self) -> impl Iterator<Item = (usize, usize, Option<u8>)>;
    }

    pub fn nfa_uncooked(s: &[u8]) -> Result<impl Automation> {
        internal::nfa_uncooked(internal::Lexer::new(s))
    }

    pub fn nfa(s: &[u8]) -> Result<impl Automation> {
        internal::build_nfa(internal::Lexer::new(s))
    }

    pub fn dfa(s: &[u8]) -> Result<impl Automation> {
        use internal::*;
        Ok(build_dfa(build_nfa(Lexer::new(s))?))
    }

    include!("charset.rs");

    mod internal {
        use super::*;
        include!("lexer.rs");
        include!("build_nfa.rs");
        include!("build_dfa.rs");
    }
}
