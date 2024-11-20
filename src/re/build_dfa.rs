/*
 * Second stage of building DFA
 * https://www.geeksforgeeks.org/program-implement-nfa-epsilon-move-dfa-conversion/
 *
 * Convert 1-NFA to DFA
 */

use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

pub struct DFA {
    nodes: Vec<HashMap<u8, usize>>,
}

impl Automation for DFA {
    fn info(&self) -> AutomationInfo {
        return AutomationInfo {
            nodes: self.nodes.len(),
            begin: 0,
        };
    }

    fn contains_edge(&self, a: usize, b: usize, c: u8) -> bool {
        let v = a < self.nodes.len() && self.nodes[a].get(&c) == Some(&b);
        return v;
    }
}

impl DFA {
    pub fn build(nfa: NFA1) -> DFA {
        let mut dfa = DFA { nodes: Vec::new() };
        dfa.init(nfa);
        return dfa;
    }

    fn node(&mut self, output: &mut Vec<HashSet<usize>>, origin: HashSet<usize>) -> usize {
        let node = self.nodes.len();
        self.nodes.push(HashMap::new());
        output.push(origin);
        return node;
    }

    fn init(&mut self, nfa: NFA1) {
        let mut output: Vec<HashSet<usize>> = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::new(); // index within output
        queue.push_back(self.node(&mut output, HashSet::from([nfa.begin])));

        while queue.len() != 0 {
            let id_from = queue.pop_front().unwrap();

            for c in 0..=255 {
                let from_set = &output[id_from];
                let mut to: HashSet<usize> = HashSet::new();
                for from in from_set {
                    for n in 0..nfa.nodes.len() {
                        // FIXME
                        if nfa.nodes[*from][n].contains(c) {
                            to.insert(n);
                        }
                    }
                }

                if to.len() == 0 {
                    continue;
                }

                match output.iter().position(|s| *s == to) {
                    Some(id_to) => _ = self.nodes[id_from].insert(c, id_to),
                    None => {
                        let id_to = self.node(&mut output, to);
                        queue.push_back(id_to);
                        self.nodes[id_from].insert(c, id_to);
                    }
                }
            }
        }
    }
}
