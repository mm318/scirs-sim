use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;

use tokio::task;

use crate::block::{Block, BlockInput};
use crate::dag::Dag;

enum CloneableOption<T> {
    Some(T),
    None,
}

impl<T> CloneableOption<T> {
    fn unwrap(&mut self) -> T {
        let mut result = CloneableOption::None;
        std::mem::swap(self, &mut result);
        return match result {
            Self::Some(val) => val,
            Self::None => panic!("called `CloneableOption::unwrap()` on a `None` value"),
        };
    }
}

impl<T> Clone for CloneableOption<T> {
    fn clone(&self) -> Self {
        return Self::None;
    }
}

// // specialization is not yet a stable feature
// impl<T: Clone> Clone for CloneableOption<T> {
//     fn clone(&self) -> Self {
//         return match self {
//             Some(x) => Some(x.clone()),
//             None => None,
//         };
//     }
// }

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

struct CalcBlockFuture {
    model: Arc<Model>,
    node_id: usize,
    futures: Arc<Vec<Mutex<CloneableOption<CalcBlockFuture>>>>,
}

impl Future for CalcBlockFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for from_node_id in self.model.dag.get_dependencies(&self.node_id) {
            match Pin::new(&mut self.futures[*from_node_id].lock().unwrap().unwrap()).poll(cx) {
                Poll::Ready(_) => continue,
                Poll::Pending => return Poll::Pending,
            }
        }

        self.model.dag.get_node(self.node_id).get_value().calc(self.model.as_ref());
        self.model.dag.get_node(self.node_id).get_value().update();
        return Poll::Ready(());
    }
}

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

    fn create_calc_block_future(
        self: Arc<Model>,
        node_id: usize,
        futures: Arc<Vec<Mutex<CloneableOption<CalcBlockFuture>>>>,
    ) {
        let future = CloneableOption::Some(CalcBlockFuture{model: self, node_id: node_id, futures: futures.clone()});
        *futures[node_id].lock().unwrap() = future;
    }

    pub fn exec(self: Arc<Model>, steps: &usize) {
        for step in 0..*steps {
            // debug
            println!("\nstep {}", step + 1);

            let mut futures_vec = Vec::<Mutex<CloneableOption<CalcBlockFuture>>>::with_capacity(
                    self.dag.get_num_nodes(),
                );

            // futures_vec.resize(self.dag.get_num_nodes(), Mutex::new(CloneableOption::None));
            for _ in 0..self.dag.get_num_nodes() {
                futures_vec.push(Mutex::new(CloneableOption::None));
            }

            // each future will have a copy of this Arc
            let futures_vec_arc = Arc::new(futures_vec);

            let mut bfs = self.dag.build_bfs().unwrap();
            loop {
                match self.dag.next_in_bfs(&bfs) {
                    Some(ref node) => {
                        println!("  Visiting {:?}", node);

                        self.dag.visited_in_bfs(&mut bfs, node);

                        let self_copy = self.clone();
                        let node_id = *node.get_id();
                        let futures_vec_copy = futures_vec_arc.clone();
                        self_copy.create_calc_block_future(node_id, futures_vec_copy);
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}
