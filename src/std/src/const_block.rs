use std::fmt::Debug;

use rust_model_core::block::Block;
use rust_model_core::model::Model;

#[derive(Debug)]
pub struct ConstBlock<T> {
    output1: T,
}

impl<T: 'static + Send + Sync + Debug> Block for ConstBlock<T> {
    fn calc(&self, _model: &Model) {}
    fn update(&self) {}
}

impl<T: 'static + Send + Sync + Clone> ConstBlock<T> {
    pub fn new(const_val: T) -> Self {
        return Self { output1: const_val };
    }

    pub fn get_output1(block: &dyn Block) -> T {
        return block
            .as_any()
            .downcast_ref::<Self>()
            .unwrap()
            ._get_output1();
    }

    fn _get_output1(&self) -> T {
        return self.output1.clone();
    }
}
