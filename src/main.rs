mod bits;
mod bytes;
mod emotion_engine;
mod fifo;
mod fix;
use argh::FromArgs;
use bytes::Bytes;
use elf::{endian::LittleEndian, ElfBytes};
use emotion_engine::{
    dmac::Dmac,
    gif::Gif,
    scheduler::{self, Event},
};
use minifb::{Scale, ScaleMode, Window, WindowOptions};

#[derive(FromArgs)]
#[argh(description = "Perpetually Unfinished PS2 emulator")]
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
    core.mmu.mmap(0, 0x2000_0000, 0);
    let mut scheduler = scheduler::Scheduler::new();
    loop {
        match scheduler.next_event() {
            Event::Run(cycles) => {
                for i in 0..cycles {
                    core.step_interpreter(&mut bus);
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
                println!("VBlank end");
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
        execute(&args.file)
    }
}
