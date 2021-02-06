use std::any::Any;
use std::marker::PhantomData;

use crate::block::{Block, BlockInput};

pub trait BlockType {
    type BlockType;
}

pub struct BlockHandle<T> {
    block_id: usize,
    data_type: PhantomData<T>,
}

impl<T> BlockType for BlockHandle<T> {
    type BlockType = T;
}

impl<T> BlockHandle<T> {
    pub fn id(&self) -> usize {
        return self.block_id;
    }
}

pub struct Model {
    blocks: Vec<Box<dyn Block>>,
}

// struct ModelTraversal {
//     dag: Dag<'a, &'a dyn Block>,
//     ready: VecDeque<&'a Node<'a, &'a dyn Block>>,
// }

impl Model {
    pub fn new() -> Self {
        return Model { blocks: Vec::new() };
    }

    pub fn add_block<B: Block>(&mut self, new_block: B) -> BlockHandle<B> {
        let id = self.blocks.len();
        self.blocks.push(Box::new(new_block));
        return BlockHandle {
            block_id: id,
            data_type: PhantomData,
        };
    }

    pub fn get_block_by_id(&self, block_id: usize) -> &dyn Block {
        return &*self.blocks[block_id];
    }

    pub fn get_block<B>(&self, block_handle: &BlockHandle<B>) -> &dyn Block {
        return self.get_block_by_id(block_handle.id());
    }

    fn get_concrete_block<B: Any>(&self, block_handle: &BlockHandle<B>) -> &B {
        return self
            .get_block(block_handle)
            .as_any()
            .downcast_ref::<B>()
            .unwrap();
    }

    fn get_mut_concrete_block<B: Any>(&mut self, block_handle: &BlockHandle<B>) -> &mut B {
        return self.blocks[block_handle.id()]
            .as_mut_any()
            .downcast_mut::<B>()
            .unwrap();
    }

    pub fn connect<A: Block, B: Block, T: Default>(
        &mut self,
        block1_handle: &BlockHandle<A>,
        block1_output: fn(&dyn Block) -> T,
        block2_handle: &BlockHandle<B>,
        block2_input: fn(&mut B, BlockInput<T>),
    ) {
        (block2_input)(
            self.get_mut_concrete_block(block2_handle),
            BlockInput::new(block1_handle.id(), block1_output),
        );
    }

    pub fn exec(&self, steps: usize) {
        for step in 0..steps {
            // debug
            println!("\nstep {}", step + 1);

            for block in &self.blocks {
                println!("  Visiting {:?}", block);
                block.calc(self);
            }

            for block in &self.blocks {
                block.update();
            }
        }
    }
}
