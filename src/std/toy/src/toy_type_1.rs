#![allow(non_snake_case)]

use rust_model_core::model::{Model, BlockId};
use rust_model_core::block::{Block, BlockInput};


#[derive(Default)]
pub struct ToyBlockType1 {
    input1: BlockInput<i32>,
    curr_output1: i32,
    next_output1: i32
}

impl Block for ToyBlockType1 {
    fn calc(&mut self, model: &Model) {
        self.next_output1 = self.input1.get_value(model);
    }

    fn update(&mut self) {
        self.curr_output1 = self.next_output1;
    }
}

impl ToyBlockType1 {
    pub fn new() -> Self {
        return ToyBlockType1::default();
    }

    pub fn set_input1<T>(&mut self, input_block: &BlockId<T>, input_func: fn(&Block) -> i32) {
        self.input1.set(input_block.idx(), input_func);
    }

    pub fn get_output1(block: &Block) -> i32 {
        return block.as_any().downcast_ref::<ToyBlockType1>().unwrap()._get_output1();
    }

    fn _get_output1(&self) -> i32 {
        return self.curr_output1;
    }
}
