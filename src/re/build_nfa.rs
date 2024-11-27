include!("build_nfa_uncooked.rs");

/*
 * Build NFA (second stage)
 *
 * - convert list of edges to an automation
 * - resolve epsilon closures
 * - propagate groups heads and tails
 */

#[derive(Clone, Debug)]
pub struct NFANode {
    edges: HashMap<u8, HashSet<usize>>,
    head: HashSet<usize>,
    tail: HashSet<usize>,
}

pub struct NFA {
    nodes: Vec<NFANode>,
    begin: usize,
}

pub fn build_nfa(lex: Lexer) -> Result<NFA> {
    let uncooked = nfa_uncooked(lex)?;
    let (mut nfa, eps, edges) = NFA::from(uncooked);
    nfa.process(eps, edges);
    Ok(nfa)
}

fn mut_pair<T>(v: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    if i < j {
        let (head, tail) = v.split_at_mut(i + 1);
        (&mut head[i], &mut tail[j - i - 1])
    } else {
        let (head, tail) = v.split_at_mut(j + 1);
        (&mut tail[i - j - 1], &mut head[j])
    }
}

impl NFA {
    fn from(nfa: NFAUncooked) -> (NFA, Vec<HashSet<usize>>, Vec<Vec<Charset>>) {
        let mut res = NFA {
            nodes: vec![
                NFANode {
                    edges: HashMap::new(),
                    head: HashSet::new(),
                    tail: HashSet::new(),
                };
                nfa.nodes
            ],
            begin: nfa.begin,
        };
        let mut eps: Vec<HashSet<usize>> = vec![HashSet::new(); nfa.nodes];
        let mut edges: Vec<Vec<Charset>> = vec![vec![charset!(); nfa.nodes]; nfa.nodes];
        for (a, b) in nfa.eps_edges {
            eps[a].insert(b);
        }
        for (a, b, s) in nfa.edges {
            edges[a][b] = s.clone();
            for c in s.iter() {
                let n = res.nodes[a].edges.entry(c).or_insert(HashSet::new());
                n.insert(b);
            }
        }
        for (n, g) in nfa.head {
            res.nodes[n].head.insert(g);
        }
        for (n, g) in nfa.tail {
            res.nodes[n].tail.insert(g);
        }
        return (res, eps, edges);
    }

    fn process(&mut self, eps: Vec<HashSet<usize>>, edges: Vec<Vec<Charset>>) {
        let mut used = vec![false; self.nodes.len()];
        for n in 0..self.nodes.len() {
            self.chain_dfs(&mut used, &eps, &edges, n, n);
            used.fill(false);
        }
    }

    fn chain_dfs(
        &mut self,
        used: &mut Vec<bool>,
        eps: &Vec<HashSet<usize>>,
        edges: &Vec<Vec<Charset>>,
        p: usize,
        n: usize,
    ) {
        used[n] = true;
        for i in 0..self.nodes.len() {
            if used[i] || !eps[n].contains(&i) {
                continue;
            }
            self.chain_dfs(used, eps, edges, p, i);
            let (np, ni) = mut_pair(&mut self.nodes, n, i);
            np.head.extend(&ni.head);
            np.tail.extend(&ni.tail);
        }
        for i in 0..self.nodes.len() {
            for c in edges[n][i].iter() {
                let s = self.nodes[p].edges.entry(c).or_insert(HashSet::new());
                s.insert(i);
            }
        }
    }
}

impl Automation for NFA {
    fn nodes(&self) -> impl Iterator<Item = (HashSet<usize>, HashSet<usize>)> {
        let s: HashSet<usize> = HashSet::new();
        std::iter::once((s.clone(), s.clone()))
            .chain(self.nodes.iter().map(|n| (n.head.clone(), n.tail.clone())))
    }

    fn edges(&self) -> impl Iterator<Item = (usize, usize, Option<u8>)> {
        self.nodes
            .iter()
            .enumerate()
            .flat_map(|(a, n)| {
                n.edges
                    .iter()
                    .flat_map(move |(c, s)| s.iter().map(move |b| (a + 1, *b + 1, Some(*c))))
            })
            .chain(std::iter::once((0, self.begin + 1, None)))
    }
}

#[cfg(test)]
mod test_nfa {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn mut_pair_quick() {
        let mut v = vec![1, 2, 3, 4];
        let (mut a, mut b, mut c, mut d) = (1, 2, 3, 4);
        assert_eq!(mut_pair(&mut v, 3, 1), (&mut d, &mut b));
        assert_eq!(mut_pair(&mut v, 0, 1), (&mut a, &mut b));
        assert_eq!(mut_pair(&mut v, 2, 3), (&mut c, &mut d));
    }

    #[test]
    fn just_works() {
        let nfa = build_nfa(Lexer::new(b"a", Config::default())).unwrap();
        assert_eq!(nfa.nodes.len(), 2);
        assert_eq!(nfa.begin, 0);
        assert_eq!(
            nfa.nodes[0].edges,
            HashMap::from([(b'a', HashSet::from([1]))])
        );
        assert_eq!(nfa.nodes[1].edges, HashMap::new());
        assert_eq!(nfa.nodes[0].head, HashSet::new());
        assert_eq!(nfa.nodes[1].head, HashSet::new());
        assert_eq!(nfa.nodes[0].tail, HashSet::new());
        assert_eq!(nfa.nodes[1].tail, HashSet::new());
    }

    fn merge<T: Iterator<Item = HashSet<usize>>>(it: T) -> HashMap<usize, HashSet<usize>> {
        let mut r: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (i, e) in it.enumerate() {
            if !e.is_empty() {
                r.entry(i).or_insert(HashSet::new()).extend(e);
            }
        }
        return r;
    }

    #[test]
    fn from_uncooked() {
        let nfa = build_nfa(Lexer::new(b"\\A(a)\\Z|\\Ab\\Z", Config::default())).unwrap();
        /*
         * 2* 6* / 3* 7*
         *       -a- 1*
         * 8** <
         *      -b- 5*
         */
        assert_eq!(nfa.nodes.len(), 10);
        assert_eq!(nfa.begin, 8);
        assert_eq!(
            merge(nfa.nodes.iter().filter_map(|x| Some(x.head.clone()))),
            HashMap::from([
                (8, HashSet::from([0, 1])),
                (2, HashSet::from([0])),
                (6, HashSet::from([1]))
            ]),
        );
        assert_eq!(
            merge(nfa.nodes.iter().filter_map(|x| Some(x.tail.clone()))),
            HashMap::from([
                (1, HashSet::from([0])),
                (5, HashSet::from([1])),
                (3, HashSet::from([0])),
                (7, HashSet::from([1])),
            ]),
        );
        assert_eq!(
            nfa.nodes[8].edges,
            HashMap::from([(b'a', HashSet::from([1])), (b'b', HashSet::from([5]))])
        );
        assert_eq!(nfa.nodes[1].edges, HashMap::from([]));
        assert_eq!(nfa.nodes[5].edges, HashMap::from([]));

        let nfa = build_nfa(Lexer::new(b"((a|b)c((d)))", Config::default())).unwrap();
        /*
         *     -a- 1
         * 4 <       > -c- 7 -d- 9
         *     -b- 3
         */
        assert_eq!(nfa.nodes.len(), 10);
        assert_eq!(nfa.begin, 4);
        assert_eq!(
            merge(nfa.nodes.iter().filter_map(|x| Some(x.tail.clone()))),
            HashMap::new()
        );
        assert_eq!(
            merge(nfa.nodes.iter().filter_map(|x| Some(x.head.clone()))),
            HashMap::new()
        );
        assert_eq!(
            nfa.nodes[4].edges,
            HashMap::from([(b'a', HashSet::from([1])), (b'b', HashSet::from([3]))])
        );
        assert_eq!(
            nfa.nodes[1].edges,
            HashMap::from([(b'c', HashSet::from([7]))])
        );
        assert_eq!(
            nfa.nodes[3].edges,
            HashMap::from([(b'c', HashSet::from([7]))])
        );
        assert_eq!(
            nfa.nodes[7].edges,
            HashMap::from([(b'd', HashSet::from([9]))])
        );
    }
}
