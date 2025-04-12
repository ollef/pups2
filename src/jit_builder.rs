use cranelift_codegen::{
    control::ControlPlane,
    ir::{types, AbiParam, InstBuilder, Signature},
    isa::OwnedTargetIsa,
    settings, Context,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

use crate::executable_memory_allocator::ExecutableMemoryAllocator;

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
