/*
 * common interface for all automation
 * Graphviz(T) -- printable url link to graph visualizer
 */

pub struct AutomationInfo {
    nodes: usize,
    begin: usize,
    epsilon: Option<u8>,
}

pub trait Automation {
    fn info(&self) -> AutomationInfo;
    fn contains_edge(&self, a: usize, b: usize, c: u8) -> bool;
}

pub struct Graphviz<T>(T);
impl<T> fmt::Display for Graphviz<T>
where
    T: Automation,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "https://dreampuf.github.io/GraphvizOnline/#digraph%7B")?;
        write!(f, "%0A%20%200%20%5Bshape%3D%22point%22%5D%3B")?;
        for a in 1..=self.0.info().nodes {
            write!(f, "%0A%20%20{}%20%5Bshape%3D%22circle%22%5D%3B", a)?;
        }
        write!(f, "%0A%20%200%2D%3E{}%3B", self.0.info().begin + 1)?;
        for a in 1..=self.0.info().nodes {
            for b in 1..=self.0.info().nodes {
                for c in 0..=255u8 {
                    if self.0.contains_edge(a - 1, b - 1, c) {
                        write!(f, "%0A%20%20{}%2D%3E{}%20%5Blabel%3D%22", a, b)?;
                        if c.is_ascii_digit() || c.is_ascii_alphabetic() {
                            write!(f, "{}", c as char)?;
                        } else if c >= b' ' && c <= b'~' {
                            write!(f, "%{:02X}", c)?;
                        } else {
                            write!(f, "%5C%5C{}", c)?;
                        }
                        write!(f, "%22%5D%3B")?;
                    }
                }
            }
        }
        write!(f, "%0A%7D")
    }
}
