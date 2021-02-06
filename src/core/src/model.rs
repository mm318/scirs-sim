use std::any::Any;
use std::marker::PhantomData;

use crate::block::{Block, BlockInput};
use crate::dag::Dag;

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
    dag: Dag<Box<dyn Block>>,
}

// struct ModelTraversal {
//     dag: Dag<'a, &'a dyn Block>,
//     ready: VecDeque<&'a Node<'a, &'a dyn Block>>,
// }

impl Model {
    pub fn new() -> Self {
        return Model { dag: Dag::new() };
    }

    pub fn add_block<B: Block>(&mut self, new_block: B) -> BlockHandle<B> {
        let id = self.dag.add_node(Box::new(new_block));
        return BlockHandle {
            block_id: id,
            data_type: PhantomData,
        };
    }

    pub fn get_block_by_id(&self, block_id: usize) -> &dyn Block {
        return &**self.dag.get_node(block_id).get_value();
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
        return self
            .dag
            .get_mut_node(block_handle.id())
            .get_mut_value()
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
        self.dag.connect(block1_handle.id(), block2_handle.id());
    }

    pub fn exec(&self, steps: usize) {
        for step in 0..steps {
            // debug
            println!("\nstep {}", step + 1);

            let mut bfs = self.dag.build_bfs().unwrap();
            loop {
                match self.dag.next_in_bfs(&bfs) {
                    Some(ref node) => {
                        println!("  Visiting {:?}", node);

                        self.dag.visited_in_bfs(&mut bfs, node);

                        node.get_value().calc(self);
                    }
                    None => {
                        break;
                    }
                }
            }

            for node in self.dag.iter_nodes() {
                node.get_value().update();
            }
        }
    }
}
