use melior::{
    dialect::llvm::{self, r#type::pointer, LoadStoreOptions},
    ir::{attribute::DenseI32ArrayAttribute, r#type::IntegerType, Block, Location, Value},
    Context as MeliorContext,
};

use crate::{constants::STACK_PTR_GLOBAL, errors::CodegenError};

pub fn stack_pop<'ctx>(
    context: &'ctx MeliorContext,
    block: &'ctx Block,
) -> Result<Value<'ctx, 'ctx>, CodegenError> {
    let uint256 = IntegerType::new(context, 256);
    let location = Location::unknown(context);
    let ptr_type = pointer(context, 0);

    let stack_ptr_ptr = block
        .append_operation(llvm_mlir::addressof(
            context,
            STACK_PTR_GLOBAL,
            ptr_type,
            location,
        ))
        .result(0)?;

    let stack_ptr = block
        .append_operation(llvm::load(
            context,
            stack_ptr_ptr.into(),
            ptr_type,
            location,
            LoadStoreOptions::default(),
        ))
        .result(0)?;

    let old_stack_ptr = block
        .append_operation(llvm::get_element_ptr(
            context,
            stack_ptr.into(),
            DenseI32ArrayAttribute::new(context, &[-1]),
            uint256.into(),
            ptr_type,
            location,
        ))
        .result(0)?;

    let value = block
        .append_operation(llvm::load(
            context,
            old_stack_ptr.into(),
            uint256.into(),
            location,
            LoadStoreOptions::default(),
        ))
        .result(0)?
        .into();

    let res = block.append_operation(llvm::store(
        context,
        old_stack_ptr.into(),
        stack_ptr.into(),
        location,
        LoadStoreOptions::default(),
    ));
    assert!(res.verify());

    Ok(value)
}

pub fn stack_push<'ctx>(
    context: &'ctx MeliorContext,
    block: &'ctx Block,
    value: Value,
) -> Result<(), CodegenError> {
    let location = Location::unknown(context);
    let ptr_type = pointer(context, 0);
    let stack_ptr_ptr = block
        .append_operation(llvm_mlir::addressof(
            context,
            STACK_PTR_GLOBAL,
            ptr_type,
            location,
        ))
        .result(0)?;

    let stack_ptr = block
        .append_operation(llvm::load(
            context,
            stack_ptr_ptr.into(),
            ptr_type,
            location,
            LoadStoreOptions::default(),
        ))
        .result(0)?;

    let uint256 = IntegerType::new(context, 256);

    let res = block.append_operation(llvm::store(
        context,
        value,
        stack_ptr.into(),
        location,
        LoadStoreOptions::default(),
    ));
    assert!(res.verify());

    let new_stack_ptr = block
        .append_operation(llvm::get_element_ptr(
            context,
            stack_ptr.into(),
            DenseI32ArrayAttribute::new(context, &[1]),
            uint256.into(),
            ptr_type,
            location,
        ))
        .result(0)?;

    let res = block.append_operation(llvm::store(
        context,
        new_stack_ptr.into(),
        stack_ptr_ptr.into(),
        location,
        LoadStoreOptions::default(),
    ));
    assert!(res.verify());

    Ok(())
}

pub mod llvm_mlir {
    use melior::{
        dialect::llvm::{self, attributes::Linkage},
        ir::{
            attribute::{FlatSymbolRefAttribute, StringAttribute, TypeAttribute},
            operation::OperationBuilder,
            Identifier, Location, Region,
        },
        Context as MeliorContext,
    };

    pub fn global<'c>(
        context: &'c MeliorContext,
        name: &str,
        global_type: melior::ir::Type<'c>,
        location: Location<'c>,
    ) -> melior::ir::Operation<'c> {
        // TODO: use ODS
        OperationBuilder::new("llvm.mlir.global", location)
            .add_regions([Region::new()])
            .add_attributes(&[
                (
                    Identifier::new(context, "sym_name"),
                    StringAttribute::new(context, name).into(),
                ),
                (
                    Identifier::new(context, "global_type"),
                    TypeAttribute::new(global_type).into(),
                ),
                (
                    Identifier::new(context, "linkage"),
                    llvm::attributes::linkage(context, Linkage::Internal),
                ),
            ])
            .build()
            .expect("valid operation")
    }

    pub fn addressof<'c>(
        context: &'c MeliorContext,
        name: &str,
        result_type: melior::ir::Type<'c>,
        location: Location<'c>,
    ) -> melior::ir::Operation<'c> {
        // TODO: use ODS
        OperationBuilder::new("llvm.mlir.addressof", location)
            .add_attributes(&[(
                Identifier::new(context, "global_name"),
                FlatSymbolRefAttribute::new(context, name).into(),
            )])
            .add_results(&[result_type])
            .build()
            .expect("valid operation")
    }
}
