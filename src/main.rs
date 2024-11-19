use std::fmt;

include!("charset.rs");
include!("automation.rs");
include!("compile/lexer.rs");
include!("compile/build_nfa.rs");
include!("compile/build_1nfa.rs");

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
