use std::collections::VecDeque;

/*
 * read lexer and build NFA
 * https://en.wikipedia.org/wiki/Nondeterministic_finite_automaton
 */

#[derive(Clone, Copy)]
pub struct Edge {
    a: usize,
    b: usize,
    c: Charset,
}

pub struct NFA {
    edges: Vec<Edge>,
    begin: usize,
    nodes: usize,
    epsilon: u8,
}

struct Slice {
    begin: usize,
    end: usize,
}

impl Automation for NFA {
    fn info(&self) -> AutomationInfo {
        return AutomationInfo {
            nodes: self.nodes,
            begin: self.begin,
            epsilon: Some(self.epsilon),
        };
    }

    fn contains_edge(&self, a: usize, b: usize, c: u8) -> bool {
        self.edges
            .iter()
            .position(|&r| r.a == a && r.b == b && r.c.contains(c))
            != None
    }
}

impl NFA {
    pub fn build(lex: &mut Lexer, epsilon: u8) -> LexerResult<NFA> {
        let mut nfa = NFA {
            edges: Vec::new(),
            begin: 0,
            nodes: 0,
            epsilon,
        };
        nfa.begin = nfa.compile(lex, 0)?.begin;
        Ok(nfa)
    }

    fn make_node(&mut self) -> usize {
        self.nodes += 1;
        return self.nodes - 1;
    }

    fn put_edge(&mut self, a: usize, b: usize, c: Charset) {
        self.edges.push(Edge { a, b, c });
    }

    fn charset(&mut self, c: Charset) -> Slice {
        let begin = self.make_node();
        let end = self.make_node();
        self.put_edge(begin, end, c);
        return Slice { begin, end };
    }

    fn concat(&mut self, q: &mut VecDeque<Slice>, concats: usize) -> LexerResult<()> {
        for _ in 1..concats {
            let (a, b) = NFA::pop2(q)?;
            self.put_edge(a.end, b.begin, Charset::char(self.epsilon));
            q.push_back(Slice {
                begin: a.begin,
                end: b.end,
            });
        }
        Ok(())
    }

    fn compile(&mut self, lex: &mut Lexer, depth: u32) -> LexerResult<Slice> {
        let mut queue: VecDeque<Slice> = VecDeque::new();
        let mut concats = 0usize;

        loop {
            match lex.token()? {
                None => {
                    if depth != 0 {
                        return Err(LexerError::BadExpr);
                    }
                    break;
                }
                Some(Token::Op(b')')) => {
                    if depth == 0 {
                        return Err(LexerError::BadExpr);
                    }
                    break;
                }
                Some(Token::Op(b'(')) => {
                    concats += 1;
                    queue.push_back(self.compile(lex, depth + 1)?);
                }
                Some(Token::Char(c)) => {
                    concats += 1;
                    queue.push_back(self.charset(c));
                }
                Some(Token::Op(b'*')) => {
                    let a = NFA::pop1(&mut queue)?;
                    let n = self.make_node();
                    self.put_edge(n, a.begin, Charset::char(self.epsilon));
                    self.put_edge(a.end, n, Charset::char(self.epsilon));
                    queue.push_back(Slice { begin: n, end: n });
                }
                Some(Token::Op(b'+')) => {
                    let n = NFA::pop1(&mut queue)?;
                    self.put_edge(n.end, n.begin, Charset::char(self.epsilon));
                    queue.push_back(n);
                }
                Some(Token::Op(b'|')) => {
                    self.concat(&mut queue, concats)?;
                    concats = 0;
                }
                Some(Token::Op(_)) => panic!("lexer returned unknown token"),
            }
        }

        self.concat(&mut queue, concats)?;
        while queue.len() >= 2 {
            let s = Slice {
                begin: self.make_node(),
                end: self.make_node(),
            };
            let (a, b) = NFA::pop2(&mut queue).unwrap();
            self.put_edge(s.begin, a.begin, Charset::char(self.epsilon));
            self.put_edge(s.begin, b.begin, Charset::char(self.epsilon));
            self.put_edge(a.end, s.end, Charset::char(self.epsilon));
            self.put_edge(b.end, s.end, Charset::char(self.epsilon));
            queue.push_back(s);
        }
        match queue.pop_back() {
            None => Err(LexerError::BadExpr),
            Some(s) => Ok(s),
        }
    }

    fn pop1(q: &mut VecDeque<Slice>) -> LexerResult<Slice> {
        return q.pop_back().ok_or(LexerError::BadExpr);
    }

    fn pop2(q: &mut VecDeque<Slice>) -> LexerResult<(Slice, Slice)> {
        let b = q.pop_back().ok_or(LexerError::BadExpr)?;
        let a = q.pop_back().ok_or(LexerError::BadExpr)?;
        return Ok((a, b));
    }
}

#[cfg(test)]
mod nfa {
    use super::*;

    #[test]
    fn jist_works() {
        // tests are small as nfa traversing is very slow
        let set = Charset::range(b'a', b'z');
        let mut n1 = Lexer {
            iter: b"(ab|(a)c*)z".iter(),
            charset: set,
            peekc: 0,
        };
        let nfa = NFA::build(&mut n1, 0).unwrap();
        assert_ne!(nfa.traverse(nfa.begin, b"az"), None);
        assert_ne!(nfa.traverse(nfa.begin, b"abz"), None);
        assert_ne!(nfa.traverse(nfa.begin, b"acz"), None);
        assert_ne!(nfa.traverse(nfa.begin, b"accz"), None);
        assert_ne!(nfa.traverse(nfa.begin, b"acccz"), None);
        assert_eq!(nfa.traverse(nfa.begin, b"bz"), None);
        assert_eq!(nfa.traverse(nfa.begin, b"cz"), None);
        assert_eq!(nfa.traverse(nfa.begin, b"bcz"), None);
        assert_eq!(nfa.traverse(nfa.begin, b"abcz"), None);
    }
}
