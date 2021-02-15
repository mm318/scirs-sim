// Based on https://github.com/bunker-inspector/rs_taskflow/tree/master/src/flow/dag

use std::cmp::Eq;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::Send;
use std::slice::Iter;

use crate::dag::node::Node;

enum CycleCheckStatus {
    Initial,
    Processing,
    Processed,
}

pub struct DagVisitationInfo {
    dependencies: Vec<HashSet<usize>>,
    dependants: Vec<HashSet<usize>>,
    roots: HashSet<usize>,
}

impl DagVisitationInfo {
    fn new(len: usize) -> Self {
        let mut result = DagVisitationInfo {
            dependencies: Vec::with_capacity(len),
            dependants: Vec::with_capacity(len),
            roots: HashSet::new(),
        };

        result.dependencies.resize(len, HashSet::new());
        result.dependants.resize(len, HashSet::new());

        return result;
    }

    fn check(self) -> Result<Self, &'static str> {
        if self.get_next_root().is_none() {
            return Err("No roots found. DAG is invalid!");
        }

        if self
            .get_roots()
            .iter()
            .all(|root_id| self._check(root_id, &mut HashMap::new()))
        {
            return Ok(self);
        } else {
            return Err("Invalid DAG detected");
        }
    }

    fn _check(&self, curr_node_id: &usize, visited: &mut HashMap<usize, CycleCheckStatus>) -> bool {
        visited.insert(*curr_node_id, CycleCheckStatus::Processing);

        for dep in self.get_dependants(curr_node_id).iter() {
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

        visited.insert(*curr_node_id, CycleCheckStatus::Processed);

        return true;
    }

    // dependencies are upstream
    fn add_dependency(&mut self, from_node_id: &usize, to_node_id: &usize) {
        self.dependencies[*to_node_id].insert(*from_node_id);
    }

    // dependants are downstream
    fn add_dependant(&mut self, from_node_id: &usize, to_node_id: &usize) {
        self.dependants[*from_node_id].insert(*to_node_id);
    }

    fn add_relationship(&mut self, from_node_id: &usize, to_node_id: &usize) {
        self.add_dependant(from_node_id, to_node_id);
        self.add_dependency(from_node_id, to_node_id);
    }

    fn add_root_node(&mut self, node_id: &usize) {
        self.roots.insert(*node_id);
    }

    fn remove_root_node(&mut self, node_id: &usize) {
        self.roots.remove(node_id);
    }

    fn get_next_root(&self) -> Option<&usize> {
        return self.roots.iter().next();
    }

    fn get_roots(&self) -> &HashSet<usize> {
        return &self.roots;
    }

    fn get_dependencies(&self, node_id: &usize) -> &HashSet<usize> {
        return &self.dependencies[*node_id];
    }

    // fn remove_dependency(&mut self, from_node_id: &usize, to_node_id: &usize) {
    //     self.dependencies[*to_node_id].remove(from_node_id);
    // }

    fn get_dependants(&self, node_id: &usize) -> &HashSet<usize> {
        return &self.dependants[*node_id];
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Dag<T: Eq + Debug> {
    nodes: Vec<Node<T>>,
    edges: Vec<(usize, usize)>, // (from_node_id, to_node_id)
}

impl<T: Eq + Debug> Dag<T> {
    pub fn new() -> Self {
        return Dag {
            nodes: Vec::new(),
            edges: Vec::new(),
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

    pub fn get_num_nodes(&self) -> usize {
        return self.nodes.len();
    }

    pub fn get_node(&self, node_id: usize) -> &Node<T> {
        return &self.nodes[node_id];
    }

    pub fn get_mut_node(&mut self, node_id: usize) -> &mut Node<T> {
        return &mut self.nodes[node_id];
    }

    pub fn iter_nodes(&self) -> Iter<'_, Node<T>> {
        return self.nodes.iter();
    }

    // find roots
    pub fn build_bfs(&self) -> Result<DagVisitationInfo, &str> {
        let mut bfs = DagVisitationInfo::new(self.get_num_nodes());

        for (from_node_id, to_node_id) in &self.edges {
            bfs.add_relationship(from_node_id, to_node_id);
        }

        for node in &self.nodes {
            if bfs.get_dependencies(node.get_id()).is_empty() {
                bfs.add_root_node(node.get_id());
            }
        }

        return bfs.check();
    }

    pub fn next_in_bfs(&self, visitation_info: &DagVisitationInfo) -> Option<&Node<T>> {
        return match visitation_info.get_next_root() {
            Some(id) => Some(&self.get_node(*id)),
            None => None,
        };
    }

    pub fn visited_in_bfs(&self, visitation_info: &mut DagVisitationInfo, node: &Node<T>) {
        for id in visitation_info.dependants[*node.get_id()].iter() {
            visitation_info.dependencies[*id].remove(node.get_id());
        }

        visitation_info.remove_root_node(node.get_id());

        for id in visitation_info.dependants[*node.get_id()].iter() {
            if visitation_info.dependencies[*id].is_empty() {
                visitation_info.roots.insert(*id);
            }
        }
    }
}

unsafe impl<T: Eq + Debug> Send for Dag<T> {}

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

        let bfs = dag.build_bfs();
        assert!(bfs.is_ok());
    }

    #[test]
    fn build_dag_with_circular_dependency() {
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

        let bfs = dag.build_bfs();
        assert!(bfs.is_err());
    }

    #[test]
    fn remove_nodes() {
        let mut dag = Dag::new();

        let a = dag.add_node(MockStruct::new('A'));
        let b = dag.add_node(MockStruct::new('B'));

        dag.connect(a, b);

        let mut bfs = dag.build_bfs().unwrap();

        assert!(
            !bfs.get_dependencies(&b).is_empty(),
            "Node was not successfully removed"
        );

        dag.visited_in_bfs(&mut bfs, dag.get_node(a));

        assert!(
            bfs.get_dependencies(&b).is_empty(),
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
