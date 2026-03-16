use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphTopologyAnalysis<T> {
    pub ordered_nodes: Vec<T>,
    pub blocked_nodes: Vec<T>,
}

impl<T> GraphTopologyAnalysis<T> {
    pub fn has_cycle_or_blocked_nodes(&self) -> bool {
        !self.blocked_nodes.is_empty()
    }
}

pub fn connected_input_mask(
    input_count: usize,
    edges: impl IntoIterator<Item = usize>,
) -> Vec<bool> {
    let mut connected = vec![false; input_count];
    for input_index in edges {
        if let Some(slot) = connected.get_mut(input_index) {
            *slot = true;
        }
    }
    connected
}

pub fn analyze_graph_topology<T, NI, EI>(nodes: NI, edges: EI) -> GraphTopologyAnalysis<T>
where
    T: Copy + Eq + Hash,
    NI: IntoIterator<Item = T>,
    EI: IntoIterator<Item = (T, T)>,
{
    let ordered_input_nodes: Vec<T> = nodes.into_iter().collect();
    let mut adjacency: HashMap<T, HashSet<T>> = HashMap::new();
    let mut in_degree: HashMap<T, usize> = HashMap::new();

    for node in &ordered_input_nodes {
        adjacency.entry(*node).or_default();
        in_degree.entry(*node).or_insert(0);
    }

    for (from, to) in edges {
        if !in_degree.contains_key(&from) || !in_degree.contains_key(&to) {
            continue;
        }

        if adjacency.entry(from).or_default().insert(to) {
            *in_degree.entry(to).or_insert(0) += 1;
        }
    }

    let mut queue = VecDeque::new();
    for node in &ordered_input_nodes {
        if in_degree.get(node).copied().unwrap_or_default() == 0 {
            queue.push_back(*node);
        }
    }

    let mut ordered_nodes = Vec::with_capacity(ordered_input_nodes.len());
    let mut processed = HashSet::with_capacity(ordered_input_nodes.len());

    while let Some(node) = queue.pop_front() {
        if !processed.insert(node) {
            continue;
        }

        ordered_nodes.push(node);

        if let Some(dependents) = adjacency.get(&node) {
            for dependent in dependents {
                if let Some(degree) = in_degree.get_mut(dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(*dependent);
                    }
                }
            }
        }
    }

    let blocked_nodes = ordered_input_nodes
        .into_iter()
        .filter(|node| !processed.contains(node))
        .collect();

    GraphTopologyAnalysis {
        ordered_nodes,
        blocked_nodes,
    }
}

pub fn would_create_cycle<T, EI>(edges: EI, from: T, to: T) -> bool
where
    T: Copy + Eq + Hash,
    EI: IntoIterator<Item = (T, T)>,
{
    if from == to {
        return true;
    }

    let mut adjacency: HashMap<T, Vec<T>> = HashMap::new();
    for (edge_from, edge_to) in edges {
        adjacency.entry(edge_from).or_default().push(edge_to);
    }

    let mut queue = VecDeque::from([to]);
    let mut visited = HashSet::new();

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current) {
            continue;
        }

        if current == from {
            return true;
        }

        if let Some(next_nodes) = adjacency.get(&current) {
            queue.extend(next_nodes.iter().copied());
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{analyze_graph_topology, connected_input_mask, would_create_cycle};

    #[test]
    fn connected_input_mask_marks_known_indexes_only() {
        assert_eq!(
            connected_input_mask(4, [0, 2, 5]),
            vec![true, false, true, false]
        );
    }

    #[test]
    fn topology_analysis_reports_blocked_nodes_for_cycles() {
        let analysis = analyze_graph_topology([1_u64, 2, 3], [(1, 2), (2, 3), (3, 2)]);
        assert_eq!(analysis.ordered_nodes, vec![1]);
        assert_eq!(analysis.blocked_nodes, vec![2, 3]);
    }

    #[test]
    fn cycle_detection_finds_reverse_path() {
        assert!(would_create_cycle([(1_u64, 2), (2, 3)], 3, 1));
        assert!(!would_create_cycle([(1_u64, 2), (2, 3)], 1, 3));
    }
}
