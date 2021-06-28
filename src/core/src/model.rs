use std::any::Any;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use tokio::task;
use tokio::task::JoinHandle;

use crate::block::{Block, BlockInput};
use crate::dag::Dag;

// enum CloneableOption<T> {
//     Some(T),
//     None,
// }
//
// impl<T> CloneableOption<T> {
//     fn unwrap(&mut self) -> T {
//         let mut result = CloneableOption::None;
//         std::mem::swap(self, &mut result);
//         return match result {
//             Self::Some(val) => val,
//             Self::None => panic!("called `CloneableOption::unwrap()` on a `None` value"),
//         };
//     }
// }
//
// impl<T> Clone for CloneableOption<T> {
//     fn clone(&self) -> Self {
//         return Self::None;
//     }
// }
//
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

struct JoinHandleWithCompletionFlag {
    join_handle: JoinHandle<()>,
    completed: bool,
}

struct CalcBlockFuture {
    model: Arc<Model>,
    node_id: usize,
    futures: Arc<Vec<Mutex<Option<JoinHandleWithCompletionFlag>>>>,
}

impl Future for CalcBlockFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for from_node_id in self.model.dag.get_dependencies(&self.node_id) {
            if cfg!(debug_assertions) {
                match *self.futures[*from_node_id].lock().unwrap() {
                    Option::Some(_) => {}
                    Option::None => println!("Node {} has no future!", *from_node_id),
                }
            }

            let mut locked_guard = self.futures[*from_node_id].lock().unwrap();
            let join_handle = locked_guard.as_mut().unwrap();
            if join_handle.completed {
                continue;
            } else {
                match Pin::new(&mut join_handle.join_handle).poll(cx) {
                    Poll::Ready(_) => continue,
                    Poll::Pending => return Poll::Pending,
                }
            }
        }

        self.model
            .dag
            .get_node(self.node_id)
            .get_value()
            .calc(self.model.as_ref());
        self.model.dag.get_node(self.node_id).get_value().update();

        self.futures[self.node_id]
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .completed = true;

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

    fn spawn_calc_block_task(
        self: Arc<Model>,
        node_id: usize,
        futures: Arc<Vec<Mutex<Option<JoinHandleWithCompletionFlag>>>>,
    ) {
        if cfg!(debug_assertions) {
            println!("Adding future for node {}", node_id);
        }

        let future_task = task::spawn(CalcBlockFuture {
            model: self,
            node_id: node_id,
            futures: futures.clone(),
        });
        *futures[node_id].lock().unwrap() = Option::Some(JoinHandleWithCompletionFlag {
            join_handle: future_task,
            completed: false,
        });

        if cfg!(debug_assertions) {
            match *futures.clone()[node_id].lock().unwrap() {
                Option::Some(_) => println!("Node {} has a future!", node_id),
                Option::None => println!("Node {} has no future!", node_id),
            }
        }
    }

    pub async fn exec(self: Arc<Model>, steps: &usize) {
        for step in 0..*steps {
            if cfg!(debug_assertions) {
                println!("\nstep {}", step + 1);
            }

            let mut futures_vec = Vec::<Mutex<Option<JoinHandleWithCompletionFlag>>>::with_capacity(
                self.dag.get_num_nodes(),
            );

            // futures_vec.resize(self.dag.get_num_nodes(), Mutex::new(Option::None));
            for _ in 0..self.dag.get_num_nodes() {
                futures_vec.push(Mutex::new(Option::None));
            }

            // each future will have a copy of this Arc
            let futures_vec_arc = Arc::new(futures_vec);

            let mut bfs = self.dag.build_bfs().unwrap();
            loop {
                match self.dag.next_in_bfs(&bfs) {
                    Some(ref node) => {
                        if cfg!(debug_assertions) {
                            println!("  Visiting {:?}", node);
                        }

                        self.dag.visited_in_bfs(&mut bfs, node);

                        let self_copy = self.clone();
                        let node_id = *node.get_id();
                        let futures_vec_copy = futures_vec_arc.clone();
                        self_copy.spawn_calc_block_task(node_id, futures_vec_copy);
                    }
                    None => {
                        break;
                    }
                }
            }

            for future_iter in futures_vec_arc.iter() {
                let mut locked_guard = future_iter.lock().unwrap();
                let join_handle = locked_guard.as_mut().unwrap();
                if join_handle.completed {
                    continue;
                } else {
                    let _ = (&mut join_handle.join_handle).await;
                }
            }
        }
    }
}
