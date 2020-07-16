use std::cmp;
use std::slice::ChunksExact;
use std::convert::TryInto;
use std::convert::From;
use std::fs::File;
use std::io;
use std::io::Read;

#[derive(Debug)]
pub enum Error {
    DecodeError(&'static str),
    IOError(io::Error),
    ByteDecodeError,
}

impl From<io::Error> for Error {
    fn from(ioerr: io::Error) -> Error {
        Error::IOError(ioerr)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Edge {
    pub(crate) from: u16,
    pub(crate) to: u16,
    pub(crate) cost: u16,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Input {
    pub(crate) start: u16,
    pub(crate) end: u16,
    pub(crate) nodes: u16,
    pub(crate) edges: Vec<Edge>
}


impl Input {
    pub fn from_decimals(items: &mut dyn Iterator<Item=u16>) -> Result<Input, Error> {

        let start = items.next().ok_or(Error::DecodeError("start"))?;
        let end    = items.next().ok_or(Error::DecodeError("end"))?;
        let count = items.next().ok_or(Error::DecodeError("number of edges"))?;

        let mut input = Input {
            start, end,
            nodes: 0,
            edges: Vec::with_capacity(count as usize),
        };

        for i in 0..count {
            let from = items.next().ok_or(Error::DecodeError("edge source"))?;
            let to = items.next().ok_or(Error::DecodeError("edge target"))?;
            let cost = items.next().ok_or(Error::DecodeError("edge weight"))?;
            let edge = Edge{from, to, cost};
            input.nodes = cmp::max(input.nodes, cmp::max(edge.from, edge.to));
            input.edges.push(edge);
        }

        Ok(input)
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<Input, Error> {

        let converted = buffer
            .chunks_exact(2)
            .into_iter()
            .map(|pair| pair
                 .try_into()
                 .map(u16::from_le_bytes)
                 .or(Err(Error::ByteDecodeError))
            )
            .collect::<Result<Vec<u16>, Error>>()?
            ;

        Input::from_decimals(&mut converted.into_iter())
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use assert2::assert;

    use std::path::Path;

    #[test]
    fn from_decimals() {
        let inp = vec![1, 5, 9,
                       1, 2, 14,
                       1, 3, 9,
                       1, 4, 7,
                       2, 5, 9,
                       3, 2, 2,
                       3, 6, 11,
                       4, 3, 10,
                       4, 6, 15,
                       6, 5, 6,
        ];
        let r = Input::from_decimals(&mut inp.into_iter());
        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn from_bytes() {
        let as_hex = vec![
            0x01, 0x00, 0x05, 0x00, 0x09, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x0e, 0x00,
            0x01, 0x00, 0x03, 0x00, 0x09, 0x00,
            0x01, 0x00, 0x04, 0x00, 0x07, 0x00,
            0x02, 0x00, 0x05, 0x00, 0x09, 0x00,
            0x03, 0x00, 0x02, 0x00, 0x02, 0x00,
            0x03, 0x00, 0x06, 0x00, 0x0b, 0x00,
            0x04, 0x00, 0x03, 0x00, 0x0a, 0x00,
            0x04, 0x00, 0x06, 0x00, 0x0f, 0x00,
            0x06, 0x00, 0x05, 0x00, 0x06, 0x00,
        ];

        let as_dec = vec![
            1, 5, 9,
            1, 2, 14,
            1, 3, 9,
            1, 4, 7,
            2, 5, 9,
            3, 2, 2,
            3, 6, 11,
            4, 3, 10,
            4, 6, 15,
            6, 5, 6,
        ];

        let converted: Vec<u16> = as_hex
            .chunks_exact(2)
            .into_iter()
            .map(|x| u16::from_le_bytes(x.try_into().unwrap()))
            .collect()
            ;

        assert!(converted == as_dec, "{:?}", converted);

    }

    fn load_map_file(name: &'static str) -> Vec<u8> {
        let path = Path::new("./data").join(name);
        let p = path.to_str().unwrap();
        let mut f = File::open(p).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        buf
    }

    #[test]
    fn from_map1() {
        let map1 = load_map_file("map1.bin");

        let map1_expected = vec![
            // hexdump data/map1.bin -e '6/1 "%02u " "\n"'
            01, 00, 05, 00, 09, 00,
            01, 00, 02, 00, 14, 00,
            01, 00, 03, 00, 09, 00,
            01, 00, 04, 00, 07, 00,
            02, 00, 05, 00, 09, 00,
            03, 00, 02, 00, 02, 00,
            03, 00, 06, 00, 11, 00,
            04, 00, 03, 00, 10, 00,
            04, 00, 06, 00, 15, 00,
            06, 00, 05, 00, 06, 00,
        ];
        assert!(map1 == map1_expected, "{:?}", map1);

        let inp = Input::from_bytes(&map1);
        assert!(inp.is_ok(), "{:?}", inp);

        let map1_expected = Input {
            // hexdump data/map1.bin -e '3/2 "%02u " "\n"'
            start: 1,
            end: 5,
            nodes: 6,
            edges: vec![
                Edge{ from: 1, to: 2, cost: 14}, // 1
                Edge{ from: 1, to: 3, cost: 09}, // 2
                Edge{ from: 1, to: 4, cost: 07}, // 3
                Edge{ from: 2, to: 5, cost: 09}, // 4
                Edge{ from: 3, to: 2, cost: 02}, // 5
                Edge{ from: 3, to: 6, cost: 11}, // 6
                Edge{ from: 4, to: 3, cost: 10}, // 7
                Edge{ from: 4, to: 6, cost: 15}, // 8
                Edge{ from: 6, to: 5, cost: 06}, // 9
            ],
        };

        let inp = inp.unwrap();
        assert!(inp == map1_expected, "{:?}", map1_expected);
    }

    #[test]
    fn from_map2() {
        let data = load_map_file("map2.bin");
        let inp = Input::from_bytes(&data);
        assert!(inp.is_ok());
        let inp = inp.unwrap();

        // hexdump data/map2.bin -e '3/2 "%02u " "\n"'
        assert!(inp.start == 65535);
        assert!(inp.end == 32767);
        assert!(inp.edges.len() == 32768);

        println!("{} {} {} {}", inp.start, inp.end, inp.nodes, inp.edges.len());

    }
}
