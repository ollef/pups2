mod bits;
mod bytes;
mod emotion_engine;
mod enum_set;
mod executable_memory_allocator;
mod fifo;
mod fix;

use argh::FromArgs;
use bytes::Bytes;
use elf::{endian::LittleEndian, ElfBytes};
use emotion_engine::{
    core::instruction_gen::Instruction,
    dmac::Dmac,
    gif::Gif,
    scheduler::{self, Event},
};
use minifb::{Scale, ScaleMode, Window, WindowOptions};
use std::time::Instant;

#[derive(FromArgs)]
#[argh(description = "Perpetually Unfinished PS2 emulator")]
struct Arguments {
    #[argh(switch, short = 'd', description = "disassemble the ELF file")]
    disassemble: bool,
    #[argh(option, short = 'b', description = "BIOS file")]
    bios: Option<String>,
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
            let instruction = Instruction::decode(data);
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

fn execute(bios: &Option<String>, file: &str) -> std::io::Result<()> {
    let mut core = emotion_engine::core::Core::new();
    let mut bus = emotion_engine::bus::Bus::new();
    if let Some(bios) = bios {
        let bios_data = std::fs::read(bios)?;
        bus.boot_memory[0..bios_data.len()].copy_from_slice(&bios_data);
    } else {
        let elf_data = std::fs::read(file)?;
        let elf = ElfBytes::<LittleEndian>::minimal_parse(&elf_data).expect("Failed to parse ELF");
        let entry_point = elf.ehdr.e_entry as u32;
        core.state.program_counter = entry_point;
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
    }
    let mut window = Window::new(
        "pups2",
        640,
        480,
        WindowOptions {
            borderless: false,
            title: true,
            resize: true,
            scale: Scale::X2,
            scale_mode: ScaleMode::Center,
            topmost: false,
            transparency: false,
            none: false,
        },
    )
    .expect("Failed to create window");
    window.set_background_color(20, 20, 20);
    let mut scheduler = scheduler::Scheduler::new();
    let mut frame_start = Instant::now();
    loop {
        match scheduler.next_event() {
            Event::Run(cycles) => {
                core.step(cycles, &mut bus);
                for i in 0..cycles {
                    if (scheduler.cycle + i) % 2 == 0 {
                        Dmac::step(&mut bus);
                        Gif::step(&mut bus);
                        bus.gs.step();
                    }
                    bus.timer.step();
                }
                scheduler.tick(cycles);
            }
            Event::VBlankStart => {
                println!("VBlank start");
            }
            Event::GsVBlank => {
                bus.gs.vblank();
                println!("GS VBlank");
            }
            Event::VBlankEnd => {
                let frame_duration = frame_start.elapsed();
                frame_start = Instant::now();
                println!("VBlank end");
                println!(
                    "Frame duration: {} ms",
                    frame_duration.as_secs_f64() * 1000.0
                );
                if let Some((frame_buffer_width, frame_buffer)) = bus.gs.frame_buffer() {
                    let frame_buffer = unsafe {
                        std::slice::from_raw_parts(
                            frame_buffer.as_ptr() as *const u32,
                            frame_buffer.len() / 4,
                        )
                    };

                    window
                        .update_with_buffer(
                            frame_buffer,
                            frame_buffer_width as usize,
                            frame_buffer.len() / frame_buffer_width as usize,
                        )
                        .expect("Failed to update window");
                } else {
                    window.update();
                }
                if window.is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No) {
                    break;
                }
                if !window.is_open() {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args: Arguments = argh::from_env();
    if args.disassemble {
        disassemble(&args.file)
    } else {
        execute(&args.bios, &args.file)
    }
}
