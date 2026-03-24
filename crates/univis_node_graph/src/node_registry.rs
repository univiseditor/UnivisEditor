//! Node registration and lookup infrastructure.
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use univis_graph_core::prelude::{
    ArcGraphNodeDefinition as ArcCoreNodeDefinition, GraphNodeRegistry,
    PortDefinition as CorePortDefinition,
};

use super::live_graph::GraphNode;
use super::node_definition::{
    ArcNodeDefinition, NodeDefinition, NodeGraphSchema, NodeId, OwnedBevyNodeDefinitionAdapter,
};
use super::value::NodeValue;

pub use inventory;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct VisualSyncNode;

/// Inventory entry used for automatic node registration.
pub struct NodeAutoRegistration {
    pub ctor: fn() -> ArcNodeDefinition,
}

inventory::collect!(NodeAutoRegistration);

/// Bevy-facing registry of all adapter node definitions known to the app.
#[derive(Resource)]
pub struct NodeRegistry {
    definitions: HashMap<NodeId, ArcNodeDefinition>,
    by_category: HashMap<String, Vec<NodeId>>,
    ordered_ids: Vec<NodeId>,
    menu_nodes: Vec<NodeId>,
    core_registry: GraphNodeRegistry<NodeValue, CorePortDefinition<NodeGraphSchema>>,
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            by_category: HashMap::new(),
            ordered_ids: Vec::new(),
            menu_nodes: Vec::new(),
            core_registry: GraphNodeRegistry::new(),
        }
    }

    pub fn register(&mut self, definition: impl NodeDefinition + 'static) {
        self.register_arc(Arc::new(definition));
    }

    pub fn register_arc(&mut self, definition: ArcNodeDefinition) {
        let id = definition.id();
        if self.definitions.contains_key(&id) {
            return;
        }

        let category = definition.category().as_str().to_string();
        let show_in_menu = definition.show_in_menu();

        let core_definition: ArcCoreNodeDefinition<NodeValue, CorePortDefinition<NodeGraphSchema>> =
            Arc::new(OwnedBevyNodeDefinitionAdapter::new(definition.clone()));

        self.definitions.insert(id.clone(), definition);
        self.by_category
            .entry(category)
            .or_default()
            .push(id.clone());
        self.ordered_ids.push(id.clone());
        if show_in_menu {
            self.menu_nodes.push(id.clone());
        }
        self.core_registry.register_arc(core_definition);
    }

    pub fn get(&self, id: &NodeId) -> Option<ArcNodeDefinition> {
        self.definitions.get(id).cloned()
    }

    pub fn contains(&self, id: &NodeId) -> bool {
        self.definitions.contains_key(id)
    }

    pub fn get_all(&self) -> impl Iterator<Item = &ArcNodeDefinition> {
        self.definitions.values()
    }

    pub fn get_all_ids(&self) -> &[NodeId] {
        &self.ordered_ids
    }

    pub fn get_menu_nodes(&self) -> Vec<ArcNodeDefinition> {
        self.lookup_definitions(self.menu_nodes.iter().cloned())
    }

    pub fn get_menu_nodes_sorted(&self) -> Vec<ArcNodeDefinition> {
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

    pub fn get_by_category(&self, category: &str) -> Vec<ArcNodeDefinition> {
        self.lookup_definitions(
            self.by_category
                .get(category)
                .into_iter()
                .flatten()
                .cloned(),
        )
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

    pub fn search(&self, query: &str) -> Vec<ArcNodeDefinition> {
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
        self.core_registry.clear();
    }

    pub fn core_registry(
        &self,
    ) -> &GraphNodeRegistry<NodeValue, CorePortDefinition<NodeGraphSchema>> {
        &self.core_registry
    }

    fn lookup_definitions(&self, ids: impl IntoIterator<Item = NodeId>) -> Vec<ArcNodeDefinition> {
        ids.into_iter()
            .filter_map(|id| self.definitions.get(&id).cloned())
            .collect()
    }
}

/// Plugin that initializes the node registry and sync hooks.
pub struct NodeRegistryPlugin;

impl Plugin for NodeRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodeRegistry>()
            .add_systems(Startup, auto_register_nodes)
            .add_systems(PostUpdate, sync_changed_node_visuals);
    }
}

fn auto_register_nodes(mut registry: ResMut<NodeRegistry>) {
    for registration in inventory::iter::<NodeAutoRegistration> {
        let definition = (registration.ctor)();
        registry.register_arc(definition);
    }
}

fn sync_changed_node_visuals(world: &mut World) {
    let visual_nodes: Vec<(Entity, ArcNodeDefinition)> = {
        let mut query = world.query_filtered::<(Entity, &GraphNode), With<VisualSyncNode>>();
        let Some(registry) = world.get_resource::<NodeRegistry>() else {
            return;
        };
        query
            .iter(world)
            .filter_map(|(entity, node)| {
                registry
                    .get(&node.definition_id)
                    .map(|definition| (entity, definition))
            })
            .collect()
    };

    // Custom node bodies can change through widget state without mutating GraphNode directly,
    // so visual sync needs to poll those nodes instead of relying on Changed<GraphNode>.
    for (entity, definition) in visual_nodes {
        definition.sync_visual(world, entity);
    }
}

#[macro_export]
macro_rules! register_node {
    ($node:path $(,)?) => {
        $crate::register_node!($node, ctor = || $node);
    };
    ($node:path, ctor = $ctor:expr $(,)?) => {
        $crate::node_registry::inventory::submit! {
            $crate::node_registry::NodeAutoRegistration {
                ctor: || -> $crate::node_definition::ArcNodeDefinition {
                    let node = ($ctor)();
                    std::sync::Arc::new(node)
                },
            }
        }
    };
}
