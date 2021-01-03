use std::cell::Cell;

use rust_model_core::block::{Block, BlockInput};
use rust_model_core::model::Model;

#[derive(Debug)]
pub struct ToyBlockType1 {
    input1_getter: BlockInput<i32>,
    curr_output1: Cell<i32>,
    next_output1: Cell<i32>,
}

impl Block for ToyBlockType1 {
    fn calc(&self, model: &Model) {
        let input1 = self.input1_getter.get_value(model);
        self.next_output1.set(input1);
    }

    fn update(&self) {
        self.curr_output1.set(self.next_output1.get());
    }
}

impl ToyBlockType1 {
    pub fn new(initial_value: i32) -> Self {
        return Self {
            input1_getter: Default::default(),
            curr_output1: Cell::new(initial_value),
            next_output1: Cell::new(i32::MIN),
        };
    }

    pub fn set_input1(&mut self, block_input: BlockInput<i32>) {
        self.input1_getter = block_input;
    }

    pub fn get_output1(block: &dyn Block) -> i32 {
        return block
            .as_any()
            .downcast_ref::<Self>()
            .unwrap()
            ._get_output1();
    }

    fn _get_output1(&self) -> i32 {
        return self.curr_output1.get();
    }
}
