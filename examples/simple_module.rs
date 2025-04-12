use std::path::Path;

use inkwell::{
    AddressSpace, OptimizationLevel,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};

fn main() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("mymodule");
    {
        let time_function = module.add_function(
            "time",
            context
                .i32_type()
                .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
            None,
        );
        let srand_function = module.add_function(
            "srand",
            context
                .void_type()
                .fn_type(&[context.i32_type().into()], false),
            None,
        );
        let rand_function =
            module.add_function("rand", context.i32_type().fn_type(&[], false), None);
        let printf_function = module.add_function(
            "printf",
            context
                .void_type()
                .fn_type(&[context.ptr_type(AddressSpace::default()).into()], true),
            None,
        );
        let sum = module.add_function("main", context.i32_type().fn_type(&[], false), None);
        let block = context.append_basic_block(sum, "entry");
        builder.position_at_end(block);
        let printf_function_format_arg =
            builder.build_global_string_ptr("Hello, %d!\n", "").unwrap();
        let time = builder
            .build_direct_call(
                time_function,
                &[context
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left();
        builder
            .build_direct_call(srand_function, &[time.into()], "")
            .unwrap();
        let random = builder
            .build_direct_call(rand_function, &[], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left();
        builder
            .build_direct_call(
                printf_function,
                &[
                    printf_function_format_arg.as_pointer_value().into(),
                    random.into(),
                ],
                "",
            )
            .unwrap();
        builder
            .build_return(Some(&context.i32_type().const_zero()))
            .unwrap();
    }
    Target::initialize_all(&InitializationConfig::default());
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap();
    target_machine
        .write_to_file(&module, FileType::Object, Path::new("sum.o"))
        .unwrap();
    module.print_to_file(Path::new("sum.ll")).unwrap();
}
