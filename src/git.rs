use std::io::Read;

use flate2::read::ZlibDecoder;

pub mod helpers;
pub mod packfile;

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
    let _header = &data[0..4];
    let _version = &data[4..8];
    let entries = u32::from_be_bytes(data[8..12].try_into().unwrap());

    let mut readed = 12;

    for _ in 0..entries {
        let (pack, total) = get_object(data, readed);
        readed += total;

        match pack.kind {
            PackfileOBJ::COMMIT | PackfileOBJ::BLOB => {
                let data = pack.data;
                let _data = String::from_utf8(data).unwrap();
                //                println!("{:#?}", pack.kind);
                //                println!("{data}");
            }
            x => {
                println!("{:#?}", x);
            }
        }
    }
    let hash = hex::encode(&data[readed..]);
    println!("done {hash}");
}

fn get_object(raw: &[u8], pos: usize) -> (PackObj, usize) {
    let data = &raw[pos..];
    let mut buf;
    let mut size: usize = 0;
    let mut shift = 0;
    let mut obj: u8 = 0;

    let mut consume = 0;

    println!("====START HEADER====");
    loop {
        buf = data[consume];
        consume += 1;
        if obj == 0 {
            obj = buf & 0b01110000;
            obj >>= 4;
        }
        size |= ((buf & if size == 0 { 0xf } else { 0x7f }) as usize) << shift;
        shift += if shift == 0 { 4 } else { 7 };

        if buf & 0x80 == 0 {
            break;
        }
    }

    println!("====END HEADER==== {size}");

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
        let (offset, readed) = offset_decode(&data[consume..]);
        consume += readed;

        println!("obj peek back {offset} {pos}");
        let (pack, _) = get_object(raw, pos - offset);

        println!("obj peek back {offset} {pos} kind {:#?}", pack.kind);
    }

    let mut z = ZlibDecoder::new(&data[consume..]);
    let mut content = Vec::new();
    z.read_to_end(&mut content).unwrap();

    let pack = PackObj {
        kind: obj,
        data: content,
    };

    println!(
        "===== end parse, read {} actual {}",
        z.total_in(),
        z.total_out()
    );

    (pack, (z.total_in() + consume as u64) as usize)
}

fn offset_decode(data: &[u8]) -> (usize, usize) {
    let mut offset: usize = 0;
    let mut consume = 0;
    let mut buf;

    loop {
        buf = data[consume];
        consume += 1;

        offset <<= 7;
        offset |= (buf & 0x7f) as usize;

        if buf & 0x80 == 0 {
            break;
        }
        offset += 1;
    }

    (offset, consume)
}
