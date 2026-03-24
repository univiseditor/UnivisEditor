use crate::internal_prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::ui::UiTargetCamera;
use univis_ui::prelude::*;

use super::{pointer_target_node, selection_additive_modifier};

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BoxSelectionState {
    pub active: bool,
    pub additive: bool,
    pub start_screen_pos: Vec2,
    pub current_screen_pos: Vec2,
    pub suppress_click_selection: bool,
}

impl BoxSelectionState {
    pub fn clear(&mut self) {
        self.active = false;
        self.additive = false;
        self.start_screen_pos = Vec2::ZERO;
        self.current_screen_pos = Vec2::ZERO;
        self.suppress_click_selection = false;
    }

    pub(crate) fn min_screen_pos(&self) -> Vec2 {
        Vec2::new(
            self.start_screen_pos.x.min(self.current_screen_pos.x),
            self.start_screen_pos.y.min(self.current_screen_pos.y),
        )
    }

    pub(crate) fn max_screen_pos(&self) -> Vec2 {
        Vec2::new(
            self.start_screen_pos.x.max(self.current_screen_pos.x),
            self.start_screen_pos.y.max(self.current_screen_pos.y),
        )
    }

    pub(crate) fn drag_distance_sq(&self) -> f32 {
        self.start_screen_pos
            .distance_squared(self.current_screen_pos)
    }
}

#[derive(Component)]
pub struct BoxSelectionOverlay;

pub fn box_selection_input_system(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    overlay: Res<GraphOverlayState>,
    mut box_selection: ResMut<BoxSelectionState>,
    mut live_document: ResMut<LiveGraphDocumentState>,
    nodes_interaction: Query<(Entity, &UInteraction), With<GraphNode>>,
    headers_interaction: Query<(Entity, &UInteraction), With<Header>>,
    ports_interaction: Query<(&UInteraction, &GraphPort)>,
    parents: Query<&ChildOf>,
    node_markers: Query<(), With<GraphNode>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    q_node_transforms: Query<(Entity, &GlobalTransform), With<GraphNode>>,
    selected_nodes: Query<Entity, With<Selected>>,
) {
    if overlay.active_surface != GraphOverlaySurface::None && !box_selection.active {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let cursor_pos = window.cursor_position();

    if mouse_button.just_pressed(MouseButton::Left)
        && overlay.active_surface == GraphOverlaySurface::None
    {
        let clicked_node = pointer_target_node(
            &nodes_interaction,
            &headers_interaction,
            &ports_interaction,
            &parents,
            &node_markers,
        );

        if clicked_node.is_none() {
            let Some(cursor_pos) = cursor_pos else {
                return;
            };

            box_selection.active = true;
            box_selection.additive = selection_additive_modifier(&keys);
            box_selection.start_screen_pos = cursor_pos;
            box_selection.current_screen_pos = cursor_pos;
            box_selection.suppress_click_selection = true;

            if !box_selection.additive {
                live_document.clear_selected_entities();
                for entity in selected_nodes.iter() {
                    commands.entity(entity).try_remove::<Selected>();
                }
            }
        }
    }

    if !box_selection.active {
        return;
    }

    if let Some(cursor_pos) = cursor_pos {
        box_selection.current_screen_pos = cursor_pos;
    }

    if !mouse_button.just_released(MouseButton::Left) {
        return;
    }

    if box_selection.drag_distance_sq() >= 16.0 {
        let Ok((camera, camera_transform)) = q_camera.single() else {
            box_selection.clear();
            return;
        };

        let min = box_selection.min_screen_pos();
        let max = box_selection.max_screen_pos();
        let mut next_selection: HashSet<Entity> = if box_selection.additive {
            selected_nodes.iter().collect()
        } else {
            HashSet::default()
        };

        for (entity, global_transform) in q_node_transforms.iter() {
            let Ok(screen_pos) =
                camera.world_to_viewport(camera_transform, global_transform.translation())
            else {
                continue;
            };
            if screen_pos.x >= min.x
                && screen_pos.x <= max.x
                && screen_pos.y >= min.y
                && screen_pos.y <= max.y
            {
                next_selection.insert(entity);
            }
        }

        for entity in selected_nodes.iter() {
            if !next_selection.contains(&entity) {
                commands.entity(entity).try_remove::<Selected>();
            }
        }
        for entity in &next_selection {
            commands.entity(*entity).try_insert(Selected);
        }
        live_document.set_selected_entities(next_selection.iter().copied());
    }

    box_selection.active = false;
    box_selection.additive = false;
    box_selection.start_screen_pos = Vec2::ZERO;
    box_selection.current_screen_pos = Vec2::ZERO;
    box_selection.suppress_click_selection = true;
}

pub fn sync_box_selection_overlay(
    mut commands: Commands,
    box_selection: Res<BoxSelectionState>,
    q_graph_camera: Query<Entity, With<GraphCamera>>,
    mut overlays: Query<(Entity, &mut Node, Option<&UiTargetCamera>), With<BoxSelectionOverlay>>,
) {
    if overlays.is_empty() {
        let mut overlay_commands = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                border: UiRect::all(Val::Px(1.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.32, 0.64, 1.0, 0.12)),
            BorderColor::all(Color::srgba(0.55, 0.8, 1.0, 0.95)),
            ZIndex(1500),
            BoxSelectionOverlay,
        ));

        if let Some(graph_camera) = q_graph_camera.iter().next() {
            overlay_commands.insert(UiTargetCamera(graph_camera));
        }
    }

    let Some(graph_camera) = q_graph_camera.iter().next() else {
        return;
    };

    for (entity, mut node, target_camera) in overlays.iter_mut() {
        if target_camera.map(|target| target.entity()) != Some(graph_camera) {
            commands
                .entity(entity)
                .try_insert(UiTargetCamera(graph_camera));
        }

        if box_selection.active && box_selection.drag_distance_sq() >= 4.0 {
            let min = box_selection.min_screen_pos();
            let max = box_selection.max_screen_pos();
            node.display = Display::Flex;
            node.left = Val::Px(min.x);
            node.top = Val::Px(min.y);
            node.width = Val::Px((max.x - min.x).max(1.0));
            node.height = Val::Px((max.y - min.y).max(1.0));
        } else {
            node.display = Display::None;
        }
    }
}
