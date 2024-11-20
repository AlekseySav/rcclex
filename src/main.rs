use std::{env, fmt};

include!("re/gv.rs");
include!("re/compile.rs");
include!("match.rs");

fn main() {
    // let args: Vec<String> = env::args().collect();
    let s = b"(he|llo|h)#";
    let charset = Charset::range(b'#', b'{');
    let cdfa = compile(charset, s).unwrap();
    println!("{}", Graphviz(cdfa));
}
