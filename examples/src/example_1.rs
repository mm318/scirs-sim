#![allow(non_snake_case)]

extern crate rust_model_core;
use rust_model_core::model::Model;

extern crate rust_model_stdlib;
use rust_model_stdlib::rust_model_stdlib_toy::ToyBlockType1;


fn main() {
    let mut model = Model::new();
    
    let block1_handle = model.add_block(ToyBlockType1::new());
    let block2_handle = model.add_block(ToyBlockType1::new());

    model.get_mut_block(&block1_handle).set_input1(&block2_handle, ToyBlockType1::get_output1);
}
