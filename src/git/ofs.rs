use core::panic;
use std::io::{BufRead, BufReader, Read};

pub fn parse(data: &[u8]) {
    let mut reader = BufReader::new(data);
    let mut instruction = [0; 1];
    reader.read_exact(&mut instruction).unwrap();
    let instruction = instruction[0];

    if instruction == 0 {
        panic!("unknow instrution {:08b}", instruction);
    }

    match instruction & 0x80 != 0 {
        // copy
        true => {
            let (offset, size) = get_offset_size(&mut reader, instruction);
            println!("copy\noffset {offset} size {size}");
        }
        // insert
        false => {
            let size = get_size(&mut reader, instruction);
            println!("insert\n{size} {size:08b}");
        }
    }
}

fn get_offset_size(data: &mut impl BufRead, instruction: u8) -> (u32, u32) {
    let offset_iter = instruction & 0x0f;
    let mut offset: u32 = 0;
    let mut buf: [u8; 1] = [0; 1];

    for i in 0..4 {
        if offset_iter >> i & 1 == 1 {
            data.read_exact(&mut buf).unwrap();
            let shifted = (buf[0] as u32) << i * 8;
            offset |= shifted;
        }
    }

    let size_iter = (instruction & 0x70) >> 4;
    let mut size: u32 = 0;
    for i in 0..3 {
        if size_iter >> i & 1 == 1 {
            data.read_exact(&mut buf).unwrap();
            let shifted = (buf[0] as u32) << i * 8;
            size |= shifted;
        }
    }

    if size == 0 {
        size = 0x10000;
    }

    (offset, size)
}

fn get_size(_: &mut impl BufRead, instruction: u8) -> u8 {
    instruction & 0x7f
}
