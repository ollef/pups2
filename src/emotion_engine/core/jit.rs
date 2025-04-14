use std::{collections::BTreeMap, ops::Range};

use bitvec::vec::BitVec;
use cranelift_codegen::{
    control::ControlPlane,
    ir::{InstBuilder, Signature},
    isa::OwnedTargetIsa,
    settings,
};
use enum_map::EnumMap;

use crate::{emotion_engine::bus::Bus, executable_memory_allocator::ExecutableMemoryAllocator};

use super::{decoder::decode, instruction::Instruction, mmu::Mmu, register::Register, Mode, State};

pub struct Jit {
    jitted_instructions: BitVec<usize>,
    jitted_starts_map: BTreeMap<u32, u16>,
    jitted_starts: Box<[CacheIndex]>,
    cache: Vec<CacheEntry>,
    next_to_remove: u16,
    isa: OwnedTargetIsa,
    codegen_context: cranelift_codegen::Context,
    function_builder_context: cranelift_frontend::FunctionBuilderContext,
    executable_memory: ExecutableMemoryAllocator,
}

#[derive(Clone, Copy)]
struct CacheIndex(u16);

const CODE_CACHE_MAX_SIZE: u16 = u16::MAX - 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CacheIndexView {
    NotCached,
    Cached(u16),
}

impl CacheIndex {
    pub fn not_cached() -> Self {
        CacheIndex(0)
    }

    pub fn cached(index: u16) -> Self {
        CacheIndex(index + 1)
    }

    pub fn view(self) -> CacheIndexView {
        match self.0 {
            0 => CacheIndexView::NotCached,
            _ => CacheIndexView::Cached(self.0 - 1),
        }
    }
}

pub struct CacheEntry {
    pub address_range: Range<u32>,
    pub code: Code,
}

pub enum Code {
    Jitted(extern "C" fn()),
    Interpreted(Instruction),
}

const VIRTUAL_MEMORY_SIZE: usize = 0x1_0000_0000;
const INSTRUCTION_SIZE: usize = std::mem::size_of::<u32>();

impl Jit {
    pub fn new() -> Self {
        Jit {
            jitted_instructions: BitVec::from_vec(vec![
                0;
                VIRTUAL_MEMORY_SIZE
                    / INSTRUCTION_SIZE
                    / (std::mem::size_of::<usize>() * 8)
            ]),
            jitted_starts_map: BTreeMap::new(),
            jitted_starts: Box::new(
                [CacheIndex::not_cached(); VIRTUAL_MEMORY_SIZE / INSTRUCTION_SIZE],
            ),
            cache: Vec::new(),
            next_to_remove: 0,
            isa: cranelift_native::builder()
                .unwrap()
                .finish(settings::Flags::new(settings::builder()))
                .unwrap(),
            codegen_context: cranelift_codegen::Context::new(),
            function_builder_context: cranelift_frontend::FunctionBuilderContext::new(),
            executable_memory: ExecutableMemoryAllocator::default(),
        }
    }

    fn remove(&mut self, cache_index: u16) {
        let entry = self.cache.swap_remove(cache_index as usize);
        if let Some(moved_code) = self.cache.last() {
            let moved_index = (self.cache.len() - 1) as u16;
            self.jitted_starts_map
                .insert(moved_code.address_range.start, moved_index)
                .unwrap();
            self.jitted_starts[moved_code.address_range.start as usize / INSTRUCTION_SIZE] =
                CacheIndex::cached(moved_index);
        }
        self.jitted_starts[entry.address_range.start as usize / INSTRUCTION_SIZE] =
            CacheIndex::not_cached();
        self.jitted_starts_map
            .remove(&entry.address_range.start)
            .unwrap();
        let start = self
            .jitted_starts_map
            .range(..entry.address_range.start)
            .rev()
            .map(|(_, index)| {
                let earlier_code = &self.cache[*index as usize];
                earlier_code.address_range.end
            })
            .take_while(|earlier_code_end| *earlier_code_end > entry.address_range.start)
            .max()
            .unwrap_or(entry.address_range.start);
        let end = self
            .jitted_starts_map
            .range(entry.address_range.start..entry.address_range.end)
            .next()
            .map(|(start, _)| *start)
            .unwrap_or(entry.address_range.end);
        if start < end {
            self.jitted_instructions
                [start as usize / INSTRUCTION_SIZE..end as usize / INSTRUCTION_SIZE]
                .fill(false);
        }
        match entry.code {
            Code::Jitted(function) => self.executable_memory.free(function as *const u8),
            Code::Interpreted(_) => {}
        }
    }

    fn add(&mut self, code: CacheEntry) -> u16 {
        if self.cache.len() as u16 == CODE_CACHE_MAX_SIZE {
            self.remove(self.next_to_remove);
            self.next_to_remove += 1;
            if self.next_to_remove >= CODE_CACHE_MAX_SIZE {
                self.next_to_remove = 0;
            }
        }
        let cache_index = self.cache.len() as u16;
        self.jitted_starts[code.address_range.start as usize / INSTRUCTION_SIZE] =
            CacheIndex::cached(cache_index);
        self.jitted_starts_map
            .insert(code.address_range.start, cache_index);
        self.jitted_instructions[code.address_range.start as usize / INSTRUCTION_SIZE
            ..code.address_range.end as usize / INSTRUCTION_SIZE]
            .fill(true);
        self.cache.push(code);
        cache_index
    }

    #[inline(always)]
    pub fn invalidate_range(&mut self, range: Range<u32>) {
        if self.jitted_instructions[range.start as usize / INSTRUCTION_SIZE
            ..(range.end as usize).div_ceil(INSTRUCTION_SIZE)]
            .not_any()
        {
            return;
        }

        self.invalidate_range_slow(range);
    }

    fn invalidate_range_slow(&mut self, range: Range<u32>) {
        let to_remove = self
            .jitted_starts_map
            .range(..range.end)
            .rev()
            .take_while(|(_, index)| {
                let entry = &self.cache[**index as usize];
                entry.address_range.end > range.start
            })
            .map(|(_, index)| *index)
            .collect::<Vec<_>>();

        for index in to_remove {
            self.remove(index);
        }
    }

    pub fn cache_entry(&mut self, state: &State, mmu: &Mmu, bus: &Bus, mode: Mode) -> &CacheEntry {
        let cache_index = self
            .jitted_starts
            .get(state.program_counter as usize / INSTRUCTION_SIZE)
            .unwrap()
            .view();
        let index = match cache_index {
            CacheIndexView::NotCached => {
                let mut jit_compiler = JitCompiler::new(
                    state,
                    &self.isa,
                    &mut self.codegen_context,
                    &mut self.function_builder_context,
                    mmu,
                    bus,
                    mode,
                );

                let entry = if let Some(end_address) = jit_compiler.compile(state.program_counter) {
                    jit_compiler.function_builder.seal_all_blocks();
                    jit_compiler.function_builder.finalize();
                    self.codegen_context
                        .compile(self.isa.as_ref(), &mut ControlPlane::default())
                        .unwrap();
                    let compiled_code = self.codegen_context.compiled_code().unwrap();
                    let pointer = self.executable_memory.allocate(compiled_code.code_buffer());
                    let function =
                        unsafe { std::mem::transmute::<*const u8, extern "C" fn()>(pointer) };
                    self.codegen_context.clear();
                    CacheEntry {
                        address_range: state.program_counter..end_address,
                        code: Code::Jitted(function),
                    }
                } else {
                    let physical_address = mmu.virtual_to_physical(state.program_counter, mode);
                    CacheEntry {
                        address_range: state.program_counter
                            ..state.program_counter + INSTRUCTION_SIZE as u32,
                        code: Code::Interpreted(decode(bus.read(physical_address))),
                    }
                };
                self.add(entry)
            }
            CacheIndexView::Cached(index) => index,
        };
        &self.cache[index as usize]
    }
}

struct JitCompiler<'a> {
    function_builder: cranelift_frontend::FunctionBuilder<'a>,
    state: &'a State,
    mmu: &'a Mmu,
    bus: &'a Bus,
    mode: Mode,
    registers: EnumMap<Register, Option<RegisterState>>,
}

struct RegisterState {
    value: cranelift_codegen::ir::Value,
    size: Size,
    dirty: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Size {
    S8,
    S16,
    S32,
    S64,
    S128,
}

impl Size {
    fn type_(self) -> cranelift_codegen::ir::Type {
        match self {
            Size::S8 => cranelift_codegen::ir::types::I8,
            Size::S16 => cranelift_codegen::ir::types::I16,
            Size::S32 => cranelift_codegen::ir::types::I32,
            Size::S64 => cranelift_codegen::ir::types::I64,
            Size::S128 => cranelift_codegen::ir::types::I128,
        }
    }

    fn bits(self) -> usize {
        match self {
            Size::S8 => 8,
            Size::S16 => 16,
            Size::S32 => 32,
            Size::S64 => 64,
            Size::S128 => 128,
        }
    }
}

impl<'a> JitCompiler<'a> {
    pub fn new(
        state: &'a State,
        isa: &'a OwnedTargetIsa,
        codegen_context: &'a mut cranelift_codegen::Context,
        function_builder_context: &'a mut cranelift_frontend::FunctionBuilderContext,
        mmu: &'a Mmu,
        bus: &'a Bus,
        mode: Mode,
    ) -> Self {
        codegen_context.func.signature = Signature::new(isa.default_call_conv());
        let mut function_builder = cranelift_frontend::FunctionBuilder::new(
            &mut codegen_context.func,
            function_builder_context,
        );
        let block = function_builder.create_block();
        function_builder.switch_to_block(block);
        JitCompiler {
            state,
            function_builder,
            mmu,
            bus,
            mode,
            registers: EnumMap::default(),
        }
    }

    fn register_address(&mut self, register: Register) -> cranelift_codegen::ir::Value {
        let register_address = &self.state.registers[register] as *const u128;
        self.function_builder
            .ins()
            .iconst(cranelift_codegen::ir::types::I64, register_address as i64)
    }

    fn writeback_register(&mut self, register: Register) {
        if let Some(mut state) = std::mem::take(&mut self.registers[register]) {
            if !state.dirty {
                return;
            }
            let register_address = self.register_address(register);
            self.function_builder.ins().store(
                cranelift_codegen::ir::MemFlags::trusted(),
                state.value,
                register_address,
                0,
            );
            state.dirty = false;
            self.registers[register] = Some(state);
        }
    }

    fn get_register(&mut self, register: Register, size: Size) -> cranelift_codegen::ir::Value {
        if register == Register::Zero {
            return self.function_builder.ins().iconst(size.type_(), 0);
        }
        if let Some(state) = &self.registers[register] {
            if state.size == size {
                return state.value;
            }
            if state.size > size {
                return self
                    .function_builder
                    .ins()
                    .ireduce(size.type_(), state.value);
            }
            self.writeback_register(register);
        }
        let register_address = self.register_address(register);
        let value = self.function_builder.ins().load(
            size.type_(),
            cranelift_codegen::ir::MemFlags::trusted(),
            register_address,
            0,
        );
        self.registers[register] = Some(RegisterState {
            value,
            size,
            dirty: false,
        });
        value
    }

    fn set_register(
        &mut self,
        register: Register,
        value: cranelift_codegen::ir::Value,
        size: Size,
    ) {
        if register == Register::Zero {
            return;
        }
        if let Some(state) = &mut self.registers[register] {
            if state.size > size {
                self.writeback_register(register);
            }
        }
        self.registers[register] = Some(RegisterState {
            value,
            size,
            dirty: true,
        });
    }

    fn writeback_program_counter(&mut self, value: cranelift_codegen::ir::Value) {
        let address = &self.state.program_counter as *const u32;
        let address = self
            .function_builder
            .ins()
            .iconst(cranelift_codegen::ir::types::I64, address as i64);
        self.function_builder.ins().store(
            cranelift_codegen::ir::MemFlags::trusted(),
            value,
            address,
            0,
        );
    }

    pub fn compile(&mut self, mut address: u32) -> Option<u32> {
        if self.state.delayed_branch_target.is_some() {
            return None;
        }
        let start_address = address;
        loop {
            let instruction = decode(
                self.bus
                    .read(self.mmu.virtual_to_physical(address, self.mode)),
            );
            if instruction.is_branch() {
                let next_instruction = decode(
                    self.bus.read(
                        self.mmu
                            .virtual_to_physical(address + INSTRUCTION_SIZE as u32, self.mode),
                    ),
                );
                if next_instruction.is_branch() {
                    break;
                }
            }
            let unhandled = || {
                println!("Unhandled instruction at {:#010x}", address);
                println!("{}", instruction);
            };
            match instruction {
                _ if instruction.is_nop() => {}
                Instruction::Unknown => {
                    println!("Unknown instruction at {:#010x}", address)
                }
                Instruction::Sll(rd, rt, shamt) => {
                    let rt_value = self.get_register(rt, Size::S32);
                    let value = self.function_builder.ins().ishl_imm(rt_value, shamt as i64);
                    let value = self
                        .function_builder
                        .ins()
                        .sextend(cranelift_codegen::ir::types::I64, value);
                    self.set_register(rd, value, Size::S64);
                }
                Instruction::Srl(rd, rt, shamt) => {
                    // let value = self.get_register::<u32>(rt) >> shamt;
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Sra(rd, rt, shamt) => {
                    // let value = (self.get_register::<u32>(rt) as i32) >> shamt;
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Sllv(rd, rt, rs) => {
                    // let value =
                    //     self.get_register::<u32>(rt) << self.get_register::<u32>(rs).bits(0..5);
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Srlv(rd, rt, rs) => {
                    // let value =
                    //     self.get_register::<u32>(rt) >> self.get_register::<u32>(rs).bits(0..5);
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Srav(rd, rt, rs) => {
                    // let value = (self.get_register::<u32>(rt) as i32)
                    //     >> self.get_register::<u32>(rs).bits(0..5);
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Jr(rs) => {
                    // self.set_delayed_branch_target(self.get_register::<u32>(rs));
                    unhandled();
                    break;
                }
                Instruction::Jalr(rd, rs) => {
                    // let branch_target = self.get_register::<u32>(rs);
                    // self.set_register(rd, (next_program_counter + 4) as u64);
                    // self.set_delayed_branch_target(branch_target);
                    unhandled();
                    break;
                }
                Instruction::Movz(rd, rs, rt) => {
                    // if self.get_register::<u64>(rt) == 0 {
                    //     let value = self.get_register::<u64>(rs);
                    //     self.set_register(rd, value);
                    // }
                    unhandled();
                    break;
                }
                Instruction::Movn(rd, rs, rt) => {
                    // if self.get_register::<u64>(rt) != 0 {
                    //     let value = self.get_register::<u64>(rs);
                    //     self.set_register(rd, value);
                    // }
                    unhandled();
                    break;
                }
                Instruction::Syscall => {
                    break;
                }
                Instruction::Break => {
                    unhandled();
                    break;
                }
                Instruction::Sync => {
                    // TODO: maybe do something here
                }
                Instruction::Mfhi(rd) => {
                    // self.set_register(rd, self.get_register::<u64>(Register::Hi))
                    unhandled();
                    break;
                }
                Instruction::Mthi(_) => todo!(),
                Instruction::Mflo(_) => todo!(),
                Instruction::Mtlo(_) => todo!(),
                Instruction::Dsllv(_, _, _) => todo!(),
                Instruction::Dsrav(_, _, _) => todo!(),
                Instruction::Dsrlv(_, _, _) => todo!(),
                Instruction::Mult(rd, rs, rt) => {
                    // let a: u64 = self.get_register::<u32>(rs).sign_extend();
                    // let b: u64 = self.get_register::<u32>(rt).sign_extend();
                    // let prod = a.wrapping_mul(b);
                    // let lo: u64 = (prod as u32).sign_extend();
                    // let hi: u64 = ((prod >> 32) as u32).sign_extend();
                    // self.set_register(rd, lo);
                    // self.set_register(Register::Lo, lo);
                    // self.set_register(Register::Hi, hi);
                    unhandled();
                    break;
                }
                Instruction::Multu(rd, rs, rt) => {
                    // let a = self.get_register::<u32>(rs) as u64;
                    // let b = self.get_register::<u32>(rt) as u64;
                    // let prod = a * b;
                    // let lo: u64 = (prod as u32).sign_extend();
                    // let hi: u64 = ((prod >> 32) as u32).sign_extend();
                    // self.set_register(rd, lo);
                    // self.set_register(Register::Lo, lo);
                    // self.set_register(Register::Hi, hi);
                    unhandled();
                    break;
                }
                Instruction::Div(_, _) => todo!(),
                Instruction::Divu(rs, rt) => {
                    // let dividend = self.get_register::<u32>(rs);
                    // let divisor = self.get_register::<u32>(rt);
                    // let (quotient, remainder) = if divisor == 0 {
                    //     (!0, dividend)
                    // } else {
                    //     (dividend / divisor, dividend % divisor)
                    // };
                    // self.set_register::<u64>(Register::Lo, quotient.sign_extend());
                    // self.set_register::<u64>(Register::Hi, remainder.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Add(rd, rs, rt) => {
                    // TODO: Exception on overflow
                    // let value = self
                    //     .get_register::<u32>(rs)
                    //     .wrapping_add(self.get_register::<u32>(rt));
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Addu(rd, rs, rt) => {
                    // let value = self
                    //     .get_register::<u32>(rs)
                    //     .wrapping_add(self.get_register::<u32>(rt));
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Sub(rd, rs, rt) => {
                    // TODO: Exception on overflow
                    // let value = self
                    //     .get_register::<u32>(rs)
                    //     .wrapping_sub(self.get_register::<u32>(rt));
                    // self.set_register::<u64>(rd, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Subu(rd, rs, rt) => {
                    // self.set_register::<u64>(
                    //     rd,
                    //     self.get_register::<u32>(rs)
                    //         .wrapping_sub(self.get_register::<u32>(rt))
                    //         .sign_extend(),
                    // );
                    unhandled();
                    break;
                }
                Instruction::And(rd, rs, rt) => {
                    // self.set_register(
                    //     rd,
                    //     self.get_register::<u64>(rs) & self.get_register::<u64>(rt),
                    // );
                    unhandled();
                    break;
                }
                Instruction::Or(rd, rs, rt) => {
                    // self.set_register(
                    //     rd,
                    //     self.get_register::<u64>(rs) | self.get_register::<u64>(rt),
                    // );
                    unhandled();
                    break;
                }
                Instruction::Xor(_, _, _) => todo!(),
                Instruction::Nor(_, _, _) => todo!(),
                Instruction::Mfsa(_) => todo!(),
                Instruction::Mtsa(_) => todo!(),
                Instruction::Slt(rd, rs, rt) => {
                    // let value = if (self.get_register::<u64>(rs) as i64)
                    //     < (self.get_register::<u64>(rt) as i64)
                    // {
                    //     1
                    // } else {
                    //     0
                    // };
                    // self.set_register::<u64>(rd, value);
                    unhandled();
                    break;
                }
                Instruction::Sltu(rd, rs, rt) => {
                    // let value = if self.get_register::<u64>(rs) < self.get_register::<u64>(rt) {
                    //     1
                    // } else {
                    //     0
                    // };
                    // self.set_register::<u64>(rd, value);
                    unhandled();
                    break;
                }
                Instruction::Dadd(_, _, _) => todo!(),
                Instruction::Daddu(rd, rs, rt) => {
                    // let value = self
                    //     .get_register::<u64>(rs)
                    //     .wrapping_add(self.get_register::<u64>(rt));
                    // self.set_register(rd, value);
                    unhandled();
                    break;
                }
                Instruction::Dsub(_, _, _) => todo!(),
                Instruction::Dsubu(_, _, _) => todo!(),
                Instruction::Tge(_, _) => todo!(),
                Instruction::Tgeu(_, _) => todo!(),
                Instruction::Tlt(_, _) => todo!(),
                Instruction::Tltu(_, _) => todo!(),
                Instruction::Teq(_, _) => todo!(),
                Instruction::Tne(_, _) => todo!(),
                Instruction::Dsll(rd, rt, shamt) => {
                    // self.set_register(rd, self.get_register::<u64>(rt) << shamt);
                    unhandled();
                    break;
                }
                Instruction::Dsrl(rd, rt, shamt) => {
                    // self.set_register(rd, self.get_register::<u64>(rt) >> shamt);
                    unhandled();
                    break;
                }
                Instruction::Dsra(_, _, _) => todo!(),
                Instruction::Dsll32(rd, rt, shamt) => {
                    // self.set_register(rd, self.get_register::<u64>(rt) << (shamt + 32));
                    unhandled();
                    break;
                }
                Instruction::Dsrl32(rd, rt, shamt) => {
                    // self.set_register(rd, self.get_register::<u64>(rt) >> (shamt + 32));
                    unhandled();
                    break;
                }
                Instruction::Dsra32(rd, rt, shamt) => {
                    // self.set_register(
                    //     rd,
                    //     ((self.get_register::<u64>(rt) as i64) >> (shamt + 32)) as u64,
                    // );
                    unhandled();
                    break;
                }
                Instruction::Bltz(rs, offset) => {
                    // if (self.get_register::<u64>(rs) as i64) < 0 {
                    //     let offset: u32 = offset.sign_extend();
                    //     self.set_delayed_branch_target(
                    //         next_program_counter.wrapping_add(offset << 2),
                    //     );
                    // }
                    unhandled();
                    break;
                }
                Instruction::Bgez(rs, offset) => {
                    // if self.get_register::<u64>(rs) as i64 >= 0 {
                    //     let offset: u32 = offset.sign_extend();
                    //     self.set_delayed_branch_target(
                    //         next_program_counter.wrapping_add(offset << 2),
                    //     );
                    // }
                    unhandled();
                    break;
                }
                Instruction::J(target) => {
                    //     self.set_delayed_branch_target(
                    //     (next_program_counter & 0xF000_0000).wrapping_add(target << 2),
                    // )
                    unhandled();
                    break;
                }
                Instruction::Jal(target) => {
                    // self.set_register(Register::Ra, (next_program_counter + 4) as u64);
                    // self.set_delayed_branch_target(
                    //     (next_program_counter & 0xF000_0000).wrapping_add(target << 2),
                    // );
                    unhandled();
                    break;
                }
                Instruction::Beq(rs, rt, offset) => {
                    // if self.get_register::<u64>(rs) == self.get_register::<u64>(rt) {
                    //     let offset: u32 = offset.sign_extend();
                    //     self.set_delayed_branch_target(
                    //         next_program_counter.wrapping_add(offset << 2),
                    //     );
                    // }
                    unhandled();
                    break;
                }
                Instruction::Bne(rs, rt, offset) => {
                    // if self.get_register::<u64>(rs) != self.get_register::<u64>(rt) {
                    //     let offset: u32 = offset.sign_extend();
                    //     self.set_delayed_branch_target(
                    //         next_program_counter.wrapping_add(offset << 2),
                    //     );
                    // }
                    unhandled();
                    break;
                }
                Instruction::Blez(rs, offset) => {
                    // if (self.get_register::<u64>(rs) as i64) <= 0 {
                    //     let offset: u32 = offset.sign_extend();
                    //     self.set_delayed_branch_target(
                    //         next_program_counter.wrapping_add(offset << 2),
                    //     );
                    // }
                    unhandled();
                    break;
                }
                Instruction::Addi(rt, rs, imm) => {
                    // TODO exception on overflow
                    // let temp = self.get_register::<u64>(rs).wrapping_add(imm.sign_extend());
                    // self.set_register::<u64>(rt, (temp as u32).sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Addiu(rt, rs, imm) => {
                    // let temp = self.get_register::<u64>(rs).wrapping_add(imm.sign_extend());
                    // self.set_register::<u64>(rt, (temp as u32).sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Slti(rt, rs, imm) => {
                    // let imm: u64 = imm.sign_extend();
                    // let value = if (self.get_register::<u64>(rs) as i64) < imm as i64 {
                    //     1
                    // } else {
                    //     0
                    // };
                    // self.set_register::<u64>(rt, value);
                    unhandled();
                    break;
                }
                Instruction::Sltiu(rt, rs, imm) => {
                    // let imm: u64 = imm.sign_extend();
                    // let value = if self.get_register::<u64>(rs) < imm {
                    //     1
                    // } else {
                    //     0
                    // };
                    // self.set_register::<u64>(rt, value);
                    unhandled();
                    break;
                }
                Instruction::Andi(rt, rs, imm) => {
                    // self.set_register::<u64>(rt, self.get_register::<u64>(rs) & (imm as u64));
                    unhandled();
                    break;
                }
                Instruction::Ori(rt, rs, imm) => {
                    // self.set_register::<u64>(rt, self.get_register::<u64>(rs) | (imm as u64));
                    unhandled();
                    break;
                }
                Instruction::Xori(rt, rs, imm) => {
                    // self.set_register::<u64>(rt, self.get_register::<u64>(rs) ^ (imm as u64));
                    unhandled();
                    break;
                }
                Instruction::Lui(rt, imm) => {
                    // self.set_register::<u64>(rt, ((imm as u32) << 16).sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Mfc1(rt, fs) => {
                    // let value = self.fpu.get_register::<u32>(fs);
                    // self.set_register::<u64>(rt, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Mtc1(rt, fs) => {
                    // let value = self.get_register::<u32>(rt);
                    // self.fpu.set_register(fs, value);
                    unhandled();
                    break;
                }
                Instruction::Muls(fd, fs, ft) => {
                    //     self.fpu.set_register(
                    //     fd,
                    //     self.fpu.get_register::<f32>(fs) * self.fpu.get_register::<f32>(ft),
                    // )
                    unhandled();
                    break;
                }
                // TODO flags
                Instruction::Divs(fd, fs, ft) => {
                    //     self.fpu.set_register(
                    //     fd,
                    //     self.fpu.get_register::<f32>(fs) / self.fpu.get_register::<f32>(ft),
                    // )
                    unhandled();
                    break;
                }
                // TODO flags
                Instruction::Movs(fd, fs) => {
                    // let value = self.fpu.get_register::<f32>(fs);
                    // self.fpu.set_register(fd, value);
                    unhandled();
                    break;
                }
                Instruction::Cvtws(fd, fs) => {
                    // let value = self.fpu.get_register::<f32>(fs) as i32;
                    // self.fpu.set_register(fd, value as u32);
                    unhandled();
                    break;
                }
                Instruction::Cvtsw(fd, fs) => {
                    // let value = self.fpu.get_register::<u32>(fs) as i32;
                    // self.fpu.set_register(fd, value as f32);
                    unhandled();
                    break;
                }
                Instruction::Ei => {
                    // TODO: Set status register
                }
                Instruction::Beql(rs, rt, offset) => {
                    // if self.get_register::<u64>(rs) == self.get_register::<u64>(rt) {
                    //     let offset: u32 = offset.sign_extend();
                    //     self.set_delayed_branch_target(
                    //         next_program_counter.wrapping_add(offset << 2),
                    //     );
                    // } else {
                    //     next_program_counter += 4;
                    // }
                    unhandled();
                    break;
                }
                Instruction::Bnel(rs, rt, offset) => {
                    // if self.get_register::<u64>(rs) != self.get_register::<u64>(rt) {
                    //     let offset: u32 = offset.sign_extend();
                    //     self.set_delayed_branch_target(
                    //         next_program_counter.wrapping_add(offset << 2),
                    //     );
                    // } else {
                    //     next_program_counter += 4;
                    // }
                    unhandled();
                    break;
                }
                Instruction::Mult1(rd, rs, rt) => {
                    // let a: u64 = self.get_register::<u32>(rs).sign_extend();
                    // let b: u64 = self.get_register::<u32>(rt).sign_extend();
                    // let prod = a.wrapping_mul(b);
                    // let lo: u64 = (prod as u32).sign_extend();
                    // let hi: u64 = ((prod >> 32) as u32).sign_extend();
                    // self.set_register(rd, lo);
                    // self.set_register::<u128>(
                    //     Register::Lo,
                    //     ((lo as u128) << 64) | self.get_register::<u64>(Register::Lo) as u128,
                    // );
                    // self.set_register::<u128>(
                    //     Register::Hi,
                    //     ((hi as u128) << 64) | self.get_register::<u64>(Register::Hi) as u128,
                    // );
                    unhandled();
                    break;
                }
                Instruction::Sq(rt, base, offset) => {
                    // let mut address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // address &= !0b1111;
                    // self.write_virtual(bus, address, self.get_register::<u128>(rt));
                    unhandled();
                    break;
                }
                Instruction::Lb(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // let value = self.read_virtual::<u8>(bus, address);
                    // self.set_register::<u64>(rt, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Lh(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..1) != 0 {
                    //     panic!("Unaligned load at {:#010x}", address);
                    // }
                    // let value = self.read_virtual::<u16>(bus, address);
                    // self.set_register::<u64>(rt, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Lw(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..2) != 0 {
                    //     panic!("Unaligned load at {:#010x}", address);
                    // }
                    // let value = self.read_virtual::<u32>(bus, address);
                    // self.set_register::<u64>(rt, value.sign_extend());
                    unhandled();
                    break;
                }
                Instruction::Lbu(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // let value = self.read_virtual::<u8>(bus, address);
                    // self.set_register(rt, value as u64);
                    unhandled();
                    break;
                }
                Instruction::Lhu(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // let value = self.read_virtual::<u16>(bus, address);
                    // self.set_register(rt, value as u64);
                    unhandled();
                    break;
                }
                Instruction::Lwr(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // let byte = address & 0b11;
                    // let memory_word = self.read_virtual::<u32>(bus, address & !0b11);
                    // let value = if byte == 0 {
                    //     memory_word.sign_extend()
                    // } else {
                    //     let existing = self.get_register::<u64>(rt);
                    //     existing & u64::mask(byte * 8..64) | (memory_word >> (byte * 8)) as u64
                    // };
                    // self.set_register(rt, value);
                    unhandled();
                    break;
                }
                Instruction::Sb(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // self.write_virtual(bus, address, self.get_register::<u8>(rt));
                    unhandled();
                    break;
                }
                Instruction::Sh(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..1) != 0 {
                    //     panic!("Unaligned store at {:#010x}", address);
                    // }
                    // self.write_virtual(bus, address, self.get_register::<u16>(rt));
                    unhandled();
                    break;
                }
                Instruction::Sw(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..2) != 0 {
                    //     panic!("Unaligned store at {:#010x}", address);
                    // }
                    // self.write_virtual(bus, address, self.get_register::<u32>(rt));
                    unhandled();
                    break;
                }
                Instruction::Lwc1(ft, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..2) != 0 {
                    //     panic!("Unaligned load at {:#010x}", address);
                    // }
                    // let value = self.read_virtual::<u32>(bus, address);
                    // self.fpu.set_register(ft, value);
                    unhandled();
                    break;
                }
                Instruction::Ld(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..3) != 0 {
                    //     panic!("Unaligned load at {:#010x}", address);
                    // }
                    // let value = self.read_virtual(bus, address);
                    // self.set_register::<u64>(rt, value);
                    unhandled();
                    break;
                }
                Instruction::Swc1(ft, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..2) != 0 {
                    //     panic!("Unaligned store at {:#010x}", address);
                    // }
                    // self.write_virtual(bus, address, self.fpu.get_register::<u32>(ft));
                    unhandled();
                    break;
                }
                Instruction::Sd(rt, base, offset) => {
                    // let address = self
                    //     .get_register::<u32>(base)
                    //     .wrapping_add(offset.sign_extend());
                    // if address.bits(0..3) != 0 {
                    //     panic!("Unaligned store at {:#010x}", address);
                    // }
                    // self.write_virtual(bus, address, self.get_register::<u64>(rt));
                    unhandled();
                    break;
                }
            }
            address += INSTRUCTION_SIZE as u32;
        }
        if address == start_address {
            return None;
        }
        for register in Register::all() {
            self.writeback_register(register);
        }
        let final_program_counter = self
            .function_builder
            .ins()
            .iconst(cranelift_codegen::ir::types::I32, address as i64);
        self.writeback_program_counter(final_program_counter);
        self.function_builder.ins().return_(&[]);

        Some(address)
    }
}
