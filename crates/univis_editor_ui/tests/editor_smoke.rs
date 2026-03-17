use bevy::camera::{CameraProjection, OrthographicProjection, Projection, RenderTargetInfo};
use bevy::prelude::*;
use univis_editor_commands::{GraphCommandRequest, GraphCommandsPlugin};
use univis_editor_ui::editor::GraphCamera;
use univis_editor_ui::interaction::{
    BoxSelectionState, box_selection_input_system, frame_selected_nodes_system, selection_system,
};
use univis_editor_ui::overlay::{GraphOverlayState, GraphOverlaySurface};
use univis_node_graph::document::{GraphDocumentNode, LiveGraphDocumentState};
use univis_node_graph::live_graph::{GraphNode, Selected};
use univis_node_graph::node_definition::NodeId;
use univis_ui::prelude::UInteraction;

fn node(id: u64, definition_id: &str, position: [f32; 2]) -> GraphDocumentNode {
    GraphDocumentNode {
        id,
        definition_id: NodeId::new(definition_id),
        position,
        inputs: Vec::new(),
        input_count: 0,
        output_count: 1,
    }
}

fn spawn_window(world: &mut World, width: f32, height: f32) {
    let mut window = Window::default();
    window
        .resolution
        .set_physical_resolution(width as u32, height as u32);
    world.spawn(window);
}

fn spawn_box_select_camera(world: &mut World) {
    let mut camera = Camera::default();
    let mut projection = OrthographicProjection::default_2d();
    projection.area = Rect::new(-400.0, -300.0, 400.0, 300.0);
    camera.computed.clip_from_view = projection.get_clip_from_view();
    camera.computed.target_info = Some(RenderTargetInfo {
        physical_size: UVec2::new(800, 600),
        scale_factor: 1.0,
    });

    world.spawn((
        GraphCamera,
        camera,
        Transform::default(),
        GlobalTransform::default(),
    ));
}

fn spawn_frame_camera(world: &mut World) {
    world.spawn((
        GraphCamera,
        Transform::default(),
        Projection::Orthographic(OrthographicProjection::default_2d()),
    ));
}

fn register_live_node(
    world: &mut World,
    live_document: &mut LiveGraphDocumentState,
    node_id: u64,
    position: Vec3,
) -> Entity {
    let entity = world
        .spawn((
            GraphNode::new(NodeId::new(format!("tests/node_{node_id}")), 0, 1),
            GlobalTransform::from_translation(position),
        ))
        .id();
    live_document.entity_to_node_id.insert(entity, node_id);
    live_document.node_id_to_entity.insert(node_id, entity);
    live_document
        .document
        .nodes
        .push(node(node_id, "tests/node", [position.x, position.y]));
    entity
}

#[test]
fn smoke_box_select_marks_nodes_inside_drag_rect() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<GraphOverlayState>()
        .init_resource::<BoxSelectionState>()
        .insert_resource(LiveGraphDocumentState::default())
        .add_systems(Update, box_selection_input_system);

    spawn_window(app.world_mut(), 800.0, 600.0);
    spawn_box_select_camera(app.world_mut());

    app.world_mut()
        .resource_scope(|world, mut live_document: Mut<LiveGraphDocumentState>| {
            register_live_node(world, &mut live_document, 1, Vec3::new(-120.0, 60.0, 0.0));
            register_live_node(world, &mut live_document, 2, Vec3::new(140.0, -80.0, 0.0));
            register_live_node(world, &mut live_document, 3, Vec3::new(320.0, 220.0, 0.0));
        });

    {
        let world = app.world_mut();
        let mut windows = world.query::<&mut Window>();
        let mut window = windows.single_mut(world).expect("single window");
        window.set_cursor_position(Some(Vec2::new(250.0, 220.0)));
    }
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    {
        let world = app.world_mut();
        let mut windows = world.query::<&mut Window>();
        let mut window = windows.single_mut(world).expect("single window");
        window.set_cursor_position(Some(Vec2::new(580.0, 400.0)));
    }
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();

    let mut selected_ids = app
        .world()
        .resource::<LiveGraphDocumentState>()
        .document
        .selected_node_ids()
        .to_vec();
    selected_ids.sort_unstable();
    assert_eq!(selected_ids, vec![1, 2]);
}

#[test]
fn smoke_frame_selected_moves_camera_to_selection_bounds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GraphCommandsPlugin)
        .insert_resource(LiveGraphDocumentState::default())
        .add_systems(Update, frame_selected_nodes_system);

    spawn_window(app.world_mut(), 1280.0, 720.0);
    spawn_frame_camera(app.world_mut());

    {
        let mut live_document = app.world_mut().resource_mut::<LiveGraphDocumentState>();
        live_document.document.nodes = vec![
            node(1, "tests/a", [100.0, 100.0]),
            node(2, "tests/b", [500.0, 300.0]),
        ];
        live_document.document.set_selected_nodes([1, 2]);
    }

    app.world_mut()
        .write_message(GraphCommandRequest::FrameSelectedNodes)
        .expect("frame request should enqueue");
    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<(&Transform, &Projection), With<GraphCamera>>();
    let (transform, projection) = query.single(world).expect("single graph camera");
    assert!((transform.translation.x - 300.0).abs() < 0.01);
    assert!((transform.translation.y - 200.0).abs() < 0.01);

    let Projection::Orthographic(ortho) = projection else {
        panic!("expected orthographic graph camera");
    };
    assert!(ortho.scale > 1.0);
}

#[test]
fn smoke_selection_is_preserved_while_overlay_surface_is_active() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<GraphOverlayState>()
        .init_resource::<BoxSelectionState>()
        .insert_resource(LiveGraphDocumentState::default())
        .add_systems(Update, selection_system);

    let selected_entity = app
        .world_mut()
        .spawn((
            GraphNode::new(NodeId::new("tests/selected"), 0, 1),
            UInteraction::default(),
            Selected,
        ))
        .id();

    {
        let mut live_document = app.world_mut().resource_mut::<LiveGraphDocumentState>();
        live_document.entity_to_node_id.insert(selected_entity, 1);
        live_document.node_id_to_entity.insert(1, selected_entity);
        live_document
            .document
            .nodes
            .push(node(1, "tests/selected", [0.0, 0.0]));
        live_document.document.set_selected_nodes([1]);
    }

    app.world_mut()
        .resource_mut::<GraphOverlayState>()
        .active_surface = GraphOverlaySurface::ContextMenu;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);

    app.update();

    let world = app.world_mut();
    assert!(world.entity(selected_entity).contains::<Selected>());
    assert_eq!(
        world
            .resource::<LiveGraphDocumentState>()
            .document
            .selected_node_ids(),
        &[1]
    );
}
