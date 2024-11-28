/*
 * att ast tests
 */

#[cfg(test)]
pub mod att_re_tests {
    include!("t.rs");
    use super::*;

    type MatchRes = Vec<(Option<usize>, Option<usize>)>;

    #[derive(Debug)]
    pub struct Test<'a> {
        pub re: &'a [u8],
        pub s: &'a [u8],
        pub groups: &'a [(Option<usize>, Option<usize>)],
        pub err: bool,
    }

    fn match_compiled(r: &re::Regex, s: &[u8], start: usize, res: &MatchRes) -> bool {
        let mut state = 0;
        let mut failed = false;
        for (i, c) in s
            .iter()
            .enumerate()
            .skip(start)
            .chain(std::iter::once((s.len(), &0)))
        {
            for (g, (h, t)) in res.iter().enumerate() {
                if (*h == Some(i) || *t == Some(i)) && failed {
                    return false;
                }
                if *h == Some(i) && !r.head[state].contains(&g) {
                    return false;
                }
                if *t == Some(i) && !r.tail[state].contains(&g) {
                    return false;
                }
            }
            if i != s.len() {
                let next = r.nodes[state].get(c);
                match next {
                    None => failed = true,
                    Some(next) => state = *next,
                }
            }
        }
        return true;
    }

    fn verify_match(pattern: &[u8], s: &[u8], res: &MatchRes) -> re::Result<bool> {
        let mut config = re::Config::default();
        config.auto_groups = true;
        let r = re::compile(pattern, config)?;
        return Ok(match_compiled(&r, s, res[0].0.unwrap(), res));
    }

    #[test]
    fn just_works() {
        assert_eq!(
            verify_match(b"(hello)", b"hello", &vec![(Some(0), Some(5))]),
            Ok(true)
        );
        assert_eq!(
            verify_match(b"(hello)", b"hellou", &vec![(Some(0), Some(5))]),
            Ok(true)
        );
        assert_eq!(
            verify_match(b"(hellou)", b"hello", &vec![(Some(0), Some(5))]),
            Ok(false)
        );
        assert_eq!(
            verify_match(b"(hello)", b"hellou", &vec![(Some(0), Some(6))]),
            Ok(false)
        );
        assert_eq!(
            verify_match(b"(hell)", b"hellou", &vec![(Some(0), Some(5))]),
            Ok(false)
        );
        assert_eq!(
            verify_match(b"(((a)))", b"a", &vec![(Some(0), Some(1)); 3]),
            Ok(true)
        );
    }

    // fn graphviz_dfa(s: &[u8]) -> Graphviz<re::DFA> {
    //     let mut c = re::Config::default();
    //     c.auto_groups = true;
    //     Graphviz(re::build_dfa(re::build_nfa(re::Lexer::new(s, c)).unwrap()))
    // }

    #[test]
    fn run_all() {
        for (i, t) in ATT_RE_TESTS_LIST.iter().enumerate() {
            eprintln!(
                "[test #{}]\tre: {:30}str: {:30} ans: {:?}",
                i,
                std::str::from_utf8(t.re).unwrap_or("<?>"),
                std::str::from_utf8(t.s).unwrap_or("<?>"),
                t.groups
            );
            let res = verify_match(t.re, t.s, &Vec::from(t.groups));
            if res.is_err() {
                eprintln!("compilation failed with '{}'", res.clone().unwrap_err());
            }
            // if res.clone().unwrap_or(false) == t.err {
            //     println!("{}", graphviz_dfa(t.re));
            // }
            assert_ne!(res.unwrap_or(false), t.err);
        }
    }
}
