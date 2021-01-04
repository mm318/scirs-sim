use std::any::Any;
use std::marker::PhantomData;

use crate::block::{Block, BlockInput};
use crate::dag::node::Node;
use crate::dag::Dag;

pub trait BlockType {
    type BlockType;
}

pub struct BlockId<T> {
    block_idx: usize,
    data_type: PhantomData<T>,
}

impl<T> BlockType for BlockId<T> {
    type BlockType = T;
}

impl<T> BlockId<T> {
    pub fn idx(&self) -> usize {
        return self.block_idx;
    }
}

// #[derive(Debug)]
pub struct Model {
    blocks: Vec<Box<dyn Block>>,
}

impl Model {
    pub fn new() -> Self {
        return Model { blocks: Vec::new() };
    }

    pub fn add_block<B: Block>(&mut self, new_block: B) -> BlockId<B> {
        let idx = self.blocks.len();
        self.blocks.push(Box::new(new_block));
        return BlockId {
            block_idx: idx,
            data_type: PhantomData,
        };
    }

    pub fn get_block<B>(&self, block_id: &BlockId<B>) -> &dyn Block {
        return self.get_block_by_idx(block_id.idx());
    }

    pub fn get_block_by_idx(&self, block_idx: usize) -> &dyn Block {
        return &*self.blocks[block_idx];
    }

    fn get_concrete_block<B: Any>(&self, block_id: &BlockId<B>) -> &B {
        return self.blocks[block_id.idx()]
            .as_any()
            .downcast_ref::<B>()
            .unwrap();
    }

    fn get_mut_concrete_block<B: Any>(&mut self, block_id: &BlockId<B>) -> &mut B {
        return self.blocks[block_id.idx()]
            .as_mut_any()
            .downcast_mut::<B>()
            .unwrap();
    }

    pub fn connect<A: Block, B: Block, T: Default>(
        &mut self,
        block1_handle: &BlockId<A>,
        block1_output: fn(&dyn Block) -> T,
        block2_handle: &BlockId<B>,
        block2_input: fn(&mut B, BlockInput<T>),
    ) {
        (block2_input)(
            self.get_mut_concrete_block(block2_handle),
            BlockInput::new(block1_handle.idx(), block1_output),
        );
    }

    pub fn exec(&self, steps: usize) {
        // debug
        self.debug_state("step 0");
        println!();

        for step in 0..steps {
            for block in &self.blocks {
                block.calc(self);
            }
            for block in &self.blocks {
                block.update();
            }

            // debug
            self.debug_state(format!("step {}", step + 1).as_str());
            println!();
        }
    }

    fn debug_state(&self, state_id: &str) {
        println!("{}:", state_id);
        for (i, block) in self.blocks.iter().enumerate() {
            println!("  block_id {} = {:?}", i, block);
        }
    }
}
