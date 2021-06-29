use rust_model_core::model::Model;
use rust_model_stdlib::toy::ToyBlockType1;
use rust_model_stdlib::ConstBlock;
use std::sync::Arc;

use tokio;

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    let mut model = Model::new();

    //
    // create system components
    //
    let block1_handle = model.add_block(ToyBlockType1::new(1));
    let block2_handle = model.add_block(ToyBlockType1::new(2));
    let const_block_handle = model.add_block(ConstBlock::new(42));

    //
    // hook up system components
    //
    if cfg!(debug_assertions) {
        println!("Hooking up system components");
    }
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

    //
    // run the system for 3 time steps
    //
    if cfg!(debug_assertions) {
        println!("Executing model");
    }
    let model_arc = Arc::new(model);
    model_arc.clone().exec(&3).await;

    //
    // get the result of the system
    //
    let result = ToyBlockType1::get_output1(model_arc.get_block(&block2_handle));
    println!("result: {}", result);
    assert_eq!(result, 42);
}
