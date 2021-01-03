use std::any::Any;
use std::fmt::Debug;

use crate::model::Model;

pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        return self;
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        return self;
    }
}

pub trait Block: AsAny + Debug {
    fn calc(&self, model: &Model);
    fn update(&self);
}

pub struct BlockInput<T> {
    block_id: usize,
    value_func: fn(&dyn Block) -> T,
}

impl<T: Default> Default for BlockInput<T> {
    fn default() -> Self {
        return BlockInput {
            block_id: usize::MAX,
            value_func: Self::_dummy_func,
        };
    }
}

impl<T: Default> BlockInput<T> {
    pub fn new(idx: usize, func: fn(&dyn Block) -> T) -> Self {
        return BlockInput {
            block_id: idx,
            value_func: func,
        };
    }

    fn _dummy_func(_block: &dyn Block) -> T {
        return Default::default();
    }

    pub fn set(&mut self, idx: usize, func: fn(&dyn Block) -> T) {
        self.block_id = idx;
        self.value_func = func;
    }

    pub fn get_value(&self, model: &Model) -> T {
        return (self.value_func)(model.get_block_by_idx(self.block_id));
    }
}

impl<T> Debug for BlockInput<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockInput")
            .field("block_id", &self.block_id)
            .field(
                "value_func",
                &format_args!("{:p}", self.value_func as *const ()),
            )
            .finish()
    }
}
