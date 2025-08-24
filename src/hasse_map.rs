use ordermap::OrderMap;
use petgraph::graph::DiGraph;
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use petgraph::algo::has_path_connecting;

pub struct Poset<K> {
    /// Stable order-of-appearance (OOA): key -> idx
    pub idx: OrderMap<K, usize>,
    /// Reverse index: idx -> key
    pub keys: Vec<K>,
    /// Hasse edges: successors as adjacency lists
    pub succ: Vec<Vec<usize>>,
    /// Oriented incomparabilities: for i, store all j>i with i ∥ j
    pub amb: Vec<Vec<usize>>,
}

impl<K: Ord + Eq + std::hash::Hash + Clone> Poset<K> {
    pub fn new() -> Self {
        Self { idx: OrderMap::new(), keys: Vec::new(), succ: Vec::new(), amb: Vec::new() }
    }

    fn add_key(&mut self, k: K) -> usize {
        if let Some(&i) = self.idx.get(&k) { return i; }
        let i = self.keys.len();
        self.idx.insert(k.clone(), i);
        self.keys.push(k);
        self.succ.push(Vec::new());
        self.amb.push(Vec::new());
        i
    }

    fn add_edge(&mut self, i: usize, j: usize) {
        if i != j && !self.succ[i].contains(&j) {
            self.succ[i].push(j);
        }
    }

    pub fn from_rows(rows: &[Vec<K>]) -> Self {
        let mut p = Poset::new();
        for row in rows {
            for k in row { let _ = p.add_key(k.clone()); }
        }
        for row in rows {
            for i in 0..row.len() {
                let u = p.idx[&row[i]];
                for j in (i+1)..row.len() {
                    let v = p.idx[&row[j]];
                    p.add_edge(u, v);
                }
            }
        }
        p.normalize();
        p
    }

    pub fn normalize(&mut self) {
        let n = self.keys.len();

        // Build a graph from current succ
        let mut g: DiGraph<(), ()> = DiGraph::new();
        let mut nodes = Vec::with_capacity(n);
        for _ in 0..n {
            nodes.push(g.add_node(()));
        }
        for (u, vs) in self.succ.iter().enumerate() {
            for &v in vs {
                g.add_edge(nodes[u], nodes[v], ());
            }
        }

        // 1. Reduce to Hasse edges
        let mut new_succ: Vec<Vec<usize>> = vec![Vec::new(); n];
        for u in 0..n {
            for &v in &self.succ[u] {
                // Temporarily skip edge u→v and see if v is still reachable
                // (i.e. there is an alternate path from u to v).
                // If not reachable, then keep this as a cover edge.
                let mut g2 = g.clone();
                // remove_edge returns Option, we don't care about result
                if let Some(eid) = g2.find_edge(nodes[u], nodes[v]) {
                    g2.remove_edge(eid);
                }
                if !has_path_connecting(&g2, nodes[u], nodes[v], None) {
                    new_succ[u].push(v);
                }
            }
        }

        // 2. Compute incomparabilities
        let mut new_amb: Vec<Vec<usize>> = vec![Vec::new(); n];
        for i in 0..n {
            for j in (i + 1)..n {
                let i_to_j = has_path_connecting(&g, nodes[i], nodes[j], None);
                let j_to_i = has_path_connecting(&g, nodes[j], nodes[i], None);
                if !i_to_j && !j_to_i {
                    new_amb[i].push(j);
                }
            }
        }
        for row in &mut new_succ { row.sort_unstable(); }
        for row in &mut new_amb { row.sort_unstable(); }

        self.succ = new_succ;
        self.amb = new_amb;
    }

    /// Produce one deterministic topological order (by smallest OOA index).
    /// Returns Err with the list of "stuck" keys if a cycle prevents a full order.
    pub fn topo_one(&self) -> Result<Vec<K>, Vec<K>> {
        // Build a petgraph DiGraph over the current keys/edges
        let mut g: DiGraph<(), ()> = DiGraph::new();
        let mut nodes: Vec<NodeIndex> = Vec::with_capacity(self.keys.len());

        // one node per key
        for _ in 0..self.keys.len() {
            nodes.push(g.add_node(()));
        }

        // add directed edges from succ
        for (u, succs) in self.succ.iter().enumerate() {
            for &v in succs {
                g.add_edge(nodes[u], nodes[v], ());
            }
        }

        // run topo sort
        match toposort(&g, None) {
            Ok(order) => {
                // order is Vec<NodeIndex> in reverse topological order,
                // so map back to your keys
                let mut out = Vec::with_capacity(order.len());
                for ix in order {
                    // each NodeIndex directly maps to your original idx
                    let idx = ix.index();
                    out.push(self.keys[idx].clone());
                }
                Ok(out)
            }
            Err(cycle) => {
                // cycle.node() gives you the NodeIndex of a problematic node
                let stuck_key = self.keys[cycle.node_id().index()].clone();
                Err(vec![stuck_key])
            }
        }
    }
}
