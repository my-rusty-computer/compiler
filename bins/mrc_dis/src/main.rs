use clap::{App, Arg};
use mrc_decoder::decode_instruction;
use mrc_instruction::Instruction;
use std::fmt::{Display, Formatter};
use std::io::{ErrorKind, Read};

struct SegmentAndOffset(u16, u16);

trait ToSegmentAndOffset {
    fn relative_to(&self, segment_and_offset: &SegmentAndOffset) -> SegmentAndOffset;
}

impl ToSegmentAndOffset for u32 {
    fn relative_to(&self, segment_and_offset: &SegmentAndOffset) -> SegmentAndOffset {
        let offset = (self & !((segment_and_offset.0 as u32) << 4)) + segment_and_offset.1 as u32;
        SegmentAndOffset(segment_and_offset.0, offset as u16)
    }
}

impl Display for SegmentAndOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04X}:{:04X}", self.0, self.1)
    }
}

struct Data {
    data: Vec<u8>,
    current_position: u32,
}

impl Data {
    fn new() -> Self {
        Self {
            data: vec![],
            current_position: 0,
        }
    }
}

impl Iterator for Data {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.data.get(self.current_position as usize) {
            Some(byte) => {
                self.current_position += 1;
                Some(*byte)
            }
            None => None,
        }
    }
}

struct Section<'a> {
    addr: SegmentAndOffset,
    data: &'a [u8],
}

impl<'a> Section<'a> {
    fn new(addr: SegmentAndOffset, data: &'a [u8]) -> Self {
        Self { addr, data }
    }
}

struct SectionIterator<'a> {
    section: &'a Section<'a>,
    position: u32,
}

impl<'a> Iterator for SectionIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.section.data.get(self.position as usize) {
            Some(byte) => {
                self.position += 1;
                Some(*byte)
            }
            None => None,
        }
    }
}

fn print_instruction(addr: SegmentAndOffset, bytes: &[u8], instruction: &Instruction) {
    let bytes_to_print = 5;
    let mut b: String = bytes
        .iter()
        .take(bytes_to_print)
        .map(|b| format!("{:02X} ", b))
        .collect();

    for _ in bytes.len()..5 {
        b.push_str("   ");
    }

    println!("{}  {}  {}", addr, b, instruction);
}

fn print_data_byte(addr: SegmentAndOffset, byte: u8) {
    println!("{}  {:02X}               db {:02X}", addr, byte, byte);
}

fn print_section(section: &Section) {
    let mut it = SectionIterator {
        section,
        position: 0,
    };

    while (it.position as usize) < section.data.len() {
        let start = it.position;
        match decode_instruction(&mut it) {
            Ok(instruction) => {
                let bytes_used = it.position - start;
                let bytes = &section.data[(start as usize)..(start + bytes_used) as usize];
                print_instruction(start.relative_to(&section.addr), bytes, &instruction);
            }
            Err(_) => {
                print_data_byte(
                    start.relative_to(&section.addr),
                    section.data[start as usize],
                );
            }
        }
    }
}

fn main() {
    let matches = App::new("mrc-dis")
        .version("0.1")
        .arg(
            Arg::with_name("binary")
                .value_name("binary")
                .help("The binary file to disassemble.")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mut data = Data::new();

    if let Some(path) = matches.value_of("binary") {
        match std::fs::File::open(path) {
            Ok(ref mut file) => {
                if let Err(err) = file.read_to_end(&mut data.data) {
                    eprintln!("Could not read file. ({}) ({})", path, err);
                    return;
                }
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => eprintln!("File not found. ({})", path),
                    _ => eprintln!("Could not open file. ({}) ({})", path, err),
                };
                return;
            }
        }
    }

    if data.data.is_empty() {
        println!("Could not read binary file.");
        return;
    }

    // Create a section of the whole file.
    let section = Section::new(SegmentAndOffset(0x1000, 0x0100), &data.data[..]);

    print_section(&section);
}
