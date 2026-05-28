use bevy::prelude::*;
use broken_divinity::core::resources::WorldSeed;
use broken_divinity::core::state::AppState;
use broken_divinity::core::turn::GameTime;
use broken_divinity::gamelog::GameLog;
use broken_divinity::ui::menu::{MenuUiAction, MenuUiChoice, process_menu_action};

const TEST_SEED: u64 = 424_242;
const TEST_TURN: u32 = 17;

#[test]
fn new_game_sets_seed_and_queues_colony_transition() {
    let mut app = App::new();
    app.insert_resource(GameLog::default());
    app.insert_resource(GameTime { turn: TEST_TURN });
    app.insert_resource(MenuUiAction(Some(MenuUiChoice::NewGame {
        seed: TEST_SEED,
    })));
    app.insert_resource(NextState::<AppState>::Unchanged);
    app.add_message::<AppExit>();
    app.add_systems(Update, process_menu_action);

    app.update();

    assert_eq!(app.world().resource::<WorldSeed>().0, TEST_SEED);
    assert!(matches!(
        *app.world().resource::<NextState<AppState>>(),
        NextState::Pending(AppState::Colony)
    ));

    let log = app.world().resource::<GameLog>();
    assert!(
        log.entries().iter().any(|entry| {
            let text = entry.text.to_lowercase();
            text.contains("seed") && text.contains("424242")
        }),
        "expected new game processing to emit exact seed feedback"
    );
}

#[test]
fn cancel_quit_preserves_menu_state_and_emits_no_exit() {
    let mut app = App::new();
    app.insert_resource(GameLog::default());
    app.insert_resource(GameTime { turn: TEST_TURN });
    app.insert_resource(MenuUiAction(Some(MenuUiChoice::CancelQuit)));
    app.insert_resource(NextState::<AppState>::Unchanged);
    app.add_message::<AppExit>();
    app.add_systems(Update, process_menu_action);

    app.update();

    assert!(matches!(
        *app.world().resource::<NextState<AppState>>(),
        NextState::Unchanged
    ));
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
}

#[test]
fn confirm_quit_emits_exit_message() {
    let mut app = App::new();
    app.insert_resource(GameLog::default());
    app.insert_resource(GameTime { turn: TEST_TURN });
    app.insert_resource(MenuUiAction(Some(MenuUiChoice::ConfirmQuit)));
    app.insert_resource(NextState::<AppState>::Unchanged);
    app.add_message::<AppExit>();
    app.add_systems(Update, process_menu_action);

    app.update();

    assert!(
        !app.world().resource::<Messages<AppExit>>().is_empty(),
        "expected confirm quit to emit an AppExit message"
    );
}
