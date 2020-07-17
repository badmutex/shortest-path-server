use std::net::{TcpListener, IpAddr, Ipv4Addr};
use std::io::{Read, Write};
use std::io;
use std::convert::From;
use std::str::FromStr;
use std::time::{Duration, Instant};

use crate::input;
use crate::graph;

#[derive(Debug)]
pub enum Error {}

impl From<input::Error> for Error {
    #[allow(unused)]
    fn from(err: input::Error) -> Error {
        unimplemented!()        // TODO:
    }
}

impl From<graph::Error> for Error {
    #[allow(unused)]
    fn from(err: graph::Error) -> Error {
        unimplemented!()        // TODO:
    }
}

impl From<io::Error> for Error {
    #[allow(unused)]
    fn from(err: io::Error) -> Error {
        unimplemented!()        // TODO:
    }
}

#[derive(Debug)]
pub struct Config {
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub read_timeout_millis: Option<u64>,
}

#[derive(Debug)]
pub struct Response(graph::PathResult);

impl Response {
    pub fn message(&self) -> String {
        self.0.path.as_ref()
            .map_or_else(|| format!("No path from '{}' to '{}'", self.0.start, self.0.dest),
                         |path|
                         format!("{} ({})",
                                 path.path.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("->"),
                                 path.cost,
                         )
            )
    }
}

pub fn shortest_path(stream: &mut dyn Read) -> Result<graph::PathResult, Error> {
    let timer = Instant::now();
    let inp = {
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes)?;
        input::Input::from_bytes(&bytes)?
    };
    let t_parse = timer.elapsed();
    println!("{} nodes {} edges\n  parsing: {} μs",
             inp.nodes, inp.edges.len(),
             t_parse.as_micros());

    let timer = Instant::now();
    let g = {
        let mut g = graph::DiGraph::new(1 + inp.nodes as usize);
        for edge in inp.edges.into_iter() {
            g.add_edge(
                edge.from as usize,
                edge.to as usize,
                edge.cost as usize
            )?;
        }
        g
    };
    let t_load = timer.elapsed();
    println!("  loading: {} μs", t_load.as_micros());

    let timer = Instant::now();
    let path = g.shortest_path(inp.start as usize, inp.end as usize)?;
    let t_search = timer.elapsed();
    println!("  search: {} μs", t_search.as_micros());

    eprintln!("{},{},{},{},{}",
              g.num_nodes, g.num_edges,
              t_parse.as_micros(),
              t_load.as_micros(),
              t_search.as_micros(),
    );
    Ok(path)
}


pub fn server(listener: TcpListener, cfg: &Config) -> Result<(), Error> {
    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.set_read_timeout(Some(Duration::from_millis(cfg.read_timeout_millis.unwrap_or(10))))?;
        let path = shortest_path(&mut stream)?;
        let resp = Response(path);
        stream.write(resp.message().as_bytes())?;
    }
    Ok(())
}

pub fn run(cfg: &Config) -> Result<(), Error> {
    let default_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let default_port = 7777;
    let addr = cfg.ip.as_ref().map(|s| IpAddr::from_str(s).unwrap()).unwrap_or(default_addr);
    let port = cfg.port.unwrap_or(default_port);
    let listener = TcpListener::bind((addr, port))?;
    server(listener, cfg)
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::thread;
    use assert2::assert;
    use std::net::TcpStream;

    #[test]
    fn test_map1() {
        let mut f = File::open("data/map1.bin").unwrap();
        let path = shortest_path(&mut f).unwrap();

        assert!(path.path.as_ref().unwrap().path == vec![1, 3, 2, 5], "{:?}", path);
        assert!(path.path.as_ref().unwrap().cost == 20, "{:?}", path);
    }

    #[test]
    fn test_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cfg = Config { ip: None, port: None, read_timeout_millis: Some(5), };
        thread::spawn(move || { server(listener, &cfg).unwrap(); });

        let data = {
            let mut f = File::open("data/map1.bin").unwrap();
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).unwrap();
            buf
        };

        let mut conn = TcpStream::connect(addr).unwrap();
        conn.write(&data).unwrap();

        let _resp = {           // TODO: check return value
            let mut resp = String::new();
            conn.read_to_string(&mut resp).unwrap();
            resp
        };
    }

    #[test]
    fn test_response_message_has_path() {
        let resp = Response (
            graph::PathResult {
                start: 1,
                dest: 42,
                path: Some(graph::Path { path: vec![1, 10, 42], cost: 20 }),
            }
        );
        assert!(resp.message() == "1->10->42 (20)", "{}", resp.message());
    }

    #[test]
    fn test_response_message_no_path() {
        let resp = Response (
            graph::PathResult {
                start: 1,
                dest: 42,
                path: None,
            }
        );
        assert!(resp.message() == "No path from '1' to '42'", "{}", resp.message());
    }

}
