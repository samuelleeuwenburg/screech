use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;

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
    nodes: &HashMap<usize, Vec<usize>>,
    marks: &mut HashMap<usize, Mark>,
    result: &mut Vec<usize>,
    node: usize,
) -> Result<(), Error> {
    match marks.get(&node) {
        Some(Mark::Permanent) => Ok(()),
        Some(Mark::Temporary) => Err(Error::NoDirectedAcyclicGraph),
        None => {
            marks.insert(node, Mark::Temporary);
            match nodes.get(&node) {
                Some(sources) => {
                    for &node in sources {
                        visit(nodes, marks, result, node)?;
                    }
                }
                None => (),
            }

            marks.insert(node, Mark::Permanent);
            result.push(node);
            Ok(())
        }
    }
}

pub fn topological_sort(nodes: HashMap<usize, Vec<usize>>) -> Result<Vec<usize>, Error> {
    let mut marks = HashMap::<usize, Mark>::new();
    let mut result = vec![];

    for (&node, _) in &nodes {
        if let None = marks.get(&node) {
            visit(&nodes, &mut marks, &mut result, node)?;
        }
    }

    result.reverse();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort() {
        let mut graph = HashMap::<usize, Vec<usize>>::new();
        graph.insert(0, vec![2, 4, 1]);
        graph.insert(1, vec![]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1]);
        graph.insert(4, vec![2]);

        assert_eq!(topological_sort(graph), Ok(vec![0, 4, 2, 3, 1]));
    }

    #[test]
    fn test_topological_sort_cyclic_graph() {
        let mut graph = HashMap::<usize, Vec<usize>>::new();
        graph.insert(0, vec![2, 4, 1]);
        graph.insert(1, vec![]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1, 4]);
        graph.insert(4, vec![2]);

        assert_eq!(topological_sort(graph), Err(Error::NoDirectedAcyclicGraph));
    }
}
