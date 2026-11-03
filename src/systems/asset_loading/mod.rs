pub mod gate;
pub mod rewind;
pub mod stage;

pub use self::gate::*;
pub use self::rewind::*;
pub use self::stage::*;

use crate::prelude::*;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoadSet {
    Poll,
    Watch,
    Settle,
    Advance,
}

pub fn settling(stage: LoadStage) -> impl Fn(Res<State<LoadStage>>, Res<StageGate>) -> bool + Clone {
    move |current, gate| *current.get() == stage && gate.is_ready()
}

fn load_boot_assets(asset_server: Res<AssetServer>, mut gate: ResMut<StageGate>) {
    gate.track(asset_server.load::<ButtonConfig>(BUTTON_CONFIG_ASSET_PATH));
}

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        build_machine(app);
        app.add_systems(OnEnter(LoadStage::Boot), load_boot_assets);
    }
}

fn build_machine(app: &mut App) {
    app.init_state::<LoadStage>();
    app.init_resource::<StageGate>();
    app.init_resource::<RewindRequest>();

    app.configure_sets(
        Update,
        (
            LoadSet::Poll,
            LoadSet::Watch,
            LoadSet::Settle,
            LoadSet::Advance,
        )
            .chain(),
    );

    app.add_systems(
        Update,
        (poll_gate, log_stage_transitions).in_set(LoadSet::Poll),
    );
    app.add_systems(
        Update,
        (advance_stage, apply_rewind).chain().in_set(LoadSet::Advance),
    );

    for stage in LoadStage::ORDER {
        app.add_systems(OnExit(*stage), clear_gate);
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPlugin;
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(AssetPlugin::default());
        build_machine(&mut app);
        app
    }

    fn stage(app: &App) -> LoadStage {
        *app.world().resource::<State<LoadStage>>().get()
    }

    fn run_to(app: &mut App, target: LoadStage) {
        for _ in 0..LoadStage::ORDER.len() * 3 {
            if stage(app) == target {
                return;
            }
            app.update();
        }
        panic!("machine never reached {target:?}, stuck at {:?}", stage(app));
    }

    fn rewind_to(app: &mut App, target: LoadStage) {
        app.world_mut().resource_mut::<RewindRequest>().to(target);
        app.update();
        app.update();
        assert_eq!(stage(app), target, "rewind to {target:?} did not land");
    }

    #[test]
    fn empty_stages_advance_one_per_frame_and_stop_at_settled() {
        let mut app = test_app();

        assert_eq!(stage(&app), LoadStage::Boot);

        app.update();
        assert_eq!(stage(&app), LoadStage::Boot);

        for expected in LoadStage::ORDER.iter().skip(1) {
            app.update();
            assert_eq!(stage(&app), *expected);
        }

        app.update();
        app.update();
        assert_eq!(stage(&app), LoadStage::Settled);
    }

    #[derive(Resource, Default)]
    struct SettleCalls(u32);

    #[test]
    fn a_blocker_holds_its_stage_open_until_released() {
        let mut app = test_app();
        app.init_resource::<SettleCalls>();
        app.add_systems(OnEnter(LoadStage::Derive), |mut gate: ResMut<StageGate>| {
            gate.block("test-blocker");
        });
        app.add_systems(
            Update,
            (|mut gate: ResMut<StageGate>, mut calls: ResMut<SettleCalls>| {
                calls.0 += 1;
                if calls.0 >= 3 {
                    gate.release("test-blocker");
                }
            })
            .in_set(LoadSet::Settle)
            .run_if(settling(LoadStage::Derive)),
        );

        run_to(&mut app, LoadStage::Derive);
        assert_eq!(app.world().resource::<SettleCalls>().0, 1);
        assert_eq!(
            app.world().resource::<StageGate>().waiting_on(),
            vec!["test-blocker"]
        );

        app.update();
        assert_eq!(app.world().resource::<SettleCalls>().0, 2);
        assert_eq!(stage(&app), LoadStage::Derive);

        app.update();
        assert_eq!(app.world().resource::<SettleCalls>().0, 3);
        assert_eq!(stage(&app), LoadStage::Derive);

        app.update();
        assert_eq!(stage(&app), LoadStage::World);
    }

    #[test]
    fn an_unloaded_handle_holds_its_stage_open_and_keeps_settle_systems_out() {
        let mut app = test_app();
        app.init_asset::<Image>();
        app.init_resource::<SettleCalls>();
        app.add_systems(OnEnter(LoadStage::Derive), |mut gate: ResMut<StageGate>| {
            gate.track(Handle::<Image>::default());
        });
        app.add_systems(
            Update,
            (|mut calls: ResMut<SettleCalls>| calls.0 += 1)
                .in_set(LoadSet::Settle)
                .run_if(settling(LoadStage::Derive)),
        );

        run_to(&mut app, LoadStage::Derive);
        app.update();
        app.update();
        assert_eq!(stage(&app), LoadStage::Derive, "unloaded handle should hold");
        assert!(!app.world().resource::<StageGate>().is_ready());
        assert_eq!(
            app.world().resource::<SettleCalls>().0,
            0,
            "a settle system must not run before its stage's handles are loaded"
        );
    }

    #[test]
    fn the_gate_is_emptied_on_leaving_a_stage() {
        let mut app = test_app();
        app.add_systems(OnEnter(LoadStage::Boot), |mut gate: ResMut<StageGate>| {
            gate.block("boot-blocker");
        });
        app.add_systems(
            Update,
            (|mut gate: ResMut<StageGate>| gate.release("boot-blocker"))
                .in_set(LoadSet::Settle)
                .run_if(settling(LoadStage::Boot)),
        );

        run_to(&mut app, LoadStage::Settled);
        assert!(
            app.world().resource::<StageGate>().waiting_on().is_empty(),
            "`clear_gate` should leave nothing behind for the next stage"
        );
    }

    #[test]
    fn rewind_coalesces_to_the_earliest_request() {
        let mut request = RewindRequest::default();
        request.to(LoadStage::Instantiate);
        request.to(LoadStage::Derive);
        request.to(LoadStage::World);
        assert_eq!(request.requested(), Some(LoadStage::Derive));
    }

    #[test]
    fn rewind_moves_the_machine_backwards() {
        let mut app = test_app();
        run_to(&mut app, LoadStage::Settled);

        rewind_to(&mut app, LoadStage::Derive);

        run_to(&mut app, LoadStage::Settled);
    }

    #[test]
    fn a_modified_asset_rewinds_to_its_consuming_stage() {
        let mut app = test_app();
        app.init_asset::<Image>();
        app.add_systems(
            Update,
            rewind_on_change::<Image>(LoadStage::Derive).in_set(LoadSet::Watch),
        );

        run_to(&mut app, LoadStage::Settled);

        app.world_mut().write_message(AssetEvent::<Image>::Added {
            id: AssetId::default(),
        });
        app.update();
        app.update();
        assert_eq!(stage(&app), LoadStage::Settled, "`Added` should not rewind");

        app.world_mut().write_message(AssetEvent::<Image>::Modified {
            id: AssetId::default(),
        });
        app.update();
        app.update();
        assert_eq!(stage(&app), LoadStage::Derive);
    }

    #[test]
    fn a_rewind_to_a_later_stage_is_ignored() {
        let mut app = test_app();
        app.add_systems(OnEnter(LoadStage::Derive), |mut gate: ResMut<StageGate>| {
            gate.block("hold-at-derive");
        });

        run_to(&mut app, LoadStage::Derive);

        app.world_mut()
            .resource_mut::<RewindRequest>()
            .to(LoadStage::Instantiate);
        app.update();
        app.update();
        assert_eq!(
            stage(&app),
            LoadStage::Derive,
            "a change a later stage consumes needs no transition - the machine \
             will read the reloaded asset when it gets there"
        );
    }

    #[derive(Component)]
    struct Instantiated;

    fn spawn_instantiated(mut commands: Commands) {
        commands.spawn(Instantiated);
    }

    fn instantiated_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<(), With<Instantiated>>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn a_spawning_stage_does_not_double_spawn_across_a_replay() {
        let mut app = test_app();
        app.add_systems(
            OnEnter(LoadStage::Instantiate),
            (despawn_on_replay::<Instantiated>, spawn_instantiated).chain(),
        );

        run_to(&mut app, LoadStage::Settled);
        assert_eq!(instantiated_count(&mut app), 1);

        rewind_to(&mut app, LoadStage::Payload);
        run_to(&mut app, LoadStage::Settled);
        assert_eq!(
            instantiated_count(&mut app),
            1,
            "replaying `Instantiate` must not leave the previous pass's entities behind"
        );
    }

    #[test]
    fn without_a_replay_guard_a_spawning_stage_double_spawns() {
        let mut app = test_app();
        app.add_systems(OnEnter(LoadStage::Instantiate), spawn_instantiated);

        run_to(&mut app, LoadStage::Settled);
        rewind_to(&mut app, LoadStage::Payload);
        run_to(&mut app, LoadStage::Settled);

        assert_eq!(instantiated_count(&mut app), 2);
    }
}
