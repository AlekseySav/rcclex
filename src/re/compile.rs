pub mod re {
    use std::collections::HashSet;

    pub trait Automation {
        fn nodes(&self) -> impl Iterator<Item = (HashSet<usize>, HashSet<usize>)>;
        fn edges(&self) -> impl Iterator<Item = (usize, usize, Option<u8>)>;
    }

    pub struct Regex {
        pub nodes: Vec<HashMap<u8, usize>>,
        pub head: Vec<HashSet<usize>>,
        pub tail: Vec<HashSet<usize>>,
    }

    pub fn compile(s: &[u8], config: Config) -> Result<Regex> {
        if !config.is_valid() {
            return Err(Error::Config);
        }
        let dfa = build_dfa(build_nfa(Lexer::new(s, config))?);
        Ok(Regex {
            nodes: dfa.nodes,
            head: dfa.head,
            tail: dfa.tail,
        })
    }

    include!("charset.rs");
    include!("config.rs");
    include!("lexer.rs");
    include!("build_nfa.rs");
    include!("build_dfa.rs");
}
