package shortest_path_server

import "fmt"
import "encoding/binary"
import "io"

type edge struct {
	src, dst, cost uint16
}

type input struct {
	src, dst, numEdges uint16
	edges []edge
}

func NewEdge(s, t, w uint16) edge {
	return edge {
		src: s,
		dst: t,
		cost: w,
	}
}

func ParseInput(r io.Reader) (res input, err error) {
	buff := make([]byte, 6)
	n, e := r.Read(buff)
	if err != nil {
		err = e
		return
	}
	if n != 6 {
		err = fmt.Errorf("header expected to read 6 bytes but got %v", n)
		return
	}

	res.src = binary.LittleEndian.Uint16(buff[0:2])
	res.dst = binary.LittleEndian.Uint16(buff[2:4])
	res.numEdges = binary.LittleEndian.Uint16(buff[4:6])

	res.edges = make([]edge, res.numEdges)

	for i := 0; i < int(res.numEdges); i++ {
		_, e := r.Read(buff)
		if e != nil {
			err = e
			return
		}

		s := binary.LittleEndian.Uint16(buff[0:2])
		t := binary.LittleEndian.Uint16(buff[2:4])
		w := binary.LittleEndian.Uint16(buff[4:6])

		res.edges[i] = NewEdge(s, t, w)
	}

	return
}
