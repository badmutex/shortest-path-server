package shortest_path_server

import "testing"
import "path"
import "bufio"
import "os"
import "io"


func TestEdge(t *testing.T) {
	e := NewEdge(0, 1, 2)
	if e.src != 0 || e.dst != 1 || e.cost != 2 {
		t.Errorf("wrong edge: %+v", e)
	}
}

func ReadDataFile(name string) (io.Reader, error) {
	p := path.Join("data", name)
	f, err := os.Open(p)
	if err != nil {
		return nil, err
	}
	return bufio.NewReader(f), nil
}


func DataFileTester(t *testing.T, name string, src, dst, edges uint16) {
	r, err := ReadDataFile(name)
	if err != nil {
		t.Errorf("%v test: could not read file: %+v", name, err)
	}
	inp, err := ParseInput(r)
	if err != nil {
		t.Errorf("%v test: could not parse the bytes: %+v", name, err)
	}
	if inp.src != src || inp.dst != dst || inp.numEdges != edges || len(inp.edges) != int(edges) {
		t.Errorf("%v test: wrong input: %+v", name, inp)
	}
}

func TestMapFiles(t *testing.T) {
	DataFileTester(t, "map1.bin", 1, 5, 9)
	DataFileTester(t, "map2.bin", 65535, 32767, 32768)
	DataFileTester(t, "map3.bin", 1, 4, 4)
	DataFileTester(t, "map4.bin", 1, 2, 2)
}
