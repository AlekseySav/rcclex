/*
 * Build CDFA
 */

pub struct CDFA {
    nodes: Vec<[usize; 256]>,
}

impl Automation for CDFA {
    fn info(&self) -> AutomationInfo {
        return AutomationInfo {
            nodes: self.nodes.len(),
            begin: 0,
        };
    }

    fn contains_edge(&self, a: usize, b: usize, c: u8) -> bool {
        return a < self.nodes.len() && self.nodes[a][c as usize] == b;
    }
}

impl CDFA {
    pub fn build(dfa: DFA) -> CDFA {
        let mut cdfa = CDFA {
            nodes: vec![[dfa.nodes.len(); 256]; dfa.nodes.len()],
        };

        for (n, set) in dfa.nodes.iter().enumerate() {
            for e in set {
                cdfa.nodes[n][*e.0 as usize] = *e.1;
            }
        }

        return cdfa;
    }
}
