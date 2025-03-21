mod bits;
mod bytes;
mod emotion_engine;
mod fifo;
use argh::FromArgs;
use bytes::Bytes;
use elf::{endian::LittleEndian, ElfBytes};
use emotion_engine::{dmac::Dmac, gif::Gif};

#[derive(FromArgs)]
#[argh(description = "Emotion Engine PS2 emulator")]
struct Arguments {
    #[argh(switch, short = 'd', description = "disassemble the ELF file")]
    disassemble: bool,
    #[argh(positional, description = "ELF file")]
    file: String,
}

fn disassemble(file: &str) -> std::io::Result<()> {
    let elf_data = std::fs::read(file)?;
    let elf = ElfBytes::<LittleEndian>::minimal_parse(&elf_data).expect("Failed to parse ELF");
    let entry_point = elf.ehdr.e_entry as u32;
    println!("Entry point: {:x?}", entry_point);
    for program_header in elf.segments().expect("Failed to get program headers") {
        let virtual_address = program_header.p_vaddr;
        let data = elf
            .segment_data(&program_header)
            .expect("Failed to get segment data");

        for (word_index, bytes) in data.chunks_exact(4).enumerate() {
            let address = virtual_address + (word_index as u64 * 4);
            let data = u32::from_bytes(bytes);
            let instruction = emotion_engine::core::disassembler::disassemble(data);
            print!(
                "{:6x?}:    {:02x?} {:02x?} {:02x?} {:02x?}    {}",
                address, bytes[3], bytes[2], bytes[1], bytes[0], instruction
            );
            if instruction.is_nop() {
                println!(" # NOP");
            } else if let Some(branch_target) = instruction.branch_target(address as u32) {
                println!(" # 0x{:x?}", branch_target);
            } else {
                println!();
            }
        }
    }
    Ok(())
}

fn execute(file: &str) -> std::io::Result<()> {
    let elf_data = std::fs::read(file)?;
    let elf = ElfBytes::<LittleEndian>::minimal_parse(&elf_data).expect("Failed to parse ELF");
    let entry_point = elf.ehdr.e_entry as u32;
    let mut core = emotion_engine::core::Core::new(entry_point);
    let mut bus = emotion_engine::bus::Bus::new();
    println!("Entry point: {:x?}", entry_point);
    println!("Program header start: {:x?}", entry_point as u32);
    for program_header in elf.segments().expect("Failed to get program headers") {
        let physical_address = program_header.p_paddr;
        let virtual_address = program_header.p_vaddr;
        println!("Physical memory address: {:x?}", physical_address);
        println!("Virtual memory address: {:x?}", virtual_address);
        let data = elf
            .segment_data(&program_header)
            .expect("Failed to get segment data");
        // state.tlb.mmap(
        //     virtual_address as u32,
        //     data.len() as u32,
        //     physical_address as u32,
        // );
        bus.main_memory[physical_address as usize..physical_address as usize + data.len()]
            .copy_from_slice(data);
    }
    core.mmu.mmap(0, 0x2000_0000, 0);
    loop {
        core.step_interpreter(&mut bus);
        Dmac::step(&mut bus);
        Gif::step(&mut bus);
        bus.timer.step();
    }
    // for program_header in elf.segments().expect("Failed to get program headers") {
    //     println!("Disassembling segment at {:x?}", program_header.p_paddr);
    //     for pc in
    //         (program_header.p_paddr..program_header.p_paddr + program_header.p_filesz).step_by(4)
    //     {
    //         let instruction_data = u32::from_le_bytes([
    //             memory.rd[pc as usize],
    //             memory.rd[pc as usize + 1],
    //             memory.rd[pc as usize + 2],
    //             memory.rd[pc as usize + 3],
    //         ]);
    //         let instruction = emotion_engine::disassembler::disassemble(instruction_data);
    //         print!("{:x?}: {}", pc, instruction);
    //         if instruction.is_nop() {
    //             println!(" (nop)");
    //         } else {
    //             println!();
    //         }
    //     }
    // }
    // Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args: Arguments = argh::from_env();
    if args.disassemble {
        disassemble(&args.file)
    } else {
        execute(&args.file)
    }
}
