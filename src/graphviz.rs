/*
 * rcclex graphviz
 */

use std::collections::{HashMap, HashSet};

pub struct Graphviz<T>(T);

fn put(f: &mut std::fmt::Formatter, s: String) -> std::fmt::Result {
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            write!(f, "{}", c)?;
        } else {
            write!(f, "%{:02X}", c as u32)?;
        }
    }
    Ok(())
}

fn dfs(
    f: &mut std::fmt::Formatter,
    g: &HashMap<usize, HashMap<Option<u8>, HashSet<usize>>>,
    nodes: &HashMap<usize, (HashSet<usize>, HashSet<usize>)>,
    used: &mut HashSet<usize>,
    n: usize,
) -> std::fmt::Result {
    if used.contains(&n) {
        return Ok(());
    }
    used.insert(n);
    let default: (HashSet<usize>, HashSet<usize>) = (HashSet::new(), HashSet::new());
    let (h, t) = nodes.get(&n).unwrap_or(&default);
    if h.is_empty() && t.is_empty() {
        put(f, format!("  {n} [shape=\"point\"];\n"))?;
    } else {
        put(f, format!("  {n} [shape=\"circle\", fontsize=10, label=\""))?;
        for g in h {
            put(f, format!("A{g} "))?;
        }
        for g in t {
            put(f, format!("Z{g} "))?;
        }
        put(f, String::from("\"];\n"))?;
    }
    for (c, s) in g.get(&n).unwrap_or(&HashMap::new()) {
        for i in s {
            match c {
                Some(c) => put(f, format!("  {n}->{i} [label=\"{}\"];\n", charset!(*c)))?,
                None => put(f, format!("  {n}->{i};\n"))?,
            }
            dfs(f, g, nodes, used, *i)?;
        }
    }
    Ok(())
}

impl<T> std::fmt::Display for Graphviz<T>
where
    T: re::Automation,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut g: HashMap<usize, HashMap<Option<u8>, HashSet<usize>>> = HashMap::new();
        for (a, b, c) in self.0.edges() {
            g.entry(a)
                .or_insert(HashMap::new())
                .entry(c)
                .or_insert(HashSet::new())
                .insert(b);
        }
        write!(f, "https://dreampuf.github.io/GraphvizOnline/#digraph")?;
        put(f, format!("{{\n  rankdir=LR;\n"))?;
        dfs(
            f,
            &g,
            &&HashMap::from_iter(self.0.nodes().enumerate()),
            &mut HashSet::new(),
            0,
        )?;
        put(f, format!("}}\n"))
    }
}
