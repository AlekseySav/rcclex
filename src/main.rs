use std::fmt;

include!("re/gv.rs");
include!("re/compile.rs");

fn main() {
    let s = b"(he|llo|h)#";
    let charset = Charset::range(b'#', b'{');
    let dfa = compile(charset, s).unwrap();
    println!("{}", Graphviz(dfa));
}
