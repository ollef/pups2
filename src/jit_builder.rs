use std::collections::{btree_map::Entry, BTreeMap};

use cranelift_codegen::{
    control::ControlPlane,
    ir::{types, AbiParam, InstBuilder, Signature},
    isa::OwnedTargetIsa,
    settings, Context,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use mmap_rs::{MmapMut, MmapOptions, UnsafeMmapFlags};

pub struct JitBuilder {
    isa: OwnedTargetIsa,
    context: Context,
}

impl JitBuilder {
    pub fn new() -> Self {
        let mut jit_builder = JitBuilder {
            isa: cranelift_native::builder()
                .unwrap()
                .finish(settings::Flags::new(settings::builder()))
                .unwrap(),
            context: Context::new(),
        };

        let mut function_context = FunctionBuilderContext::new();
        let mut signature = jit_builder.make_signature();
        signature.params.push(AbiParam::new(types::I32));
        signature.returns.push(AbiParam::new(types::I32));

        jit_builder.context.func.signature = signature;

        let mut function_builder =
            FunctionBuilder::new(&mut jit_builder.context.func, &mut function_context);
        let block = function_builder.create_block();
        function_builder.switch_to_block(block);
        function_builder.append_block_params_for_function_params(block);
        let param = function_builder.block_params(block)[0];
        let constant = function_builder.ins().iconst(types::I32, 123);
        let result = function_builder.ins().iadd(constant, param);
        function_builder.ins().return_(&[result]);
        function_builder.seal_all_blocks();
        function_builder.finalize();
        let result = jit_builder
            .context
            .compile(jit_builder.isa.as_ref(), &mut ControlPlane::default())
            .unwrap();
        println!("Result : {:?}", result);
        let compiled_code = jit_builder.context.compiled_code().unwrap();
        println!("Compiled code: {:?}", compiled_code);
        let mut exe_allocator = ExecutableMemoryAllocator::default();
        let mem = exe_allocator.allocate(compiled_code.code_buffer());
        let mut mems = Vec::new();
        for _ in 0..100000 {
            let mem = exe_allocator.allocate(compiled_code.code_buffer());
            mems.push(mem);
        }
        for i in 200..300 {
            exe_allocator.free(mems[i]);
        }
        for i in 0..200 {
            exe_allocator.free(mems[i]);
        }
        for i in 300..100000 {
            exe_allocator.free(mems[i]);
        }

        let function = unsafe { std::mem::transmute::<*const u8, extern "C" fn(i32) -> i32>(mem) };
        let result = function(10);
        exe_allocator.free(mem);
        println!("Exe allocator: {:?}", exe_allocator);
        println!("Result of function call: {}", result);

        jit_builder
    }
}

impl JitBuilder {
    pub fn make_signature(&self) -> Signature {
        Signature::new(self.isa.default_call_conv())
    }
}

#[derive(Debug, Clone)]
struct Block {
    address: *const u8,
    len: u32,
}

impl Block {
    pub fn end(&self) -> *const u8 {
        unsafe { self.address.add(self.len as usize) }
    }
}

extern "C" {
    /// This function is provided by LLVM to clear the instruction cache for the specified range.
    fn __clear_cache(start: *mut core::ffi::c_void, end: *mut core::ffi::c_void);
}

#[derive(Debug, Default)]
struct ExecutableMemoryAllocator {
    mmaps: Vec<MmapMut>,
    free_by_len: BTreeMap<u32, Vec<*const u8>>,
    free_by_address: BTreeMap<*const u8, u32>,
}

impl ExecutableMemoryAllocator {
    fn insert_free_block(&mut self, mut block: Block) {
        if block.len == 0 {
            return;
        }
        if let Some((next_block_address, next_block_len)) =
            self.free_by_address.remove_entry(&block.end())
        {
            block.len += next_block_len;
            match self.free_by_len.entry(next_block_len) {
                Entry::Vacant(_) => panic!("Block not found"),
                Entry::Occupied(mut occupied_entry) => {
                    let blocks = occupied_entry.get_mut();
                    blocks.retain(|address| *address != next_block_address);
                    if blocks.is_empty() {
                        occupied_entry.remove();
                    }
                }
            }
        }
        if let Some((prev_block_address, prev_block_len)) =
            self.free_by_address.range_mut(..block.address).next_back()
        {
            if unsafe { prev_block_address.add(*prev_block_len as usize) } == block.address {
                match self.free_by_len.entry(*prev_block_len) {
                    Entry::Vacant(_) => panic!("Block not found"),
                    Entry::Occupied(mut occupied_entry) => {
                        let blocks = occupied_entry.get_mut();
                        blocks.retain(|address| *address != *prev_block_address);
                        if blocks.is_empty() {
                            occupied_entry.remove();
                        }
                    }
                }
                *prev_block_len += block.len;
                self.free_by_len
                    .entry(*prev_block_len)
                    .or_default()
                    .push(*prev_block_address);
                return;
            }
        }
        self.free_by_len
            .entry(block.len)
            .or_default()
            .push(block.address);
        self.free_by_address.insert(block.address, block.len);
    }

    fn try_allocate(&mut self, data: &[u8]) -> Option<*const u8> {
        let len_including_size = data.len() as u32 + std::mem::size_of::<u32>() as u32;
        if let Some((block_len, blocks)) = self.free_by_len.range_mut(len_including_size..).next() {
            let block_len = *block_len;
            let block_address = blocks.pop().unwrap();
            if blocks.is_empty() {
                self.free_by_len.remove(&block_len);
            }
            self.free_by_address.remove(&block_address).unwrap();
            let allocated_block = Block {
                address: block_address,
                len: len_including_size,
            };
            let after_len = unsafe { block_address.add(std::mem::size_of::<u32>()) };
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &len_including_size as *const u32,
                    block_address as *mut u32,
                    1,
                );
                std::ptr::copy_nonoverlapping(data.as_ptr(), after_len as *mut u8, data.len());
            }
            unsafe {
                __clear_cache(
                    after_len as *mut std::ffi::c_void,
                    after_len.add(data.len()) as *mut std::ffi::c_void,
                )
            };
            let free_block = Block {
                address: allocated_block.end(),
                len: block_len - allocated_block.len,
            };
            self.insert_free_block(free_block);
            Some(after_len)
        } else {
            None
        }
    }

    pub fn allocate(&mut self, data: &[u8]) -> *const u8 {
        if let Some(address) = self.try_allocate(data) {
            return address;
        }
        let len_including_size = data.len() + std::mem::size_of::<u32>();
        let mmap_len = len_including_size
            .max(
                self.mmaps
                    .last()
                    .map(|mmap| mmap.len() * 2)
                    .unwrap_or_default(),
            )
            .div_ceil(MmapOptions::page_size())
            * MmapOptions::page_size();
        let mmap = unsafe {
            MmapOptions::new(mmap_len)
                .unwrap()
                .with_unsafe_flags(UnsafeMmapFlags::JIT)
                .map_exec_mut()
                .unwrap()
        };
        self.insert_free_block(Block {
            address: mmap.as_ptr(),
            len: mmap.len() as u32,
        });
        self.mmaps.push(mmap);
        self.try_allocate(data).unwrap()
    }

    pub fn free(&mut self, address: *const u8) {
        let block_address = unsafe { address.sub(std::mem::size_of::<u32>()) };
        let len = unsafe { *(block_address as *const u32) };
        self.insert_free_block(Block {
            address: block_address,
            len,
        });
    }
}
