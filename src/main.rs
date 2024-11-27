include!("re/compile.rs");
include!("graphviz.rs");
include!("tests.rs");

fn main() {
    println!(
        "{}",
        Graphviz(re::build_dfa(
            re::build_nfa(re::Lexer::new(
                b"\\A(\\A(a?)\\Z{2})\\Z",
                re::Config::default(),
            ))
            .unwrap()
        ))
    );
}
