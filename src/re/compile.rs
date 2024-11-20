include!("charset.rs");
include!("lexer.rs");
include!("build_nfa.rs");
include!("build_1nfa.rs");
include!("build_dfa.rs");

pub fn compile(charset: Charset, s: &[u8]) -> LexerResult<DFA> {
    let lex = Lexer {
        iter: s.iter(),
        charset,
        peekc: 0,
    };
    Ok(DFA::build(NFA1::build(NFA::build(lex, 0)?)))
}

#[cfg(test)]
mod regex {
    use super::*;
    include!("t.rs");

    fn traverse(dfa: &DFA, s: &[u8], buf: &mut Vec<u8>) -> bool {
        let mut state = 0usize;
        for c in s {
            state = match dfa.nodes[state].get(c) {
                None => break,
                Some(n) => *n,
            };
            buf.push(*c);
        }
        return dfa.nodes[state].get(&127u8) != None;
    }

    fn run<'a>(re: &[u8], s: &[u8], buf: &'a mut Vec<u8>) -> ReMatchResult<'a> {
        let mut v: Vec<u8> = Vec::from(b"(");
        v.extend_from_slice(re);
        v.extend_from_slice(b")\\177");
        let dfa = match compile(Charset::new().invert(), &v) {
            Ok(d) => d,
            Err(_) => return ReMatchResult::Err,
        };
        for i in 0..s.len() {
            buf.clear();
            match traverse(&dfa, &s[i..], buf) {
                false => continue,
                true => return ReMatchResult::Yes(buf),
            }
        }
        return ReMatchResult::No;
    }

    #[test]
    fn perl_test() {
        for ts in RE_TEST_SUITES {
            println!(
                "{} {}",
                String::from_utf8_lossy(ts.re),
                String::from_utf8_lossy(ts.s)
            );
            let mut buf: Vec<u8> = Vec::new();
            let res = run(ts.re, ts.s, &mut buf);
            assert_eq!(res, ts.res);
        }
    }
}
