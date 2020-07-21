package shortest_path_server

import "fmt"
import "sort"
import "container/list"
import "math"

type node = uint16
type Weight = uint

type edges struct {
	edge []map[node]Weight
}

func newEdges(numnodes uint) edges {
	edge := make([]map[node]Weight, numnodes)
	for s := uint(0); s < numnodes; s++ {
		edge[s] = make(map[node]Weight)
	}
	return edges{edge: edge}
}

func (e *edges) getEdge(s, t node) (Weight, error) {
	var cost Weight = 0
	var err error = nil
	var present = false
	if cost, present = e.edge[s][t]; !present {
		err = fmt.Errorf("edge %v->%v does not exist", s, t)
	}
	return cost, err
}

func (e *edges) neighbors(s node) []node {
	neighbors := e.edge[s]
	if len(neighbors) == 0 {
		return make([]node, 0)
	}

	ns := make([]node, len(neighbors))
	i := 0
	for t := range neighbors {
		ns[i] = t
		i += 1
	}
	sort.Slice(ns, func(i, j int) bool { return ns[i] < ns[j] })
	return ns
}

func (e *edges) addEdge(s, t node, w Weight) {
	e.edge[s][t] = w
}

type DiGraph struct {
	numNodes, numEdges  uint
	inDegree, outDegree map[node]uint
	edges               edges
}

type Path struct {
	Path []node
	Cost Weight
}

type PathResult struct {
	Start, Dest node
	Path        *Path
}

func NewDiGraph(numnodes uint) DiGraph {
	return DiGraph{
		numNodes:  numnodes,
		numEdges:  0,
		inDegree:  make(map[node]uint),
		outDegree: make(map[node]uint),
		edges:     newEdges(numnodes),
	}
}

func (g DiGraph) NumNodes() uint {
	return g.numNodes
}

func (g DiGraph) NumEdges() uint {
	return g.numEdges
}

func (g DiGraph) InDegree(n node) uint {
	return g.inDegree[n]
}

func (g DiGraph) OutDegree(n node) uint {
	return g.outDegree[n]
}

func (g *DiGraph) AddEdge(s, t node, w Weight) {
	g.outDegree[s] += 1
	g.inDegree[t] += 1
	g.numEdges += 1
	g.edges.addEdge(s, t, w)
}

func (g DiGraph) GetEdge(s, t node) *Weight {
	if w, e := g.edges.getEdge(s, t); e == nil {
		return &w
	}
	return nil
}

func (g DiGraph) Neighbors(n node) []node {
	return g.edges.neighbors(n)
}

func (g DiGraph) TopSortKahn() ([]node, error) {
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

	// ****************************************************************
	// first, some setup...

	// need to make decisions based on in-degree
	inDegree := make([]uint, g.numNodes)
	for s := uint(0); s < g.numNodes; s++ {
		inDegree[s] = g.inDegree[node(s)]
	}

	// the E in the algorithm above
	// we'll be mutating this and dont want to cause side-effects
	edgesToConsider := make([]map[node]Weight, g.numNodes)
	for s := 0; s < len(g.edges.edge); s++ {
		e0 := g.edges.edge[s]
		e := make(map[node]Weight, len(e0))
		for t, w := range e0 {
			e[t] = w
		}
		edgesToConsider[s] = e
	}

	// ****************************************************************
	// lets do this

	L := make([]node, g.numNodes)

	S := list.New()
	for n, deg := range inDegree {
		if deg > 0 {
			continue
		}
		S.PushBack(node(n))
	}

	var iL int = 0
	for S.Len() > 0 {
		nd := S.Front()
		S.Remove(nd)
		n := node(nd.Value.(node))

		L[iL] = n
		iL += 1

		for m := range edgesToConsider[n] {
			delete(edgesToConsider[n], m)
			inDegree[m] -= 1
			if inDegree[m] <= 0 {
				S.PushBack(m)
			}
		}
	}

	var err error = nil
	for s := 0; s < len(edgesToConsider); s++ {
		if len(edgesToConsider[s]) > 0 {
			err = fmt.Errorf("cycle detected")
			break
		}
	}

	return L, err

}

func (g DiGraph) ShortestPath(source, target node) (result PathResult, err error) {
	// d <- track shortest paths from s to d[i] (where i is all other nodes in the graph)
	// p <- predecessors s.t. p[u] is the predecessor of u in shortest path s to u
	//
	// for each u in topsort(G)
	//   for each v s.t. and edge u->v exists in E(G)
	//     w := weight(u -> v)
	//     if d[v] > d[u] + w
	//       d[v] := d[u] + w
	//       p[v] := u

	result = PathResult{Start: source, Dest: target}
	err = nil

	dist := make([]Weight, g.numNodes)
	for s := Weight(0); s < g.numNodes; s++ {
		dist[s] = math.MaxInt64
	}
	dist[source] = Weight(0)

	pred := make([]*node, g.numNodes)

	ordered, err := g.TopSortKahn()
	if err != nil {
		return
	}

	for _, u := range ordered {
		for _, v := range g.Neighbors(u) {
			w := g.GetEdge(u, v)
			if w == nil {
				return PathResult{}, fmt.Errorf("known edge %v->%v is not weighted", source, target)
			}
			if dist[v] > dist[u] + *w {
				dist[v] = dist[u] + *w
				pred[v] = &u
			}
		}
	}

	path := make([]node, 0)
	curr := &target
	for curr != nil && *curr != source {
		path = append(path, *curr)
		curr = pred[*curr]
	}

	p := Path{
		Path: path,
		Cost: dist[target],
	}

	result.Path = &p
	return result, err

}
