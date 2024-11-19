use std::fmt;

include!("re/charset.rs");
include!("re/automation.rs");
include!("re/compile/lexer.rs");
include!("re/compile/build_nfa.rs");
include!("re/compile/build_1nfa.rs");

fn main() {
    let s = b"(he|llo)#";
    let mut lex = Lexer {
        iter: s.iter(),
        charset: Charset::range(b'#', b'{'),
        peekc: 0,
    };
    let mut nfa = NFA::build(&mut lex, 0).unwrap();
    let nfa1 = NFA1::build(&mut nfa);
    println!("{}", nfa.traverse(nfa1.begin, b"he").unwrap());
    println!("{}", Graphviz(nfa1));
}
