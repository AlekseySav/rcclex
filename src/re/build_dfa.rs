/*
 * Build DFA
 */

use std::collections::VecDeque;

pub struct DFA {
    nodes: Vec<HashMap<u8, usize>>,
    head: Vec<HashSet<usize>>,
    tail: Vec<HashSet<usize>>,
}

pub fn build_dfa(nfa: NFA) -> DFA {
    let mut dfa = DFA {
        nodes: vec![],
        head: vec![],
        tail: vec![],
    };
    dfa.init_from_nfa(nfa);
    return dfa;
}

impl DFA {
    fn node(&mut self, nfa: &NFA, out: &mut Vec<HashSet<usize>>, origin: HashSet<usize>) -> usize {
        let node = self.nodes.len();
        self.nodes.push(HashMap::new());
        let mut head: HashSet<usize> = HashSet::new();
        let mut tail: HashSet<usize> = HashSet::new();
        for n in origin.iter() {
            head.extend(&nfa.nodes[*n].head);
            tail.extend(&nfa.nodes[*n].tail);
        }
        self.head.push(head);
        self.tail.push(tail);
        out.push(origin);
        return node;
    }

    fn init_from_nfa(&mut self, nfa: NFA) {
        let mut output: Vec<HashSet<usize>> = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::new(); // index within output
        queue.push_back(self.node(&nfa, &mut output, HashSet::from([nfa.begin])));

        while queue.len() != 0 {
            let id_from = queue.pop_front().unwrap();

            for c in Charset::ALL.iter() {
                let from_set = &output[id_from];
                let mut to: HashSet<usize> = HashSet::new();
                for from in from_set {
                    match nfa.nodes[*from].edges.get(&c) {
                        None => (),
                        Some(s) => to.extend(s),
                    }
                }

                if to.len() == 0 {
                    continue;
                }

                match output.iter().position(|s| *s == to) {
                    Some(id_to) => _ = self.nodes[id_from].insert(c, id_to),
                    None => {
                        let id_to = self.node(&nfa, &mut output, to);
                        queue.push_back(id_to);
                        self.nodes[id_from].insert(c, id_to);
                    }
                }
            }
        }
    }
}

impl Automation for DFA {
    fn nodes(&self) -> impl Iterator<Item = (HashSet<usize>, HashSet<usize>)> {
        (0..self.nodes.len()).map(|n| (self.head[n].clone(), self.tail[n].clone()))
    }

    fn edges(&self) -> impl Iterator<Item = (usize, usize, Option<u8>)> {
        self.nodes
            .iter()
            .enumerate()
            .flat_map(|(a, n)| n.iter().map(move |(c, b)| (a, *b, Some(*c))))
    }
}

#[cfg(test)]
mod test_dfa {
    use super::*;

    #[test]
    fn just_works() {
        let nfa = build_dfa(build_nfa(Lexer::new(b"a")).unwrap());
        assert_eq!(nfa.nodes.len(), 2);
        assert_eq!(nfa.nodes[0], HashMap::from([(b'a', 1)]));
        assert_eq!(nfa.nodes[1], HashMap::new());
    }
}
