use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::{CompositeAlphaMode, CursorGrabMode, CursorOptions, PrimaryWindow, WindowLevel};
use bevy::winit::WINIT_WINDOWS;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin()))
        .insert_resource(ClearColor(Color::NONE))
        .init_resource::<PawAnimState>()
        .add_systems(Startup, (setup, setup_primary_window))
        .add_systems(Update, (follow_mouse, update_inner_arm, animate_paw))
        .run();
}

#[derive(Resource, Default)]
struct PawAnimState {
    factor: f32, // -1.0 (Clench) to 1.0 (Open), 0.0 (Neutral)
}

#[derive(Component)]
struct PawArm;

#[derive(Component)]
struct PawPalm;

#[derive(Component)]
struct PawBottom;

#[derive(Component)]
struct PawFinger {
    base_pos: Vec3,
    index: usize,
}

// Visual constants
const OUTLINE_WIDTH: f32 = 6.0;
const ARM_WIDTH: f32 = 40.0;
const PALM_RADIUS: f32 = 50.0;
const FINGER_RADIUS: f32 = 20.0;
// Colors
const COLOR_FILL: Color = Color::WHITE;
const COLOR_OUTLINE: Color = Color::BLACK;

fn window_plugin() -> WindowPlugin {
    let window = Window {
        title: "Cat Paw".into(),
        transparent: true,
        decorations: false,
        resizable: false,
        has_shadow: false,
        window_level: WindowLevel::AlwaysOnTop,
        #[cfg(target_os = "macos")]
        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
        #[cfg(target_os = "linux")]
        composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
        ..default()
    };

    let cursor_options = CursorOptions {
        visible: false,
        hit_test: false,
        ..default()
    };

    WindowPlugin {
        primary_window: Some(window),
        primary_cursor_options: Some(cursor_options),
        ..default()
    }
}

fn setup_primary_window(
    primary_window: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    _non_send_marker: NonSendMarker,
) {
    let (entity, mut window) = primary_window.into_inner();
    WINIT_WINDOWS.with_borrow(|winit_windows| {
        let Some(winit_window) = winit_windows.get_window(entity) else {
            error!("Primary window找不到: {:?}", entity);
            return;
        };
        let Some(current_monitor) = winit_window.current_monitor() else {
            error!("当前显示器找不到: {:?}", entity);
            return;
        };

        let monitor_pos = current_monitor.position();
        let monitor_size = current_monitor.size();
        let scale_factor = current_monitor.scale_factor() as f32;

        let window_width = monitor_size.width as f32 / scale_factor;
        let window_height = monitor_size.height as f32 / scale_factor;
        let window_left = monitor_pos.x;
        let window_top = monitor_pos.y;

        debug!(
            "当前屏幕: {:?}, 屏幕尺寸: {}x{}, 窗口新尺寸: {}x{}, 窗口新坐标: {}x{}",
            current_monitor.name(),
            monitor_size.width,
            monitor_size.height,
            window_width,
            window_height,
            window_left,
            window_top
        );

        window.resolution.set(window_width, window_height);
        window.position = WindowPosition::At(IVec2::new(window_left, window_top));
    });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let mesh_circle = meshes.add(Circle::new(1.0));
    let mesh_rect = meshes.add(Rectangle::new(1.0, 1.0));

    let mat_white = materials.add(ColorMaterial::from(COLOR_FILL));
    let mat_black = materials.add(ColorMaterial::from(COLOR_OUTLINE));

    // Spawn Arm
    commands
        .spawn((
            Mesh2d(mesh_rect.clone()),
            MeshMaterial2d(mat_black.clone()),
            Transform::default(),
            PawArm,
        ))
        .with_children(|parent| {
            // Inner white arm
            parent.spawn((
                Mesh2d(mesh_rect.clone()),
                MeshMaterial2d(mat_white.clone()),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

    // Spawn Arm Bottom (Semi-circle effect)
    commands
        .spawn((
            Mesh2d(mesh_circle.clone()),
            MeshMaterial2d(mat_black.clone()),
            Transform::from_scale(Vec3::splat((ARM_WIDTH + OUTLINE_WIDTH) / 2.0)),
            PawBottom,
        ))
        .with_children(|parent| {
            // Inner white bottom
            parent.spawn((
                Mesh2d(mesh_circle.clone()),
                MeshMaterial2d(mat_white.clone()),
                Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec3::splat(
                    (ARM_WIDTH - OUTLINE_WIDTH) / 2.0 / ((ARM_WIDTH + OUTLINE_WIDTH) / 2.0),
                )),
            ));
        });

    // Spawn Palm
    commands
        .spawn((
            Mesh2d(mesh_circle.clone()),
            MeshMaterial2d(mat_black.clone()),
            Transform::from_scale(Vec3::splat(PALM_RADIUS + OUTLINE_WIDTH)),
            PawPalm,
        ))
        .with_children(|parent| {
            // Inner white palm
            parent.spawn((
                Mesh2d(mesh_circle.clone()),
                MeshMaterial2d(mat_white.clone()),
                Transform::from_xyz(0.0, 0.0, 0.1)
                    .with_scale(Vec3::splat(PALM_RADIUS / (PALM_RADIUS + OUTLINE_WIDTH))),
            ));

            // Fingers
            let finger_offsets = [
                Vec3::new(-50.0, 50.0, 0.0),
                Vec3::new(-20.0, 65.0, 0.0),
                Vec3::new(20.0, 65.0, 0.0),
                Vec3::new(50.0, 50.0, 0.0),
            ];

            for (i, &pos) in finger_offsets.iter().enumerate() {
                parent
                    .spawn((
                        Mesh2d(mesh_circle.clone()),
                        MeshMaterial2d(mat_black.clone()),
                        Transform::from_translation(pos).with_scale(Vec3::splat(
                            (FINGER_RADIUS + OUTLINE_WIDTH) / (PALM_RADIUS + OUTLINE_WIDTH),
                        )),
                        PawFinger {
                            base_pos: pos,
                            index: i,
                        },
                    ))
                    .with_children(|finger_parent| {
                        // Inner white finger
                        finger_parent.spawn((
                            Mesh2d(mesh_circle.clone()),
                            MeshMaterial2d(mat_white.clone()),
                            Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec3::splat(
                                FINGER_RADIUS / (FINGER_RADIUS + OUTLINE_WIDTH),
                            )),
                        ));
                    });
            }
        });
}

fn follow_mouse(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut arm_query: Query<&mut Transform, (With<PawArm>, Without<PawPalm>, Without<PawBottom>)>,
    mut palm_query: Query<&mut Transform, (With<PawPalm>, Without<PawArm>, Without<PawBottom>)>,
    mut bottom_query: Query<&mut Transform, (With<PawBottom>, Without<PawArm>, Without<PawPalm>)>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(mouse_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let mouse_world_pos: Vec2 = mouse_world_pos;
            let start_pos = Vec2::new(0.0, -window.height() / 2.0);
            let diff = mouse_world_pos - start_pos;
            let length = diff.length();
            let angle = diff.y.atan2(diff.x) - std::f32::consts::FRAC_PI_2;

            for mut transform in palm_query.iter_mut() {
                transform.translation = mouse_world_pos.extend(2.0);
                transform.rotation = Quat::from_rotation_z(angle);
            }

            let midpoint = (start_pos + mouse_world_pos) / 2.0;

            for mut transform in arm_query.iter_mut() {
                transform.translation = midpoint.extend(1.0);
                transform.rotation = Quat::from_rotation_z(angle);
                transform.scale = Vec3::new(ARM_WIDTH + OUTLINE_WIDTH, length, 1.0);
            }

            for mut transform in bottom_query.iter_mut() {
                transform.translation = start_pos.extend(1.0);
            }
        }
    }
}

fn update_inner_arm(
    arm_query: Query<(&Transform, &Children), With<PawArm>>,
    mut inner_query: Query<&mut Transform, Without<PawArm>>,
) {
    for (parent_transform, children) in arm_query.iter() {
        let parent_scale = parent_transform.scale;
        let w_outer = parent_scale.x;
        let l_outer = parent_scale.y;

        if w_outer > 0.0 && l_outer > 0.0 {
            for child in children.iter() {
                if let Ok(mut child_transform) = inner_query.get_mut(child) {
                    let w_inner = (w_outer - 2.0 * OUTLINE_WIDTH).max(0.0);
                    let l_inner = l_outer.max(0.0);

                    child_transform.scale = Vec3::new(w_inner / w_outer, l_inner / l_outer, 1.0);
                }
            }
        }
    }
}

fn animate_paw(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut anim_state: ResMut<PawAnimState>,
    mut fingers: Query<(&mut Transform, &PawFinger)>,
    time: Res<Time>,
) {
    let mut target = 0.0f32;
    if mouse_button.pressed(MouseButton::Left) {
        target = -1.0;
    } else if mouse_button.pressed(MouseButton::Right) {
        target = 1.0;
    }

    // Interpolate
    let speed = 10.0;
    anim_state.factor += (target - anim_state.factor) * speed * time.delta_secs();

    for (mut transform, finger) in fingers.iter_mut() {
        let original_pos = finger.base_pos;

        // Base clench offset
        let clench_offset = original_pos * -0.3; // Move 30% inward

        // Base open offset
        let mut open_offset = original_pos * 0.2; // Move 20% outward
        open_offset.x *= 1.5; // Spread wider horizontally

        let current_offset = if anim_state.factor < 0.0 {
            clench_offset * -anim_state.factor
        } else {
            open_offset * anim_state.factor
        };

        transform.translation = original_pos + current_offset;
    }
}
