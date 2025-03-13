mod disassembler;
use elf::{endian::LittleEndian, ElfBytes};

struct Memory {
    pub rd: Vec<u8>,
}

impl Memory {
    fn new() -> Self {
        Self {
            rd: vec![0; 1024 * 1024 * 32],
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let elf_data = std::fs::read("demos/demo2a.elf")?;
    let elf = ElfBytes::<LittleEndian>::minimal_parse(&elf_data).expect("Failed to parse ELF");
    let entry_point = elf.ehdr.e_entry as u32;
    println!("Entry point: {:x?}", entry_point);
    println!("Program header start: {:x?}", entry_point as u32);
    let mut memory = Memory::new();
    for program_header in elf.segments().expect("Failed to get program headers") {
        let physical_memory_address = program_header.p_paddr;
        println!("Physical memory address: {:x?}", physical_memory_address);
        let data = elf
            .segment_data(&program_header)
            .expect("Failed to get segment data");
        memory.rd[physical_memory_address as usize..physical_memory_address as usize + data.len()]
            .copy_from_slice(data);
    }
    let mut pc = entry_point;
    loop {
        let instruction_data = u32::from_le_bytes([
            memory.rd[pc as usize],
            memory.rd[pc as usize + 1],
            memory.rd[pc as usize + 2],
            memory.rd[pc as usize + 3],
        ]);
        let instruction = disassembler::disassemble(instruction_data);
        print!("{:x?}: {}", pc, instruction);
        if instruction.is_nop() {
            println!(" (nop)");
        } else {
            println!();
        }
        pc += 4;
    }
    Ok(())
}
