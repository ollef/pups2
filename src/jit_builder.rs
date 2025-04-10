use std::{
    collections::{btree_map::Entry, BTreeMap},
    rc::Rc,
};

use cranelift_codegen::{
    control::ControlPlane,
    ir::{types, AbiParam, InstBuilder, Signature, UserFuncName},
    isa::OwnedTargetIsa,
    settings, Context,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use memmap2::{Mmap, MmapOptions, RemapOptions};

pub struct JitBuilder {
    isa: OwnedTargetIsa,
    context: Context,
    // declarations: ModuleDeclarations,
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
        let mut exe_allocator = ExecutableMemoryAllocator::new();
        let mem = exe_allocator.allocate(compiled_code.code_buffer());

        let function = unsafe { std::mem::transmute::<*const u8, extern "C" fn(i32) -> i32>(mem) };
        let result = function(10);
        exe_allocator.deallocate(mem);
        println!("Result of function call: {}", result);

        jit_builder
    }
}

impl JitBuilder {
    pub fn make_signature(&self) -> Signature {
        Signature::new(self.isa.default_call_conv())
    }
}

const MINIMUM_MMAP_SIZE: usize = 4096;

#[derive(Debug, Clone)]
struct Block {
    mmap: u32,
    address: *const u8,
    len: u32,
}

impl Block {
    pub fn end(&self) -> *const u8 {
        unsafe { self.address.add(self.len as usize) }
    }
}

struct ExecutableMemoryAllocator {
    mmaps: Vec<Mmap>,
    free_by_len: BTreeMap<u32, Vec<Block>>,
    free_by_address: BTreeMap<*const u8, Block>,
    used_by_address: BTreeMap<*const u8, Block>,
}

impl ExecutableMemoryAllocator {
    fn new() -> Self {
        ExecutableMemoryAllocator {
            mmaps: Vec::new(),
            free_by_len: BTreeMap::new(),
            free_by_address: BTreeMap::new(),
            used_by_address: BTreeMap::new(),
        }
    }

    fn insert_free_block(&mut self, mut block: Block) {
        if block.len == 0 {
            return;
        }
        if let Some((_, prev_block)) = self.free_by_address.range_mut(..block.address).next_back() {
            if prev_block.end() == block.address && prev_block.mmap == block.mmap {
                match self.free_by_len.entry(prev_block.len) {
                    Entry::Vacant(_) => panic!("Block not found"),
                    Entry::Occupied(mut occupied_entry) => {
                        let blocks = occupied_entry.get_mut();
                        blocks.retain(|b| b.address != prev_block.address);
                        if blocks.is_empty() {
                            occupied_entry.remove();
                        }
                    }
                }
                prev_block.len += block.len;
                self.free_by_len
                    .entry(prev_block.len)
                    .or_default()
                    .push(prev_block.clone());
                return;
            }
        }
        if let Some((_, next_block)) = self.free_by_address.range_mut(block.address..).next() {
            if block.end() == next_block.address && next_block.mmap == block.mmap {
                block.len += next_block.len;
                let next_block_address = next_block.address;
                let next_block_len = next_block.len;
                self.free_by_address.remove(&next_block_address);
                match self.free_by_len.entry(next_block_len) {
                    Entry::Vacant(_) => panic!("Block not found"),
                    Entry::Occupied(mut occupied_entry) => {
                        let blocks = occupied_entry.get_mut();
                        blocks.retain(|b| b.address != next_block_address);
                        if blocks.is_empty() {
                            occupied_entry.remove();
                        }
                    }
                }
            }
        }
        self.free_by_len
            .entry(block.len)
            .or_default()
            .push(block.clone());
        self.free_by_address.insert(block.address, block);
    }

    pub fn allocate(&mut self, data: &[u8]) -> *const u8 {
        if let Some((_, blocks)) = self.free_by_len.range_mut(data.len() as u32..).next() {
            let block = blocks.pop().unwrap();
            if blocks.is_empty() {
                self.free_by_len.remove(&block.len);
            }
            let allocated_block = Block {
                mmap: block.mmap,
                address: block.address,
                len: data.len() as u32,
            };
            let address = allocated_block.address;
            let mmap = self
                .mmaps
                .swap_remove(block.mmap as usize)
                .make_mut()
                .unwrap();
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), address as *mut u8, data.len());
            }
            let mmap = mmap.make_exec().unwrap();
            self.mmaps.push(mmap);
            let last_index = self.mmaps.len() - 1;
            self.mmaps.swap(block.mmap as usize, last_index);

            let free_block = Block {
                mmap: block.mmap,
                address: allocated_block.end(),
                len: block.len - data.len() as u32,
            };
            self.insert_free_block(free_block);
            self.free_by_address
                .remove(&allocated_block.address)
                .unwrap();
            self.used_by_address
                .insert(allocated_block.address, allocated_block);
            address
        } else {
            let len = data.len().max(MINIMUM_MMAP_SIZE);
            let mut mmap = MmapOptions::new().len(len).map_anon().unwrap();
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), mmap.as_mut_ptr(), data.len());
            }
            let address = mmap.as_mut_ptr();
            let mmap = mmap.make_exec().unwrap();
            let mmap_index = self.mmaps.len() as u32;
            self.mmaps.push(mmap);
            let allocated_block = Block {
                mmap: mmap_index,
                address,
                len: data.len() as u32,
            };
            let free_block = Block {
                mmap: mmap_index,
                address: allocated_block.end(),
                len: len as u32 - data.len() as u32,
            };
            self.used_by_address
                .insert(allocated_block.address, allocated_block);
            self.insert_free_block(free_block);
            address
        }
    }

    fn deallocate(&mut self, ptr: *const u8) {
        let block = self.used_by_address.remove(&ptr).unwrap();
        self.insert_free_block(block)
    }
}
