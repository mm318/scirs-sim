use rust_model_core::model::Model;
use rust_model_stdlib::toy::ToyBlockType1;
use rust_model_stdlib::ConstBlock;

fn main() {
    let mut model = Model::new();

    let const_block_handle = model.add_block(ConstBlock::new(42 as i32));
    let block1_handle = model.add_block(ToyBlockType1::new());
    let block2_handle = model.add_block(ToyBlockType1::new());

    // model.get_mut_block(&block1_handle).set_input1(&block2_handle, ToyBlockType1::get_output1);

    model.connect(
        &const_block_handle,
        ConstBlock::get_output1,
        &block1_handle,
        ToyBlockType1::set_input1,
    );

    model.connect(
        &block1_handle,
        ToyBlockType1::get_output1,
        &block2_handle,
        ToyBlockType1::set_input1,
    );

    model.exec();
}
