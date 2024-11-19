use std::fmt;

include!("charset.rs");
include!("automation.rs");
include!("compile/lexer.rs");
include!("compile/build_nfa.rs");

fn main() {
    let s = b"he|llo{";
    let mut lex = Lexer {
        iter: s.iter(),
        charset: Charset::range(b'a', b'{'),
        peekc: 0,
    };
    let nfa = NFA::build(&mut lex, 0).unwrap();
    println!("{}", nfa.traverse(nfa.begin, b"he").unwrap());
    println!("{}", Graphviz(nfa));
}
