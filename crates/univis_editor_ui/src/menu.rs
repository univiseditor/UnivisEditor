//! نظام القائمة السياقية - النظام الجديد القابل للتوسع
//! يعمل مع NodeRegistry لعرض جميع العُقد المسجلة

use crate::prelude::*;
use bevy::prelude::*;
use univis_ui::prelude::*;

/// مورد لتخزين حالة القائمة
#[derive(Resource, Default)]
pub struct ContextMenuState {
    pub is_open: bool,
    pub position: Vec2, // موقع الفأرة في الشاشة (Screen Pixels)
    /// 🎯 جديد: نص البحث
    pub search_query: String,
}

/// مكون لتمييز واجهة المستخدم الخاصة بالقائمة
#[derive(Component)]
pub struct ContextMenuUI;

/// 🎯 جديد: مكون لحقل البحث
#[derive(Component)]
pub struct SearchInput;

/// مكون لتمييز عناصر القائمة حسب التصنيف
#[derive(Component)]
pub struct CategoryHeader {
    pub category: String,
}

/// نظام لفتح القائمة عند الضغط بالزر الأيمن في الفراغ
pub fn open_context_menu(
    mut _commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut menu_state: ResMut<ContextMenuState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    // نتحقق هل ضغطنا على منفذ؟ إذا نعم، لا تفتح القائمة (تجنب التضارب مع wire_start_system)
    ports: Query<&UInteraction, With<GraphPort>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        menu_state.is_open = false;
        return;
    }

    if mouse_button.just_pressed(MouseButton::Right) {
        // التأكد من أننا لم نضغط على منفذ (Port)
        let clicked_on_port = ports.iter().any(|interaction| {
            matches!(
                *interaction,
                UInteraction::Pressed | UInteraction::Hovered | UInteraction::Clicked
            )
        });
        if !clicked_on_port {
            if !menu_state.is_open {
                if let Ok(window) = windows.single() {
                    if let Some(pos) = window.cursor_position() {
                        menu_state.is_open = true;
                        menu_state.position = pos;
                        menu_state.search_query.clear(); // مسح البحث عند الفتح
                    }
                }
            } else {
                menu_state.is_open = false;
            }
        }
    }
}

/// نظام لرسم القائمة (يعمل فقط عندما تكون الحالة مفتوحة)
pub fn draw_context_menu(
    mut commands: Commands,
    menu_state: Res<ContextMenuState>,
    activation: Option<Res<GraphEditingUiActivation>>,
    existing_menu: Query<Entity, With<ContextMenuUI>>,
    registry: Res<NodeRegistry>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        for entity in existing_menu.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // إذا كانت القائمة مفتوحة ولكن عنصر واجهة المستخدم غير موجود، قم بإنشائه
    if menu_state.is_open {
        if existing_menu.is_empty() {
            // 🎯 محسّن: استخدام get_menu_nodes_sorted للحصول على العُقد المرتبة
            let menu_nodes = if menu_state.search_query.is_empty() {
                registry.get_menu_nodes_sorted()
            } else {
                registry.search(&menu_state.search_query)
            };

            // تجميع العُقد حسب التصنيف
            let mut categories: std::collections::HashMap<String, Vec<(NodeId, String, Color)>> =
                std::collections::HashMap::new();

            for definition in menu_nodes {
                let category = definition.category().as_str().to_string();
                categories.entry(category).or_insert_with(Vec::new).push((
                    definition.id(),
                    definition.display_name().to_string(),
                    definition.color(),
                ));
            }

            // حساب موقع القائمة (لتجنب الخروج من الشاشة)
            let menu_width = 220.0;
            let menu_height = 600.0_f32.min(600.0); // الحد الأقصى للارتفاع

            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(menu_state.position.x.min(1280.0 - menu_width)),
                        top: Val::Px(menu_state.position.y.min(720.0 - menu_height)),
                        width: Val::Px(menu_width),
                        max_height: Val::Px(menu_height),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(5.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        overflow: Overflow::scroll_y(), // 🎯 Scrolling!
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.8)),
                    // BorderWidth(Val::Px(1.0)),
                    ZIndex(100), // لضمان ظهورها فوق كل شيء
                    ContextMenuUI,
                ))
                .with_children(|parent| {
                    // 🎯 جديد: حقل البحث
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(32.0),
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                margin: UiRect::bottom(Val::Px(5.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
                        ))
                        .with_children(|search_container| {
                            search_container.spawn((
                                Text::new("🔍 Search..."),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                            ));
                        });

                    // عرض العُقد حسب التصنيف
                    let mut sorted_categories: Vec<_> = categories.iter().collect();
                    sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

                    for (category, nodes) in sorted_categories {
                        // عنوان التصنيف
                        parent
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(25.0),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                margin: UiRect::top(Val::Px(5.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
                        ))
                            .with_children(|header| {
                                header.spawn((
                                    Text::new(category.clone()),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                ));
                            });

                        // أزرار العُقد في هذا التصنيف
                        for (node_id, title, color) in nodes {
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(28.0),
                                        justify_content: JustifyContent::FlexStart,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::horizontal(Val::Px(10.0)),
                                        margin: UiRect::top(Val::Px(2.0)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(*color),
                                    NodeTypeButton {
                                        definition_id: node_id.clone(),
                                    },
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(title.clone()),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        }
                    }

                    // رسالة إذا لم توجد نتائج
                    if categories.is_empty() {
                        parent
                            .spawn((Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },))
                            .with_children(|empty| {
                                empty.spawn((
                                    Text::new("No nodes found"),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                ));
                            });
                    }
                });
        }
    }

    // إذا تم إغلاق القائمة (من نظام آخر)، احذف العنصر
    if !menu_state.is_open {
        for entity in existing_menu.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// مخصص للأزرار لتخزين معرف العقدة التي يجب إنشاؤها
#[derive(Component)]
pub struct NodeTypeButton {
    pub definition_id: NodeId,
}

/// نظام للتعامل مع الضغط على أزرار القائمة
pub fn interact_context_menu(
    mut commands: Commands,
    registry: Res<NodeRegistry>,
    activation: Option<Res<GraphEditingUiActivation>>,
    mut interaction_query: Query<
        (&Interaction, &NodeTypeButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu_state: ResMut<ContextMenuState>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
) {
    if !graph_editing_enabled(activation.as_deref()) {
        return;
    }

    for (interaction, button_data) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            // 1. تحويل موقع الفأرة في الشاشة إلى موقع في العالم (World Coordinates)
            let Ok(window) = windows.single() else { return };
            let Ok((camera, cam_transform)) = camera_query.single() else {
                return;
            };

            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
                    // 2. إنشاء العقدة باستخدام النظام الجديد
                    commands.spawn_node_from_definition(
                        &button_data.definition_id,
                        world_pos,
                        &registry,
                    );
                    // 3. إغلاق القائمة
                    menu_state.is_open = false;
                }
            }
        }
    }
}

fn graph_editing_enabled(activation: Option<&GraphEditingUiActivation>) -> bool {
    activation.map(|activation| activation.enabled).unwrap_or(true)
}
