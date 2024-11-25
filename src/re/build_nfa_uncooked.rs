/*
 * Build NFA (first stage)
 *
 * Run Thompson algorithm
 * - NFA is stored as a list of edges
 * - for each group, nfa.head[n] = g, if n starts group g, nfa.tail[n] = g, if end
 */

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct NFAUncooked {
    nodes: usize,
    groups: usize,
    begin: usize,
    edges: Vec<(usize, usize, Charset)>,
    eps_edges: Vec<(usize, usize)>,
    head: HashMap<usize, usize>,
    tail: HashMap<usize, usize>,
}

pub fn nfa_uncooked(mut lex: Lexer) -> Result<NFAUncooked> {
    let mut nfa = NFAUncooked {
        nodes: 0,
        groups: 0,
        begin: 0,
        edges: Vec::new(),
        eps_edges: Vec::new(),
        head: HashMap::new(),
        tail: HashMap::new(),
    };
    nfa.begin = nfa.compile(&mut lex, 0)?.0;
    Ok(nfa)
}

impl NFAUncooked {
    fn node(&mut self) -> usize {
        self.nodes += 1;
        return self.nodes - 1;
    }

    fn group(&mut self) -> usize {
        self.groups += 1;
        return self.groups - 1;
    }

    fn join(&mut self, queue: &mut Vec<(usize, usize, usize)>, last: usize) -> Result<()> {
        if last == queue.len() {
            return Err(Error::Union);
        }
        while last < queue.len() - 1 {
            let (b, a) = (queue.pop().unwrap(), queue.pop().unwrap());
            self.eps_edges.push((a.1, b.0));
            queue.push((a.0, b.1, a.2 + b.2));
        }
        Ok(())
    }

    fn union(&mut self, queue: &mut Vec<(usize, usize, usize)>) {
        while queue.len() > 1 {
            let (q, p) = (queue.pop().unwrap(), queue.pop().unwrap());
            let (a, b) = (self.node(), self.node());
            self.eps_edges.push((a, p.0));
            self.eps_edges.push((a, q.0));
            self.eps_edges.push((p.1, b));
            self.eps_edges.push((q.1, b));
            queue.push((a, b, p.2 + q.2 + 2));
        }
    }

    fn copy_last(&mut self, p: (usize, usize, usize)) -> (usize, usize, usize) {
        let size = p.2;
        let origin = self.nodes - size;
        self.nodes += size;
        for i in (0..self.edges.len()).rev() {
            let (a, b, c) = self.edges[i].clone();
            if a < origin && b < origin {
                break;
            }
            self.edges.push((a + size, b + size, c));
        }
        for i in (0..self.eps_edges.len()).rev() {
            let (a, b) = self.eps_edges[i];
            if a < origin || b < origin {
                break;
            }
            self.eps_edges.push((a + size, b + size));
        }
        return (p.0 + size, p.1 + size, size);
    }

    fn compile(&mut self, lex: &mut Lexer, scope: usize) -> Result<(usize, usize, usize)> {
        let mut queue: Vec<(usize, usize, usize)> = Vec::new();
        let mut last_union = 0;
        loop {
            match lex.token()? {
                Token::Close(eof) => {
                    if (scope == 0) != eof {
                        return Err(Error::Balance);
                    }
                    self.join(&mut queue, last_union)?;
                    self.union(&mut queue);
                    return queue.pop().ok_or(Error::Empty);
                }

                Token::Open => queue.push(self.compile(lex, scope + 1)?),

                Token::Repeat((min, max)) => {
                    if last_union == queue.len() {
                        return Err(Error::Postfix);
                    }
                    let max_bound = max.unwrap_or(min + 1);
                    let mut a = queue.last().ok_or(Error::Group)?.clone();
                    for i in 0..max_bound {
                        if i == min {
                            self.eps_edges.push((a.0, a.1));
                        }
                        if i < max_bound - 1 {
                            let b = self.copy_last(a);
                            queue.push(b);
                            a = b;
                        }
                    }
                    if max == None {
                        let (a, b, p) = (self.node(), self.node(), queue.pop().unwrap());
                        self.eps_edges.push((a, p.0));
                        self.eps_edges.push((p.1, b));
                        self.eps_edges.push((b, a));
                        queue.push((a, b, p.2 + 2));
                    }
                }

                Token::Char(charset) => {
                    let (a, b) = (self.node(), self.node());
                    self.edges.push((a, b, charset));
                    queue.push((a, b, 2));
                }

                Token::Union => {
                    self.join(&mut queue, last_union)?;
                    last_union = queue.len();
                }

                Token::Group => {
                    if queue.len() == last_union {
                        return Err(Error::Postfix);
                    }
                    let p = queue.pop().unwrap();
                    let (a, b, g) = (self.node(), self.node(), self.group());
                    self.head.insert(a, g);
                    self.tail.insert(b, g);
                    self.eps_edges.push((a, p.0));
                    self.eps_edges.push((p.1, b));
                    queue.push((a, b, p.2 + 2));
                }
            }
        }
    }
}

impl Automation for NFAUncooked {
    fn nodes(&self) -> impl Iterator<Item = (HashSet<usize>, HashSet<usize>)> {
        let g = |s: &HashMap<usize, usize>, i| match s.get(&i) {
            None => HashSet::new(),
            Some(c) => HashSet::from([*c]),
        };
        let s: HashSet<usize> = HashSet::new();
        std::iter::once((s.clone(), s.clone()))
            .chain((0..self.nodes).map(move |i| (g(&self.head, i), g(&self.tail, i))))
    }

    fn edges(&self) -> impl Iterator<Item = (usize, usize, Option<u8>)> {
        self.edges
            .iter()
            .flat_map(|(a, b, s)| s.iter().map(|c| (*a + 1, *b + 1, Some(c))))
            .chain(self.eps_edges.iter().map(|(a, b)| (*a + 1, *b + 1, None)))
            .chain(std::iter::once((0, self.begin + 1, None)))
    }
}

#[cfg(test)]
mod test_nfa_uncooked {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn basic() {
        let nfa = nfa_uncooked(Lexer::new(b"(a)\\Z|b\\Z")).unwrap();
        /*
         *     2* - 0 -a- 1 - 3*
         * 8 <                 > 9
         *     6* - 4 -b- 5 - 7*
         */
        assert_eq!(nfa.nodes, 10);
        assert_eq!(nfa.begin, 8);
        assert_eq!(nfa.groups, 2);
        assert_eq!(nfa.head, HashMap::from([(2, 0), (6, 1)]));
        assert_eq!(nfa.tail, HashMap::from([(3, 0), (7, 1)]));
        assert_eq!(
            nfa.edges,
            vec![(0, 1, charset!(b'a')), (4, 5, charset!(b'b'))]
        );
        assert_eq!(
            HashSet::from_iter(nfa.eps_edges),
            HashSet::from([
                (2, 0),
                (1, 3),
                (6, 4),
                (5, 7),
                (8, 2),
                (8, 6),
                (3, 9),
                (7, 9)
            ])
        );

        let nfa = nfa_uncooked(Lexer::new(b"((a|b)c((d)))")).unwrap();
        /*
         *     0 -a- 1
         * 4 <         > 5 - 6 -c- 7 - 8 -d- 9
         *     2 -b- 3
         */
        assert_eq!(nfa.nodes, 10);
        assert_eq!(nfa.begin, 4);
        assert_eq!(nfa.head, HashMap::new());
        assert_eq!(nfa.tail, HashMap::new());
        assert_eq!(
            nfa.edges,
            vec![
                (0, 1, charset!(b'a')),
                (2, 3, charset!(b'b')),
                (6, 7, charset!(b'c')),
                (8, 9, charset!(b'd'))
            ]
        );
        assert_eq!(
            HashSet::from_iter(nfa.eps_edges),
            HashSet::from([(4, 0), (4, 2), (1, 5), (3, 5), (5, 6), (7, 8),])
        );
    }

    #[test]
    fn repeats() {
        let nfa = nfa_uncooked(Lexer::new(b"a{3}")).unwrap();
        // 0 -a- 1 - 2 -a- 3 - 4 -a- 5
        assert_eq!(nfa.nodes, 6);
        assert_eq!(nfa.begin, 0);
        assert_eq!(
            HashSet::from_iter(nfa.eps_edges),
            HashSet::from([(1, 2), (3, 4)])
        );
        assert_eq!(
            nfa.edges,
            vec![
                (0, 1, charset!(b'a')),
                (2, 3, charset!(b'a')),
                (4, 5, charset!(b'a'))
            ]
        );

        let nfa = nfa_uncooked(Lexer::new(b"a{1,2}b{,1}")).unwrap();
        // 0 -a- 1 - 2 -a-,- 3 - 4 -b-,- 5
        assert_eq!(nfa.nodes, 6);
        assert_eq!(nfa.begin, 0);
        assert_eq!(
            HashSet::from_iter(nfa.eps_edges),
            HashSet::from([(1, 2), (2, 3), (3, 4), (4, 5)])
        );
        assert_eq!(
            nfa.edges,
            vec![
                (0, 1, charset!(b'a')),
                (2, 3, charset!(b'a')),
                (4, 5, charset!(b'b'))
            ]
        );

        let nfa = nfa_uncooked(Lexer::new(b"a{,1}b{,}c{1,}d{2,}")).unwrap();
        // [  a?  ]    [        b*       ]   [              c+              ]
        // 0 -a-> 1 -> 4 -> 2 -b-> 3 -> 5 -> 6 -c-> 7 -> 10 -> 8 -c-> 9 -> 11
        //   --->             --->                              --->
        //              <- - - - - - - -                  < - - - - - - - -
        //    [                 d{2,}                          ]
        // -> 12 -d-> 13 -> 14 -d-> 15 -> 18 -> 16 -d-> 17 -> 19
        //                                         --->
        //                                 < - - - - - - - - -
        assert_eq!(nfa.nodes, 20);
        assert_eq!(nfa.begin, 0);
        assert_eq!(
            HashSet::from_iter(nfa.eps_edges.clone()),
            HashSet::from([
                (0, 1),
                (1, 4),
                (4, 2),
                (2, 3),
                (3, 5),
                (5, 6),
                (7, 10),
                (10, 8),
                (8, 9),
                (9, 11),
                (11, 12),
                (13, 14),
                (15, 18),
                (18, 16),
                (16, 17),
                (17, 19),
                (5, 4),
                (11, 10),
                (19, 18)
            ])
        );
        assert_eq!(
            nfa.edges,
            vec![
                (0, 1, charset!(b'a')),
                (2, 3, charset!(b'b')),
                (6, 7, charset!(b'c')),
                (8, 9, charset!(b'c')),
                (12, 13, charset!(b'd')),
                (14, 15, charset!(b'd')),
                (16, 17, charset!(b'd')),
            ]
        );

        let nfa2 = nfa_uncooked(Lexer::new(b"a?b*c+d{2,}")).unwrap();
        assert_eq!(nfa, nfa2);
    }

    fn nfa_err(s: &[u8]) -> Error {
        return nfa_uncooked(Lexer::new(s)).unwrap_err();
    }

    #[test]
    fn errors() {
        assert_eq!(nfa_err(b"("), Error::Balance);
        assert_eq!(nfa_err(b"((())))"), Error::Union);
        assert_eq!(nfa_err(b"a|"), Error::Union);
        assert_eq!(nfa_err(b"|ada"), Error::Union);
        assert_eq!(nfa_err(b"\\Z"), Error::Postfix);
        assert_eq!(nfa_err(b"a|*"), Error::Postfix);
        assert_eq!(nfa_err(b"a|\\Z"), Error::Postfix);
    }
}
