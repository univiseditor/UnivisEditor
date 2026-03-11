//! سجل العُقد - لإدارة وتسجيل تعريفات العُقد
//! يوفر وصولاً سهلاً لجميع العُقد المسجلة

use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::node_definition::{ArcNodeDefinition, GraphNode, NodeDefinition, NodeId};

pub use inventory;

/// تسجيل تلقائي لعقدة.
pub struct NodeAutoRegistration {
    pub ctor: fn() -> ArcNodeDefinition,
}

inventory::collect!(NodeAutoRegistration);

/// سجل العُقد - Resource
#[derive(Resource, Default)]
pub struct NodeRegistry {
    /// تعريفات العُقد حسب المعرف
    definitions: HashMap<NodeId, ArcNodeDefinition>,
    /// العُقد حسب التصنيف
    by_category: HashMap<String, Vec<NodeId>>,
    /// ترتيب العُقد (للعرض)
    ordered_ids: Vec<NodeId>,
    /// 🎯 جديد: العُقد للقائمة فقط (show_in_menu = true)
    menu_nodes: Vec<NodeId>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            by_category: HashMap::new(),
            ordered_ids: Vec::new(),
            menu_nodes: Vec::new(),
        }
    }

    /// تسجيل عقدة جديدة
    pub fn register(&mut self, definition: impl NodeDefinition + 'static) {
        self.register_arc(Arc::new(definition));
    }

    /// تسجيل عقدة من Arc
    pub fn register_arc(&mut self, definition: ArcNodeDefinition) {
        let id = definition.id();
        if self.definitions.contains_key(&id) {
            return;
        }

        let category = definition.category().as_str().to_string();
        let show_in_menu = definition.show_in_menu();

        self.definitions.insert(id.clone(), definition);

        self.by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(id.clone());

        self.ordered_ids.push(id.clone());

        if show_in_menu {
            self.menu_nodes.push(id);
        }
    }

    /// الحصول على تعريف العقدة بالمعرف
    pub fn get(&self, id: &NodeId) -> Option<ArcNodeDefinition> {
        self.definitions.get(id).cloned()
    }

    /// التحقق من وجود عقدة
    pub fn contains(&self, id: &NodeId) -> bool {
        self.definitions.contains_key(id)
    }

    /// الحصول على جميع العُقد
    pub fn get_all(&self) -> impl Iterator<Item = &ArcNodeDefinition> {
        self.definitions.values()
    }

    /// الحصول على جميع معرفات العُقد
    pub fn get_all_ids(&self) -> &[NodeId] {
        &self.ordered_ids
    }

    /// 🎯 جديد: الحصول على العُقد للقائمة فقط
    pub fn get_menu_nodes(&self) -> Vec<ArcNodeDefinition> {
        self.menu_nodes
            .iter()
            .filter_map(|id| self.definitions.get(id).cloned())
            .collect()
    }

    /// 🎯 جديد: الحصول على العُقد للقائمة مرتبة حسب التصنيف والترتيب
    pub fn get_menu_nodes_sorted(&self) -> Vec<ArcNodeDefinition> {
        let mut nodes: Vec<_> = self
            .menu_nodes
            .iter()
            .filter_map(|id| self.definitions.get(id).cloned())
            .collect();

        // ترتيب حسب التصنيف ثم menu_order
        nodes.sort_by(|a, b| {
            let cat_cmp = a.category().as_str().cmp(b.category().as_str());
            if cat_cmp != std::cmp::Ordering::Equal {
                cat_cmp
            } else {
                a.menu_order().cmp(&b.menu_order())
            }
        });

        nodes
    }

    /// الحصول على العُقد حسب التصنيف
    pub fn get_by_category(&self, category: &str) -> Vec<ArcNodeDefinition> {
        self.by_category
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.definitions.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// الحصول على جميع التصنيفات
    pub fn get_categories(&self) -> impl Iterator<Item = &String> {
        self.by_category.keys()
    }

    /// عدد العُقد المسجلة
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// هل السجل فارغ؟
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// 🎯 محسّن: البحث في العُقد (يدعم keywords)
    pub fn search(&self, query: &str) -> Vec<ArcNodeDefinition> {
        let query_lower = query.to_lowercase();
        self.definitions
            .values()
            .filter(|def| {
                // البحث في الاسم
                def.display_name().to_lowercase().contains(&query_lower)
                    // البحث في المعرف
                    || def.id().as_str().to_lowercase().contains(&query_lower)
                    // البحث في الوصف
                    || def.description()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    // 🎯 جديد: البحث في الكلمات المفتاحية
                    || def.keywords().iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// مسح جميع العُقد
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.by_category.clear();
        self.ordered_ids.clear();
        self.menu_nodes.clear();
    }
}

/// Plugin لتسجيل السجل
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
    let changed_nodes: Vec<(Entity, ArcNodeDefinition)> = {
        let mut query = world
            .query_filtered::<(Entity, &GraphNode), Or<(Added<GraphNode>, Changed<GraphNode>)>>();
        let Some(registry) = world.get_resource::<NodeRegistry>() else {
            return;
        };
        query
            .iter(world)
            .filter_map(|(entity, node)| {
                registry
                    .get(&node.definition_id)
                    .filter(|definition| definition.needs_visual_sync())
                    .map(|definition| (entity, definition))
            })
            .collect()
    };

    for (entity, definition) in changed_nodes {
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
