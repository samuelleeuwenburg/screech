use alloc::vec;
use alloc::vec::Vec;
use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq)]
pub enum Mark {
    Temporary,
    Permanent,
}

fn visit(
    nodes: &FxHashMap<usize, Vec<usize>>,
    marks: &mut FxHashMap<usize, Mark>,
    result: &mut Vec<usize>,
    node: usize,
) {
    match marks.get(&node) {
        Some(Mark::Permanent) => (),
        Some(Mark::Temporary) => {
            marks.insert(node, Mark::Permanent);
        }
        None => {
            marks.insert(node, Mark::Temporary);

            if let Some(sources) = nodes.get(&node) {
                for &node in sources {
                    visit(nodes, marks, result, node);
                }
            }

            marks.insert(node, Mark::Permanent);
            result.push(node);
        }
    }
}

pub fn topological_sort(nodes: FxHashMap<usize, Vec<usize>>) -> Vec<usize> {
    let mut marks = FxHashMap::<usize, Mark>::default();
    let mut result = vec![];

    for &node in nodes.keys() {
        if marks.get(&node).is_none() {
            visit(&nodes, &mut marks, &mut result, node);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort() {
        let mut graph = FxHashMap::<usize, Vec<usize>>::default();
        graph.insert(0, vec![2, 4, 1]);
        graph.insert(1, vec![]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1]);
        graph.insert(4, vec![2]);

        let sorted = topological_sort(graph);

        assert_eq!(sorted, vec![1, 3, 2, 4, 0]);
    }

    #[test]
    fn test_topological_sort_cyclic_graph() {
        let mut graph = FxHashMap::<usize, Vec<usize>>::default();
        graph.insert(0, vec![2]);
        graph.insert(1, vec![0]);
        graph.insert(2, vec![1]);

        let sorted = topological_sort(graph);

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted.contains(&0), true);
        assert_eq!(sorted.contains(&1), true);
        assert_eq!(sorted.contains(&2), true);
    }

    #[test]
    fn test_topological_sort_combo() {
        let mut graph = FxHashMap::<usize, Vec<usize>>::default();
        // second layer
        graph.insert(0, vec![2, 3]);
        graph.insert(1, vec![2, 3]);

        // first layer
        graph.insert(2, vec![]);
        graph.insert(3, vec![]);

        // third layer, cycle
        graph.insert(4, vec![0, 1, 6]);
        graph.insert(5, vec![0, 1, 4]);
        graph.insert(6, vec![0, 1, 5]);

        let sorted = topological_sort(graph);

        assert_eq!(sorted[..2].contains(&2), true);
        assert_eq!(sorted[..2].contains(&3), true);
        assert_eq!(sorted[2..4].contains(&0), true);
        assert_eq!(sorted[2..4].contains(&1), true);
        assert_eq!(sorted[4..].contains(&4), true);
        assert_eq!(sorted[4..].contains(&5), true);
        assert_eq!(sorted[4..].contains(&6), true);
    }
}
