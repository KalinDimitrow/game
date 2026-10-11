//! Dev tool: renders a few frames with no visible display and saves a PNG,
//! so a screenshot can be produced inside a container with no real X/Wayland
//! session. Needs a virtual framebuffer + software Vulkan driver:
//!
//!   Xvfb :99 -screen 0 800x600x24 &
//!   DISPLAY=:99 WGPU_BACKEND=vulkan cargo run --bin headless_screenshot
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{ExitCondition, WindowResolution};

#[derive(Resource, Default)]
struct ScreenshotState {
    requested: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            exit_condition: ExitCondition::DontExit,
            ..default()
        }))
        .insert_resource(ScreenshotState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, capture_then_exit)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Gates on wall-clock time, not frame count: under llvmpipe's software
/// rendering the ECS Update loop can tick far faster than the GPU actually
/// finishes a frame, so an early frame-count capture grabs an empty,
/// not-yet-rendered swapchain image.
fn capture_then_exit(
    time: Res<Time>,
    mut state: ResMut<ScreenshotState>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let elapsed = time.elapsed_secs();
    if !state.requested && elapsed > 2.0 {
        state.requested = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("headless_screenshot.png"));
    }
    // Screenshot capture is async (GPU readback); leave a wide margin after
    // the request before tearing the app down.
    if elapsed > 6.0 {
        exit.write(AppExit::Success);
    }
}
