package shortest_path_server

import "testing"
import "reflect"
import _ "fmt"

func TestEdges(t *testing.T) {
	edges := newEdges(10)

	if _, err := edges.getEdge(0, 5); err == nil {
		t.Error("nonexistent edge should not be found")
	}
	edges.addEdge(0, 5, 5)
	if _, err := edges.getEdge(0, 5); err != nil {
		t.Error("existing edge should be found")
	}

	ns := edges.neighbors(0)
	if !reflect.DeepEqual(ns, []node{5}) {
		t.Errorf("bad neighborhood %v", ns)
	}

}

func TestDiGraph(t *testing.T) {
	g := NewDiGraph(10)
	g.AddEdge(0, 1, 1)
	g.AddEdge(0, 2, 2)
	g.AddEdge(1, 3, 3)
	g.AddEdge(2, 3, 3)

	if g.NumNodes() != 10 {
		t.Errorf("wrong number of nodes %+v", g)
	}

	if g.NumEdges() != 4 {
		t.Errorf("wrong number of edges %+v", g)
	}

	if g.InDegree(3) != 2 {
		t.Errorf("wrong outdegree %+v", g)
	}


	if g.OutDegree(0) != 2 {
		t.Errorf("wrong outdegree %+v", g)
	}

	if ns := g.Neighbors(0); !reflect.DeepEqual(ns, []node{1, 2}) {
		t.Errorf("wrong neighborhood %v %+v", ns, g)
	}
}

func TestTopSortKahn(t *testing.T) {
	g := NewDiGraph(3)
	g.AddEdge(0, 1, 1)
	g.AddEdge(1, 2, 2)
	g.AddEdge(0, 2, 3)
	ordered, _ := g.TopSortKahn()
	if !reflect.DeepEqual(ordered, []node{0, 1, 2}) {
		t.Errorf("wrong order %+v", ordered)
	}
}

func TestTopSortKahnCycleDetection(t *testing.T) {
	g := NewDiGraph(2)
	g.AddEdge(0, 1, 1)
	g.AddEdge(1, 0, 1)
	_, err := g.TopSortKahn()
	if err == nil {
		t.Errorf("cycle not detected %+v", g)
	}
}
