// Based on https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cell::RefCell;
use std::cmp::Eq;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;

use crate::dag::node::Node;

enum CycleCheckStatus {
    Initial,
    Processing,
    Processed,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Dag<T: Eq + Debug> {
    nodes: Vec<Node<T>>,
    edges: Vec<(usize, usize)>,

    // visitation info
    roots: RefCell<HashSet<usize>>,
}

impl<T: Eq + Debug> Dag<T> {
    pub fn new() -> Self {
        return Dag {
            nodes: Vec::new(),
            edges: Vec::new(),
            roots: RefCell::new(HashSet::new()),
        };
    }

    pub fn add_node(&mut self, value: T) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node::new(id, value));
        return id;
    }

    pub fn connect(&mut self, from_node_id: usize, to_node_id: usize) {
        self.edges.push((from_node_id, to_node_id));
    }

    pub fn get_node(&self, node_id: usize) -> &Node<T> {
        return &self.nodes[node_id];
    }

    pub fn get_mut_node(&mut self, node_id: usize) -> &mut Node<T> {
        return &mut self.nodes[node_id];
    }

    // find roots
    pub fn build_bfs_visit(&self) -> bool {
        self.roots.replace(HashSet::new());
        for node in &self.nodes {
            node.dependencies.borrow_mut().clear();
            node.dependants.borrow_mut().clear();
        }
        for (from_node_id, to_node_id) in &self.edges {
            self.get_node(*from_node_id)
                .add_dependant(&self.get_node(*to_node_id));
            self.get_node(*to_node_id)
                .add_dependency(&self.get_node(*from_node_id));
        }

        for node in &self.nodes {
            if node.dependencies.borrow().is_empty() {
                self.roots.borrow_mut().insert(node.id);
            }
        }

        return self.check();
    }

    fn check(&self) -> bool {
        if self.roots.borrow().is_empty() {
            panic!("No roots found. DAG is invalid!");
        }

        if self
            .roots
            .borrow()
            .iter()
            .all(|root_id| self._check(root_id, &mut HashMap::new()))
        {
            return true;
        } else {
            panic!("Invalid DAG detected")
        }
    }

    fn _check(&self, pt: &usize, visited: &mut HashMap<usize, CycleCheckStatus>) -> bool {
        visited.insert(*pt, CycleCheckStatus::Processing);

        let deps = self.get_node(*pt).dependants.borrow();

        for dep in deps.iter() {
            let status = match visited.get(dep) {
                Some(v) => v,
                None => &CycleCheckStatus::Initial,
            };

            match status {
                CycleCheckStatus::Initial => {
                    if !self._check(dep, visited) {
                        return false;
                    }
                }
                CycleCheckStatus::Processing => return false,
                CycleCheckStatus::Processed => {}
            }
        }

        visited.insert(*pt, CycleCheckStatus::Processed);

        true
    }

    pub fn remove_as_dependency(&self, node_id: usize) {
        let to_remove = &self.get_node(node_id);

        for id in to_remove.dependants.borrow().iter() {
            self.get_node(*id)
                .dependencies
                .borrow_mut()
                .remove(&to_remove.id);
        }

        self.roots.borrow_mut().remove(&to_remove.id);

        for id in to_remove.dependants.borrow().iter() {
            if self.get_node(*id).dependencies.borrow().is_empty() {
                self.roots.borrow_mut().insert(*id);
            }
        }
    }

    pub fn next_bfs_visit(&self) -> Option<&Node<T>> {
        for id in self.roots.borrow().iter() {
            return Some(&self.get_node(*id));
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::dag::node::Node;
    use crate::dag::Dag;
    use std::collections::HashSet;

    #[derive(Hash, Eq, PartialEq, Debug)]
    struct MockStruct {
        id: char,
    }

    impl MockStruct {
        fn new(id: char) -> MockStruct {
            MockStruct { id }
        }
    }

    #[test]
    #[should_panic]
    fn build_dag() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('a'));
        let c = dag.add_node(MockStruct::new('C'));
        let d = dag.add_node(MockStruct::new('D'));
        let e = dag.add_node(MockStruct::new('E'));
        let f = dag.add_node(MockStruct::new('F'));
        let g = dag.add_node(MockStruct::new('G'));
        let h = dag.add_node(MockStruct::new('H'));

        dag.connect(a, b);
        dag.connect(b, c);
        dag.connect(c, d);
        dag.connect(d, e);
        dag.connect(d, f);
        dag.connect(f, g);
        dag.connect(f, h);
        dag.connect(d, b); // causes circular dependency

        dag.build();
    }

    #[test]
    fn remove_nodes() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('B'));

        dag.connect(a, b);

        dag.build();

        assert!(
            !dag.get_node(b).dependencies.borrow().is_empty(),
            "Node was not successfully removed"
        );

        dag.remove_as_dependency(a);

        assert!(
            dag.get_node(b).dependencies.borrow().is_empty(),
            "Node was not successfully removed"
        );
    }

    #[test]
    fn node_hash() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('a'));
        let c = dag.add_node(MockStruct::new('C'));
        let d = dag.add_node(MockStruct::new('D'));
        let e = dag.add_node(MockStruct::new('E'));
        let f = dag.add_node(MockStruct::new('F'));
        let g = dag.add_node(MockStruct::new('G'));
        let h = dag.add_node(MockStruct::new('H'));

        let mut hash: HashSet<&Node<MockStruct>> = HashSet::new();

        hash.insert(dag.get_node(a));
        hash.insert(dag.get_node(b));
        hash.insert(dag.get_node(c));
        hash.insert(dag.get_node(d));
        hash.insert(dag.get_node(e));
        hash.insert(dag.get_node(f));
        hash.insert(dag.get_node(g));
        hash.insert(dag.get_node(h));

        assert!(hash.contains(dag.get_node(a)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(b)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(c)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(d)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(e)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(f)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(g)), "Node did not hash properly");
        assert!(hash.contains(dag.get_node(h)), "Node did not hash properly");
    }
}
