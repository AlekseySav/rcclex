include!("charset.rs");
include!("lexer.rs");
include!("build_nfa.rs");
include!("build_1nfa.rs");
include!("build_dfa.rs");

pub fn compile(charset: Charset, s: &[u8]) -> LexerResult<DFA> {
    let lex = Lexer {
        iter: s.iter(),
        charset,
        peekc: 0,
    };
    Ok(DFA::build(NFA1::build(NFA::build(lex, 0)?)))
}
