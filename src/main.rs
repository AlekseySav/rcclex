use std::fmt;

include!("re/gv.rs");
include!("re/compile.rs");

fn main() {
    let s = b"[ab]+a#";
    let charset = Charset::range(b'#', b'{');
    let cdfa = compile(charset, charset, s).unwrap();
    println!("{}", Graphviz(cdfa));
}
