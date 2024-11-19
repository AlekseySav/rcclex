use std::fmt;

include!("re/charset.rs");
include!("re/automation.rs");
include!("re/lexer.rs");
include!("re/build_nfa.rs");
include!("re/build_1nfa.rs");
include!("re/build_dfa.rs");

fn main() {
    let s = b"(he|llo|h)#";
    let lex = Lexer {
        iter: s.iter(),
        charset: Charset::range(b'#', b'{'),
        peekc: 0,
    };
    let nfa = NFA::build(lex, 0).unwrap();
    let nfa1 = NFA1::build(nfa);
    let dfa = DFA::build(nfa1);
    println!("{}", dfa.traverse(0, b"he").unwrap());
    println!("{}", Graphviz(dfa));
}
