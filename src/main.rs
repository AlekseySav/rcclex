include!("re/compile.rs");
include!("graphviz.rs");

fn main() {
    println!(
        "{}",
        Graphviz(re::build_dfa(
            re::build_nfa(re::Lexer::new(
                b"a\\A([bc]*)\\Z\\A(cd)\\Z",
                re::Config::default(),
            ))
            .unwrap()
        ))
    );
}
