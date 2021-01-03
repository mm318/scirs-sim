use rust_model_core::model::Model;
use rust_model_stdlib::toy::ToyBlockType1;
use rust_model_stdlib::ConstBlock;

#[test]
fn main() {
    let mut model = Model::new();

    //
    // create system components
    //
    let const_block_handle = model.add_block(ConstBlock::new(42));
    let block1_handle = model.add_block(ToyBlockType1::new(1));
    let block2_handle = model.add_block(ToyBlockType1::new(2));

    //
    // hook up system components
    //
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
    // old style:
    // model.get_mut_concrete_block(&block1_handle)
    //      .set_input1(&block2_handle, ToyBlockType1::get_output1);

    //
    // run the system for 3 time steps
    //
    model.exec(3);

    //
    // get the result of the system
    //
    let result = ToyBlockType1::get_output1(model.get_block(&block2_handle));
    // this will panic due to type mismatch
    // let result = ToyBlockType1::get_output1(model.get_block(&const_block_handle));

    println!("result: {}", result);
    assert_eq!(result, 42);
}
