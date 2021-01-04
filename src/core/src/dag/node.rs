// Taken from https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(Eq)]
pub struct Node<'a, T>
where
    T: Eq + Hash + Debug,
{
    pub value: T,
    pub dependencies: RefCell<HashSet<&'a Node<'a, T>>>,
    pub dependants: RefCell<HashSet<&'a Node<'a, T>>>,
}

impl<'a, T> PartialEq for Node<'a, T>
where
    T: Eq + Hash + Debug,
{
    fn eq(&self, other: &Node<'a, T>) -> bool {
        self.value == other.value
    }
}

impl<'a, T> Debug for Node<'a, T>
where
    T: Eq + Hash + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Node: {:?})", self.value)
    }
}

impl<'a, T> Hash for Node<'a, T>
where
    T: Eq + Hash + Debug,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<'a, T> Node<'a, T>
where
    T: Eq + Hash + Debug,
{
    pub fn new(v: T) -> Node<'a, T> {
        Node {
            value: v,
            dependencies: RefCell::new(HashSet::new()),
            dependants: RefCell::new(HashSet::new()),
        }
    }

    // dependencies are upstream
    pub fn add_dependency(&self, dep: &'a Node<'a, T>) {
        self.dependencies.borrow_mut().insert(dep);
    }

    // dependants are downstream
    pub fn add_dependant(&self, dep: &'a Node<'a, T>) {
        self.dependants.borrow_mut().insert(dep);
    }

    pub fn remove(&self) {
        for dependant in self.dependants.borrow().iter() {
            dependant.dependencies.borrow_mut().remove(self);
        }
    }
}
