use std::collections::HashMap;
use std::sync::Arc;

use crate::identity::NodeId;
use crate::processing::GraphNodeDefinition;

pub type ArcGraphNodeDefinition<Value, Port> = Arc<dyn GraphNodeDefinition<Value, Port>>;

/// Engine-independent registry for pure graph node definitions.
pub struct GraphNodeRegistry<Value, Port> {
    definitions: HashMap<NodeId, ArcGraphNodeDefinition<Value, Port>>,
    by_category: HashMap<String, Vec<NodeId>>,
    ordered_ids: Vec<NodeId>,
    menu_nodes: Vec<NodeId>,
}

impl<Value, Port> Default for GraphNodeRegistry<Value, Port> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Value, Port> GraphNodeRegistry<Value, Port> {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            by_category: HashMap::new(),
            ordered_ids: Vec::new(),
            menu_nodes: Vec::new(),
        }
    }

    pub fn register(&mut self, definition: impl GraphNodeDefinition<Value, Port> + 'static) {
        self.register_arc(Arc::new(definition));
    }

    pub fn register_arc(&mut self, definition: ArcGraphNodeDefinition<Value, Port>) {
        let id = definition.id();
        if self.definitions.contains_key(&id) {
            return;
        }

        let category = definition.category().as_str().to_string();
        let show_in_menu = definition.show_in_menu();

        self.definitions.insert(id.clone(), definition);
        self.by_category
            .entry(category)
            .or_default()
            .push(id.clone());
        self.ordered_ids.push(id.clone());

        if show_in_menu {
            self.menu_nodes.push(id);
        }
    }

    pub fn get(&self, id: &NodeId) -> Option<ArcGraphNodeDefinition<Value, Port>> {
        self.definitions.get(id).cloned()
    }

    pub fn contains(&self, id: &NodeId) -> bool {
        self.definitions.contains_key(id)
    }

    pub fn get_all(&self) -> impl Iterator<Item = &ArcGraphNodeDefinition<Value, Port>> {
        self.definitions.values()
    }

    pub fn get_all_ids(&self) -> &[NodeId] {
        &self.ordered_ids
    }

    pub fn get_menu_nodes(&self) -> Vec<ArcGraphNodeDefinition<Value, Port>> {
        self.menu_nodes
            .iter()
            .filter_map(|id| self.definitions.get(id).cloned())
            .collect()
    }

    pub fn get_menu_nodes_sorted(&self) -> Vec<ArcGraphNodeDefinition<Value, Port>> {
        let mut nodes = self.get_menu_nodes();
        nodes.sort_by(|a, b| {
            let category_cmp = a.category().as_str().cmp(b.category().as_str());
            if category_cmp != std::cmp::Ordering::Equal {
                category_cmp
            } else {
                a.menu_order().cmp(&b.menu_order())
            }
        });
        nodes
    }

    pub fn get_by_category(&self, category: &str) -> Vec<ArcGraphNodeDefinition<Value, Port>> {
        self.by_category
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.definitions.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_categories(&self) -> impl Iterator<Item = &String> {
        self.by_category.keys()
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn search(&self, query: &str) -> Vec<ArcGraphNodeDefinition<Value, Port>> {
        let query_lower = query.to_lowercase();
        self.definitions
            .values()
            .filter(|definition| {
                definition
                    .display_name()
                    .to_lowercase()
                    .contains(&query_lower)
                    || definition
                        .id()
                        .as_str()
                        .to_lowercase()
                        .contains(&query_lower)
                    || definition
                        .description()
                        .map(|description| description.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || definition
                        .keywords()
                        .iter()
                        .any(|keyword| keyword.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.definitions.clear();
        self.by_category.clear();
        self.ordered_ids.clear();
        self.menu_nodes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::GraphNodeRegistry;
    use crate::identity::{NodeCategory, NodeId};
    use crate::processing::{GraphNodeDefinition, ProcessContext, ProcessResult};

    struct DummyNode {
        id: NodeId,
        name: &'static str,
        category: &'static str,
        show_in_menu: bool,
        keywords: Vec<&'static str>,
    }

    impl GraphNodeDefinition<(), ()> for DummyNode {
        fn id(&self) -> NodeId {
            self.id.clone()
        }

        fn display_name(&self) -> &str {
            self.name
        }

        fn category(&self) -> NodeCategory {
            NodeCategory::new(self.category)
        }

        fn inputs(&self) -> Vec<()> {
            vec![]
        }

        fn outputs(&self) -> Vec<()> {
            vec![]
        }

        fn process(&self, _context: &mut ProcessContext<'_, ()>) -> ProcessResult {
            ProcessResult::Success
        }

        fn show_in_menu(&self) -> bool {
            self.show_in_menu
        }

        fn keywords(&self) -> Vec<&str> {
            self.keywords.clone()
        }
    }

    #[test]
    fn registry_orders_and_searches_registered_nodes() {
        let mut registry = GraphNodeRegistry::<(), ()>::new();
        registry.register(DummyNode {
            id: NodeId::new("tests/alpha"),
            name: "Alpha",
            category: "Tests",
            show_in_menu: true,
            keywords: vec!["number"],
        });
        registry.register(DummyNode {
            id: NodeId::new("tests/beta"),
            name: "Beta",
            category: "Hidden",
            show_in_menu: false,
            keywords: vec!["logic"],
        });

        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get_all_ids().len(), 2);
        assert_eq!(registry.get_menu_nodes().len(), 1);
        assert_eq!(registry.search("logic").len(), 1);
        assert!(registry.contains(&NodeId::new("tests/alpha")));
    }
}
