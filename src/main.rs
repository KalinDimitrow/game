use bevy::prelude::*;
use game::components::*;
use game::systems::*;
use leafwing_input_manager::prelude::*;



fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .run();
}

fn hello_world() {
    println!("hello world!");
}

pub struct HelloPlugin;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // This will capture the total continuous value, for direct use.
    // Note that you can also use discrete gesture-like motion,
    // via the `MouseMoveDirection` enum.
    let input_map = InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default());
    commands.spawn(Camera2d).insert(input_map);

    // commands.spawn((
    //     Sprite::default(),
    //     Transform::from_scale(Vec3::new(100., 100., 1.)),
    // ));

    commands.spawn(
        (
        // #CircularBase
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d::<StandardMaterial>(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        )
    );
    commands.spawn(
        (
        // #Cube
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d::<StandardMaterial>(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0)
        )
    );
    commands.spawn(
    (
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0)
        )
    );
    commands.spawn(
        (
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default())
    )
    );
}

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        // app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        // app.add_systems(Startup, add_people);
        // app.add_systems(Update, hello_world);
        // app.add_systems(Update,  (update_people, greet_people).chain());
        app.add_systems(Update, (keyboard_input_system, pan_camera));
        app.add_systems(Startup, (scene.spawn(), setup));
    }
}




