// Based on https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(Eq)]
pub struct Node<T: Eq + Debug> {
    pub id: usize,
    pub value: T,

    // visitation info
    pub dependencies: RefCell<HashSet<usize>>,
    pub dependants: RefCell<HashSet<usize>>,
}

impl<T: Eq + Debug> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        self.value == other.value
    }
}

impl<T: Eq + Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Node {}: {:?})", self.id, self.value)
    }
}

impl<T: Eq + Debug> Hash for Node<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Eq + Debug> Node<T> {
    pub fn new(i: usize, v: T) -> Node<T> {
        Node {
            id: i,
            value: v,
            dependencies: RefCell::new(HashSet::new()),
            dependants: RefCell::new(HashSet::new()),
        }
    }

    // dependencies are upstream
    pub fn add_dependency(&self, dep: &Node<T>) {
        self.dependencies.borrow_mut().insert(dep.id);
    }

    // dependants are downstream
    pub fn add_dependant(&self, dep: &Node<T>) {
        self.dependants.borrow_mut().insert(dep.id);
    }
}
