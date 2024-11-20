/*
 * First stage of building DFA
 * https://www.geeksforgeeks.org/program-implement-nfa-epsilon-move-dfa-conversion/
 *
 * Make a graph matrix from list of edges, apply eps-closures
 */

pub struct NFA1 {
    nodes: Vec<Vec<Charset>>,
    begin: usize,
}

impl Automation for NFA1 {
    fn info(&self) -> AutomationInfo {
        return AutomationInfo {
            nodes: self.nodes.len(),
            begin: self.begin,
        };
    }

    fn contains_edge(&self, a: usize, b: usize, c: u8) -> bool {
        return a < self.nodes.len() && b < self.nodes.len() && self.nodes[a][b].contains(c);
    }
}

impl NFA1 {
    pub fn build(nfa: NFA) -> NFA1 {
        let mut nfa1 = NFA1 {
            nodes: vec![vec![Charset::new(); nfa.nodes]; nfa.nodes],
            begin: nfa.begin,
        };
        for e in &nfa.edges {
            nfa1.nodes[e.a][e.b] = nfa1.nodes[e.a][e.b] | e.c;
        }

        let mut used: Vec<bool> = vec![false; nfa.nodes];
        for n in 0..nfa.nodes {
            nfa1.chain_dfs(&mut used, n, n, nfa.epsilon);
            used.fill(false);
        }

        let rm_eps = Charset::char(nfa.epsilon).invert();
        for i in 0..nfa.nodes {
            for j in 0..nfa.nodes {
                nfa1.nodes[i][j] = nfa1.nodes[i][j] & rm_eps;
            }
        }
        return nfa1;
    }

    fn chain_dfs(&mut self, used: &mut Vec<bool>, p: usize, n: usize, eps: u8) {
        used[n] = true;
        for i in 0..self.nodes.len() {
            if used[i] || !self.nodes[n][i].contains(eps) {
                continue;
            }
            self.chain_dfs(used, p, i, eps);
        }
        for i in 0..self.nodes.len() {
            self.nodes[p][i] = self.nodes[p][i] | self.nodes[n][i];
        }
    }
}
