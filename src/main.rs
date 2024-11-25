include!("re/compile.rs");
include!("graphviz.rs");

fn main() {
    let nfa = re::dfa(b"(ab)\\Z|a\\Z").unwrap();
    println!("{}", Graphviz(nfa));
    println!("Hello, world!");
}
