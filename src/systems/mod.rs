use bevy::prelude::*;
use crate::components::*;

pub fn add_people(mut commands: Commands) {
    commands.spawn((Person, crate::components::Name("Elaina Proctor".to_string())));
    commands.spawn((Person, crate::components::Name("Renzo Hume".to_string())));
    commands.spawn((Person, crate::components::Name("Zayna Nieves".to_string())));
}

pub fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&crate::components::Name, With<Person>>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

pub fn update_people(mut query: Query<&mut crate::components::Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don't need to change any other names.
        }
    }
}



#[test]
fn test_shortcut() {
    use bevy::ecs::system::RunSystemOnce; // Bring the trait into scope
    let mut app = App::new();

    // 1. Setup your test data directly in the world
    app.world_mut().spawn((Person, crate::components::Name("Elaina Proctor".to_string())));

    // 2. RUN THE FUNCTION DIRECTLY using the world
    // Bevy will automatically build the Query arguments for you behind the scenes!
    app.world_mut().run_system_once(update_people);

    // 3. Assert your results
    let mut query = app.world_mut().query::<&crate::components::Name>();
    assert_eq!(query.single(app.world()).unwrap().0, "Elaina Hume");
}