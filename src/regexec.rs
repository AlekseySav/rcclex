/*
 * use regex to match a string
 * rcclex match <pattern> <string>
 *
 * characters '\x80'-'\xbe' used to define group opening
 * characters '\xc0'-'\xfe' used to define group closing
 */

const GROUP_0: usize = 0x80;
const GROUP_CLOSE: usize = 0x40;

fn regexec_group(lex: &mut Lexer, pattern: &mut Vec<u8>, mut group: u8) -> RegexResult<u8> {
    let origin = group;
    pattern.push(b'(');
    pattern.push(origin);
    pattern.push(b'(');
    loop {
        match lex.token()? {
            None => break,
            Some(Token::Op(b')')) => break,
            Some(Token::Op(b'(')) => group = regexec_group(lex, pattern, group + 1)?,
            Some(Token::Op(c)) => pattern.push(c),
            Some(Token::Char(c)) => pattern.extend_from_slice(format!("{}", c).as_bytes()),
        }
    }
    pattern.push(b')');
    pattern.push(origin | GROUP_CLOSE as u8);
    pattern.push(b')');
    Ok(group)
}

fn regexec_pattern(pattern: &[u8]) -> RegexResult<(Charset, Vec<u8>, usize)> {
    let mut lex = Lexer {
        iter: pattern.iter(),
        charset: Charset::range(0, 127),
        peekc: 0,
    };
    let mut v: Vec<u8> = Vec::new();
    let last_group: u8 = regexec_group(&mut lex, &mut v, GROUP_0 as u8)?;
    return Ok((
        Charset::range(0, 127)
            | Charset::range(GROUP_0 as u8, last_group)
            | Charset::range(
                (GROUP_0 | GROUP_CLOSE) as u8,
                last_group | GROUP_CLOSE as u8,
            ),
        v,
        last_group as usize - GROUP_0 + 1,
    ));
}

fn traverse_regex(re: &Regex, s: &[u8], n_groups: usize) -> Option<Vec<(usize, usize)>> {
    let mut state = 0usize;
    let mut saved: Option<Vec<(usize, usize)>> = None;
    let mut groups: Vec<(usize, usize)> = vec![(0, 0); n_groups];
    for (i, c) in s.iter().enumerate() {
        let mut prev: usize;
        loop {
            prev = state;
            for c in 0..groups.len() {
                let next = re.nodes[prev][c | GROUP_0];
                if next != re.end() {
                    println!("into group {}", state);
                    state = next;
                    groups[c].0 = i;
                }
                let next = re.nodes[prev][c | GROUP_0 | GROUP_CLOSE];
                if next != re.end() {
                    println!("exit group {}", state);
                    state = next;
                    groups[c].1 = i;
                    if c == 0 {
                        saved = Some(groups.clone());
                    }
                }
            }
            if prev == state {
                break;
            }
        }
        let next = re.nodes[state][*c as usize];
        if next == re.end() {
            break;
        }
        state = next;
    }
    println!("{} {}", re.nodes[state][GROUP_0], re.end());
    return saved;
}

pub fn regexec(pattern: &[u8], s: &[u8]) -> RegexResult<Vec<(usize, usize)>> {
    let (charset, v, n_groups) = regexec_pattern(pattern)?;
    let mut s: Vec<u8> = Vec::from(s);
    s.push(0xbf);

    let re = match compile(charset, &v) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };

    for i in 0..s.len() {
        match traverse_regex(&re, &s[i..], n_groups) {
            None => (),
            Some(mut g) => {
                for t in g.iter_mut() {
                    t.0 += i;
                    t.1 += i;
                }
                return Ok(g);
            }
        }
    }
    return Err(RegexError::NoMatch);
}

#[cfg(test)]
mod regexec {
    use super::*;

    #[test]
    fn pattern_test() {
        let (c1, v1, g1) = regexec_pattern(b"hello").unwrap();
        let set = Charset::range(0, 128) | Charset::char(0xc0);
        assert_eq!(c1, set);
        assert_eq!(g1, 1);
        assert_eq!(v1, b"(\x80([h][e][l][l][o])\xc0)");

        let (c1, v1, g1) = regexec_pattern(b"(a(())(b(c)))").unwrap();
        let set = Charset::range(0, 127 + 6) | Charset::range(0xc0, 0xc5);
        assert_eq!(c1, set);
        assert_eq!(g1, 6);
        assert_eq!(
            v1,
            b"(\x80((\x81([a](\x82((\x83()\xc3))\xc2)(\x84([b](\x85([c])\xc5))\xc4))\xc1))\xc0)"
        );
    }

    #[test]
    fn regexec_test() {
        assert_eq!(regexec(b"ab", b"ab"), Ok(vec![(0, 2)]));
        assert_eq!(regexec(b"a...b", b"abababbb"), Ok(vec![(2, 7)]));
        assert_eq!(regexec(b"(..)*(...)*", b"a"), Ok(vec![(0, 0); 3]));
        assert_eq!(regexec(b"(..)*(...)*", b"abcd"), Ok(vec![(0, 4), (2, 4)]));
        // E	(..)*(...)*		abcd	(0,4)(2,4)
        // E	(ab|a)(bc|c)		abc	(0,3)(0,2)(2,3)
        // E	(ab)c|abc		abc	(0,3)(0,2)
    }
}
