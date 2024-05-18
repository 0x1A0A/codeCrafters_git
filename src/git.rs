use std::io::{BufRead, BufReader, Read};

use flate2::read::ZlibDecoder;

#[derive(Debug, PartialEq, Eq)]
enum PackfileOBJ {
    COMMIT,
    TREE,
    BLOB,
    TAG,
    OFS,
    REF,
}

struct PackObj {
    kind: PackfileOBJ,
    data: Vec<u8>,
}

pub fn packfile_parse(data: &Vec<u8>) {
    let mut reader = BufReader::new(data.as_slice());

    let mut header = [0; 4];
    let mut version = [0; 4];
    let mut entries = [0; 4];
    reader.read_exact(&mut header).unwrap();
    reader.read_exact(&mut version).unwrap();
    reader.read_exact(&mut entries).unwrap();

    let entries = u32::from_be_bytes(entries);

    let mut readed = 12;

    for _ in 0..entries {
        let (pack, total) = packfile_get_size(&mut &data[readed..]);
        readed += total;

        match pack.kind {
            PackfileOBJ::COMMIT | PackfileOBJ::BLOB => {
                let data = pack.data;
                let data = String::from_utf8(data).unwrap();
                println!("{:#?}", pack.kind);
                println!("{data}");
            }
            x => {
                println!("{:#?}", x);
            }
        }
    }
    let hash = hex::encode(&data[readed..]);
    println!("done {hash}");
}

fn packfile_get_size(reader: &mut impl BufRead) -> (PackObj, usize) {
    let mut buf = [0; 1];
    let mut size: usize = 0;
    let mut shift = 0;
    let mut obj: u8 = 0;

    let mut consume = 0;

    loop {
        consume += 1;
        reader.read_exact(&mut buf).unwrap();
        if obj == 0 {
            obj = buf[0] & 0b01110000;
            obj >>= 4;
        }
        size |= ((buf[0] & if size == 0 { 0xf } else { 0x7f }) as usize) << shift;
        shift += if shift == 0 { 4 } else { 7 };

        if buf[0] & 0x80 == 0 {
            break;
        }
    }

    let obj = match obj {
        0b001 => PackfileOBJ::COMMIT,
        0b010 => PackfileOBJ::TREE,
        0b011 => PackfileOBJ::BLOB,
        0b100 => PackfileOBJ::TAG,
        0b110 => PackfileOBJ::OFS,
        0b111 => PackfileOBJ::REF,
        _ => panic!("Unregonized OBJ type"),
    };

    if obj == PackfileOBJ::OFS {
        shift = 0;
        let mut _nagative = 0;
        loop {
            consume += 1;
            reader.read_exact(&mut buf).unwrap();
            _nagative |= ((buf[0] & 0x7f) as usize) << shift;
            shift += if shift == 0 { 4 } else { 7 };

            if buf[0] & 0x80 == 0 {
                break;
            }
        }
    }

    let mut z = ZlibDecoder::new(reader);
    let mut content = Vec::new();
    z.read_to_end(&mut content).unwrap();

    let pack = PackObj {
        kind: obj,
        data: content,
    };

    (pack, (z.total_in() + consume) as usize)
}
