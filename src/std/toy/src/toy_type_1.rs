use rust_model_core::block::{Block, BlockInput};
use rust_model_core::model::Model;

#[derive(Default)]
pub struct ToyBlockType1 {
    input1_getter: BlockInput<i32>,
    curr_output1: i32,
    next_output1: i32,
}

impl Block for ToyBlockType1 {
    fn calc(&mut self, model: &Model) {
        let input1 = self.input1_getter.get_value(model);
        self.next_output1 = input1;
    }

    fn update(&mut self) {
        self.curr_output1 = self.next_output1;
    }
}

impl ToyBlockType1 {
    pub fn new() -> Self {
        return Self::default();
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
        return self.curr_output1;
    }
}
