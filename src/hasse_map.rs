use ordermap::OrderMap;

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
        // reachability matrix
        let mut reach: Vec<Vec<bool>> = vec![vec![false; n]; n];
        for s in 0..n {
            let mut st = vec![s];
            let mut seen = vec![false; n];
            while let Some(u) = st.pop() {
                for &v in &self.succ[u] {
                    if !seen[v] {
                        seen[v] = true;
                        reach[s][v] = true;
                        st.push(v);
                    }
                }
            }
        }
        // reduce edges to Hasse
        for u in 0..n {
            let mut implied = vec![false; n];
            for &v in &self.succ[u] {
                for w in 0..n {
                    if reach[v][w] { implied[w] = true; }
                }
            }
            self.succ[u].retain(|&w| !implied[w]);
        }
        // rebuild incomparabilities
        for i in 0..n { self.amb[i].clear(); }
        for i in 0..n {
            for j in (i+1)..n {
                if !reach[i][j] && !reach[j][i] {
                    self.amb[i].push(j);
                }
            }
        }
        for row in &mut self.amb { row.sort_unstable(); }
        for row in &mut self.succ { row.sort_unstable(); }
    }

    /// Produce one deterministic topological order (by smallest OOA index).
    /// Returns Err with the list of "stuck" keys if a cycle prevents a full order.
    pub fn topo_one(&self) -> Result<Vec<K>, Vec<K>> {
        let n = self.keys.len();
        let mut indeg = vec![0usize; n];
        for u in 0..n {
            for &v in &self.succ[u] {
                indeg[v] += 1;
            }
        }

        let mut avail: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
        let mut out = Vec::with_capacity(n);

        while !avail.is_empty() {
            // pick smallest index available (deterministic)
            let u_pos = avail.iter().enumerate().min_by_key(|(_, &x)| x).unwrap().0;
            let u = avail.remove(u_pos);

            out.push(self.keys[u].clone());

            for &v in &self.succ[u] {
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    let pos = avail.binary_search(&v).err().unwrap();
                    avail.insert(pos, v);
                }
            }
        }

        if out.len() == n {
            Ok(out)
        } else {
            // remaining nodes with indegree > 0 are part of (or blocked by) a cycle
            let stuck: Vec<K> = (0..n)
                .filter(|&i| indeg[i] > 0)
                .map(|i| self.keys[i].clone())
                .collect();
            Err(stuck)
        }
    }
}
