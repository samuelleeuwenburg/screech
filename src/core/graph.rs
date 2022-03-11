use alloc::vec;
use alloc::vec::Vec;
use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq)]
pub enum Error {
    NoDirectedAcyclicGraph,
}

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
) -> Result<(), Error> {
    match marks.get(&node) {
        Some(Mark::Permanent) => Ok(()),
        Some(Mark::Temporary) => Err(Error::NoDirectedAcyclicGraph),
        None => {
            marks.insert(node, Mark::Temporary);

            if let Some(sources) = nodes.get(&node) {
                for &node in sources {
                    visit(nodes, marks, result, node)?;
                }
            }

            marks.insert(node, Mark::Permanent);
            result.push(node);
            Ok(())
        }
    }
}

pub fn topological_sort(nodes: FxHashMap<usize, Vec<usize>>) -> Result<Vec<usize>, Error> {
    let mut marks = FxHashMap::<usize, Mark>::default();
    let mut result = vec![];

    for &node in nodes.keys() {
        if marks.get(&node).is_none() {
            visit(&nodes, &mut marks, &mut result, node)?;
        }
    }

    Ok(result)
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

        assert_eq!(topological_sort(graph), Ok(vec![1, 3, 2, 4, 0]));
    }

    #[test]
    fn test_topological_sort_cyclic_graph() {
        let mut graph = FxHashMap::<usize, Vec<usize>>::default();
        graph.insert(0, vec![2, 4, 1]);
        graph.insert(1, vec![]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1, 4]);
        graph.insert(4, vec![2]);

        assert_eq!(topological_sort(graph), Err(Error::NoDirectedAcyclicGraph));
    }
}
