use std::vec::Vec;
use std::collections::{HashMap, LinkedList};
use std::default::Default;

type Weight = usize;

#[derive(Debug)]
pub enum Error {
    UndefinedNode(usize),
    EdgeExists(usize, usize, Weight),
    CycleExists,
}

type Res = Result<(), Error>;


#[derive(Debug)]
pub struct DiGraph{
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,

    pub(crate) out_degrees: Vec<usize>,
    pub(crate) in_degrees: Vec<usize>,

    /// The adjacency matrix containing weights.
    /// This assumes that there is at most one s -> t edge
    pub(crate) adjacency: Matrix<Weight>,
}

#[derive(Debug)]
pub struct Path {
    pub path: Vec<usize>,
    pub cost: Weight
}

#[derive(Debug)]
pub struct PathResult {
    pub start: usize,
    pub dest: usize,
    pub path: Option<Path>
}

#[derive(Debug)]
pub struct Matrix<T> {
    pub rows: usize,
    m: Vec<HashMap<usize, T>>,
}


impl <T> Matrix<T> where T: std::fmt::Display+std::fmt::Debug {
    pub fn new(rows: usize) -> Matrix<T>
        where T: Default + Clone
    {
        let mut m = Vec::with_capacity(rows);
        m.resize(rows, HashMap::new());
        Matrix{
            rows, m
        }
    }

    #[inline]
    pub fn set (&mut self, i: usize, j: usize, val: T) {
        let row = &mut self.m[i];
        row.insert(j, val);
    }

    #[inline]
    pub fn get(&self, i: usize, j: usize) -> Option<&T> {
        self.m.get(i).and_then(|r| r.get(&j))
    }
}

impl DiGraph {
    pub fn new(nodes: usize) -> DiGraph
    {
        let mut out_degrees = Vec::with_capacity(nodes);
        out_degrees.resize(nodes, 0);

        let mut in_degrees = Vec::with_capacity(nodes);
        in_degrees.resize(nodes, 0);
        DiGraph {
            num_nodes: nodes,
            num_edges: 0,
            out_degrees, in_degrees,
            adjacency: Matrix::new(nodes)
        }
    }

    #[allow(unused)]
    pub fn node_exists(&self, n: usize) -> bool {
        n <= self.num_nodes
    }


    pub fn edge_exists(&self, s: usize, t: usize) -> bool {
        let w = self.adjacency.get(s, t);
        w.is_some() && *w.unwrap() > 0
    }

    pub fn get_edge(&self, s: usize, t: usize) -> Option<&Weight> {
        self.adjacency.get(s, t)
    }

    pub fn add_edge(&mut self, s: usize, t: usize, w: Weight) -> Res
    {
        if s > self.num_nodes { return Err(Error::UndefinedNode(s)) }
        if t > self.num_nodes { return Err(Error::UndefinedNode(t)) }
        if self.edge_exists(s, t) { return Err(Error::EdgeExists(s, t, *self.get_edge(s, t).unwrap())) }

        self.out_degrees[s] += 1;
        self.in_degrees[t] += 1;
        self.num_edges += 1;
        self.adjacency.set(s, t, w);
        Ok(())
    }

    pub fn neighbors(&self, s: usize) -> Vec<usize> {
        let mut ns: Vec<usize> = self.adjacency.m[s].keys().map(|v| *v).collect();
        ns.sort();
        ns
    }

    /// Topologically sort the nodes using Kahn algorithm (1962)
    pub fn topsort_kahn(&self) -> Result<Vec<usize>, Error> {
        // The algorithm is:
        //
        // L <- sorted elements to be emitted
        // S <- set of nodes with no incoming edge
        //
        // while S is not empty:
        //   n <- pop(S)
        //   append(L, n)
        //   for m such that the edge (n -> m) exists in edge set E
        //     pop(E, n->m)
        //     if in-degree(m) = 0
        //       append(S, m)
        //
        // if |E(G)| > 0
        //   error: cycle detected
        // else
        //   return L

        let mut emit = LinkedList::new();

        let mut in_degree = Vec::with_capacity(self.num_nodes);
        in_degree.resize(self.num_nodes, 0);
        // ^ we make decisions based on in-degree so track it here.

        // copy out the edges so we can pop then
        let mut edges: Vec<LinkedList<usize>> = {
            let mut edges = Vec::with_capacity(self.num_nodes);
            edges.resize(self.num_nodes, LinkedList::new());
            for s in 0..self.num_nodes {
                // let mut es = LinkedList::new();
                for t in self.neighbors(s) {
                    edges[s].push_back(t);
                    in_degree[t] += 1;
                }
            }
            edges
        };

        let mut start_nodes: LinkedList<usize> = {
            let mut start_nodes = LinkedList::new();
            for n in 0..self.num_nodes {
                if self.in_degrees[n] == 0 {
                    start_nodes.push_back(n)
                }
            }
            start_nodes
        };

        while let Some(n) = start_nodes.pop_front() {
            emit.push_back(n);

            while let Some(m) = edges[n].pop_front() {
                in_degree[m] -= 1;
                if in_degree[m] <= 0 {
                    start_nodes.push_back(m);
                }
            }
        }

        if edges.iter().map(|es| es.len()).sum::<usize>() > 0 {
            return Err(Error::CycleExists)
        }

        Ok(emit.into_iter().collect())
    }

    /// Find the shortest path between nodes /s/ and /t/
    pub fn shortest_path(&self, s: usize, t: usize) -> Result<PathResult, Error> {

        // d <- track shortest paths from s to d[i] (where i is all other nodes in the graph)
        // p <- predecessors s.t. p[u] is the predecessor of u in shortest path s to u
        //
        // for each u in topsort(G)
        //   for each v s.t. and edge u->v exists in E(G)
        //     w := weight(u -> v)
        //     if d[v] > d[u] + w
        //       then d[v] := d[u] + w
        //       else p[v] := u

        let mut dist = Vec::with_capacity(self.num_nodes);
        dist.resize(self.num_nodes, std::f64::INFINITY);
        dist[s] = 0.0;

        let mut pred = Vec::with_capacity(self.num_nodes);
        pred.resize(self.num_nodes, None);

        for u in self.topsort_kahn()? {
            for v in self.neighbors(u) {
                let w = self.adjacency.get(u, v).unwrap();
                let w = *w as f64;
                if dist[v] > dist[u] + w {
                    dist[v] = dist[u] + w;
                    pred[v] = Some(u);
                }
            }
        }

        if dist[t] == std::f64::INFINITY {
            return Ok(PathResult {
                start: s, dest: t, path: None,
            })
        }

        let path = {
            let mut path = LinkedList::new();
            // walk backwards from t to s

            let mut node = t;
            path.push_front(node);
            while let Some(n) = pred[node] {
                node = n;
                path.push_front(node);
            }
            path
        };

        let path = Path {
            path: path.into_iter().collect(),
            cost: dist[t].floor() as usize,
        };

        Ok(PathResult{ start: s, dest: t, path: Some(path), })
    }




}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{assert, check};

    #[test]
    fn matrix() {
        let mut m: Matrix<u32> = Matrix::new(5);
        assert_eq!(m.rows, 5);

        m.set(0, 1, 42);
        let r = m.get(0, 1);
        assert!(r.is_some());
        assert_eq!(42, *r.unwrap());
    }

    #[test]
    fn neighbors() {
        let mut g = DiGraph::new(3);

        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 2);
        g.add_edge(1, 0, 4);
        g.add_edge(2, 1, 3);

        assert!(g.neighbors(0) == vec![1, 2],  "{:?}", g);
        assert!(g.neighbors(1) == vec![0],  "{:?}", g);
        assert!(g.neighbors(2) == vec![1],  "{:?}", g);

    }

    #[test]
    fn topsort_kahn(){
        let mut g = DiGraph::new(3);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 2);
        g.add_edge(0, 2, 3);

        let r = g.topsort_kahn();
        assert!(r.is_ok());
        assert!(r.unwrap() == vec![0, 1, 2])
    }

    #[test]
    fn shortest_path() {
        let mut g = DiGraph::new(4);
        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 10);
        g.add_edge(1, 3, 1);
        g.add_edge(2, 3, 1);

        let r = g.shortest_path(0, 3);
        assert!(r.is_ok());
        let path = r.unwrap();
        assert!(path.path.as_ref().unwrap().path == vec![0, 1, 3]);
        assert!(path.path.as_ref().unwrap().cost == 2);
    }

    #[test]
    fn graph() {
        let mut g = DiGraph::new(3);
        assert!(g.node_exists(1), "{:?}", g);

        assert!(!g.edge_exists(0, 1));
        let r = g.add_edge(0, 1, 42);
        assert!(r.is_ok(), "{:?}", r);
        assert!(g.edge_exists(0, 1));

        assert!(g.out_degrees[0] == 1);
        assert!(g.in_degrees[1] == 1);

        assert!(!g.edge_exists(5, 10));
        let r = g.add_edge(5, 10, 1);
        assert!(r.is_err());

        assert!(!g.edge_exists(1, 2));
        g.add_edge(1, 2, 2);
        assert!(g.edge_exists(1, 2));
    }

}
