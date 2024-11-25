include!("errors.rs");
include!("charset.rs");
include!("lexer.rs");
include!("build_nfa.rs");
include!("build_1nfa.rs");
include!("build_dfa.rs");
include!("build_cdfa.rs");

const EPS: u8 = 255;

pub type Regex = CDFA;

pub fn compile(charset: Charset, dot: Charset, s: &[u8]) -> RegexResult<Regex> {
    let lex = Lexer {
        iter: s.iter(),
        charset,
        dot,
        peekc: 0,
    };
    Ok(CDFA::build(DFA::build(NFA1::build(NFA::build(lex, EPS)?))))
}
