//! Node registration and lookup infrastructure.
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use univis_graph_core::prelude::{
    ArcGraphNodeDefinition as ArcCoreNodeDefinition, GraphNodeRegistry, PortDefinition as CorePortDefinition,
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

        let core_definition: ArcCoreNodeDefinition<NodeValue, CorePortDefinition<NodeGraphSchema>> =
            Arc::new(OwnedBevyNodeDefinitionAdapter::new(definition.clone()));

        self.definitions.insert(id, definition);
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
        self.core_registry.get_all_ids()
    }

    pub fn get_menu_nodes(&self) -> Vec<ArcNodeDefinition> {
        self.lookup_definitions(
            self.core_registry
                .get_menu_nodes()
                .into_iter()
                .map(|definition| definition.id()),
        )
    }

    pub fn get_menu_nodes_sorted(&self) -> Vec<ArcNodeDefinition> {
        self.lookup_definitions(
            self.core_registry
                .get_menu_nodes_sorted()
                .into_iter()
                .map(|definition| definition.id()),
        )
    }

    pub fn get_by_category(&self, category: &str) -> Vec<ArcNodeDefinition> {
        self.lookup_definitions(
            self.core_registry
                .get_by_category(category)
                .into_iter()
                .map(|definition| definition.id()),
        )
    }

    pub fn get_categories(&self) -> impl Iterator<Item = &String> {
        self.core_registry.get_categories()
    }

    pub fn len(&self) -> usize {
        self.core_registry.len()
    }

    pub fn is_empty(&self) -> bool {
        self.core_registry.is_empty()
    }

    pub fn search(&self, query: &str) -> Vec<ArcNodeDefinition> {
        self.lookup_definitions(
            self.core_registry
                .search(query)
                .into_iter()
                .map(|definition| definition.id()),
        )
    }

    pub fn clear(&mut self) {
        self.definitions.clear();
        self.core_registry.clear();
    }

    fn lookup_definitions(
        &self,
        ids: impl IntoIterator<Item = NodeId>,
    ) -> Vec<ArcNodeDefinition> {
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
            .filter_map(|(entity, node)| registry.get(&node.definition_id).map(|definition| (entity, definition)))
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
