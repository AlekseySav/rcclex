use std::fmt;

include!("re/automation.rs");
include!("re/compile.rs");

fn main() {
    let s = b"(he|llo|h)#";
    let charset = Charset::range(b'#', b'{');
    let dfa = compile(charset, s).unwrap();
    println!("{}", dfa.traverse(0, b"he").unwrap());
    println!("{}", Graphviz(dfa));
}
