//! Debug command enum and string parser.

use bd_core::signals::PoolKind;

/// Parsed debug command ready for dispatch.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugCommand {
    AddResource(PoolKind, i32),
    SetDay(u64),
    SetTurn(u64),
    SkipDay,
    TriggerEvent(String),
    EndEvent,
    KillAllEnemies,
    Heal,
    GodMode(bool),
    SpawnSurvivor(String),
    AssignTask(usize, String),
    SpawnEntity(String, i32, i32),
    Teleport(i32, i32),
    GotoShelter,
    ListBlueprints,
    ListEvents,
    Stats,
    Help,
    Clear,
    Unknown(String),
}

/// Parse a raw console input string into a [`DebugCommand`].
pub fn parse(input: &str) -> DebugCommand {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return DebugCommand::Unknown("empty command".into());
    }
    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    match cmd {
        "s" | "supplies" => parse_add_resource(PoolKind::Supplies, &args),
        "m" | "materials" => parse_add_resource(PoolKind::Materials, &args),
        "f" | "faith" => parse_add_resource(PoolKind::Faith, &args),
        "p" | "plants" => parse_add_resource(PoolKind::WildPlants, &args),
        "day" => parse_u64(&args)
            .map(DebugCommand::SetDay)
            .unwrap_or_else(DebugCommand::Unknown),
        "turn" => parse_u64(&args)
            .map(DebugCommand::SetTurn)
            .unwrap_or_else(DebugCommand::Unknown),
        "skip_day" => DebugCommand::SkipDay,
        "event" => {
            if args.is_empty() {
                DebugCommand::Unknown("usage: event <event_id>".into())
            } else {
                DebugCommand::TriggerEvent(args[0].to_string())
            }
        }
        "end_event" => DebugCommand::EndEvent,
        "kill_all" => DebugCommand::KillAllEnemies,
        "heal" => DebugCommand::Heal,
        "god" => match args.first().copied() {
            Some("on") => DebugCommand::GodMode(true),
            Some("off") => DebugCommand::GodMode(false),
            _ => DebugCommand::Unknown("usage: god on|off".into()),
        },
        "survivor" => {
            if args.is_empty() {
                DebugCommand::Unknown("usage: survivor <name>".into())
            } else {
                DebugCommand::SpawnSurvivor(args.join(" "))
            }
        }
        "task" => {
            if args.len() < 2 {
                DebugCommand::Unknown("usage: task <idx> <task_name>".into())
            } else {
                match args[0].parse::<usize>() {
                    Ok(idx) => DebugCommand::AssignTask(idx, args[1..].join(" ")),
                    Err(_) => DebugCommand::Unknown(format!(
                        "invalid survivor index '{}' — expected number",
                        args[0]
                    )),
                }
            }
        }
        "spawn" => {
            if args.len() < 3 {
                DebugCommand::Unknown("usage: spawn <blueprint_id> <x> <y>".into())
            } else {
                match (args[1].parse::<i32>(), args[2].parse::<i32>()) {
                    (Ok(x), Ok(y)) => DebugCommand::SpawnEntity(args[0].to_string(), x, y),
                    _ => DebugCommand::Unknown("invalid coordinates".into()),
                }
            }
        }
        "goto" => {
            if args.len() < 2 {
                DebugCommand::Unknown("usage: goto <x> <y>".into())
            } else {
                match (args[0].parse::<i32>(), args[1].parse::<i32>()) {
                    (Ok(x), Ok(y)) => DebugCommand::Teleport(x, y),
                    _ => DebugCommand::Unknown("invalid coordinates".into()),
                }
            }
        }
        "shelter" => DebugCommand::GotoShelter,
        "blueprints" => DebugCommand::ListBlueprints,
        "events" => DebugCommand::ListEvents,
        "stats" => DebugCommand::Stats,
        "help" => DebugCommand::Help,
        "clear" => DebugCommand::Clear,
        other => DebugCommand::Unknown(format!(
            "unknown command '{}' — type 'help' for available commands",
            other
        )),
    }
}

fn parse_add_resource(kind: PoolKind, args: &[&str]) -> DebugCommand {
    if args.is_empty() {
        return DebugCommand::Unknown("usage: supplies|materials|faith|plants <amount>".into());
    }
    match args[0].parse::<i32>() {
        Ok(amount) => DebugCommand::AddResource(kind, amount),
        Err(_) => {
            DebugCommand::Unknown(format!("invalid amount '{}' — expected a number", args[0]))
        }
    }
}

fn parse_u64(args: &[&str]) -> Result<u64, String> {
    if args.is_empty() {
        return Err("expected a number argument".into());
    }
    args[0]
        .parse::<u64>()
        .map_err(|_| format!("invalid number '{}'", args[0]))
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    // ── Phase 2: parser contract tests ──

    #[test]
    fn parse_empty_returns_unknown() {
        assert!(matches!(parse(""), DebugCommand::Unknown(_)));
        assert!(matches!(parse("   "), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_supplies() {
        assert_eq!(
            parse("supplies 50"),
            DebugCommand::AddResource(PoolKind::Supplies, 50)
        );
    }

    #[test]
    fn parse_supplies_negative() {
        assert_eq!(
            parse("supplies -10"),
            DebugCommand::AddResource(PoolKind::Supplies, -10)
        );
    }

    #[test]
    fn parse_supplies_missing_amount() {
        assert!(matches!(parse("supplies"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_supplies_bad_amount() {
        assert!(matches!(parse("supplies abc"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_materials() {
        assert_eq!(
            parse("materials 20"),
            DebugCommand::AddResource(PoolKind::Materials, 20)
        );
    }

    #[test]
    fn parse_faith() {
        assert_eq!(
            parse("faith 5"),
            DebugCommand::AddResource(PoolKind::Faith, 5)
        );
    }

    #[test]
    fn parse_plants() {
        assert_eq!(
            parse("plants 3"),
            DebugCommand::AddResource(PoolKind::WildPlants, 3)
        );
    }

    #[test]
    fn parse_day() {
        assert_eq!(parse("day 10"), DebugCommand::SetDay(10));
    }

    #[test]
    fn parse_day_zero() {
        assert_eq!(parse("day 0"), DebugCommand::SetDay(0));
    }

    #[test]
    fn parse_day_large() {
        assert_eq!(parse("day 999"), DebugCommand::SetDay(999));
    }

    #[test]
    fn parse_day_missing_value() {
        assert!(matches!(parse("day"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_day_bad_value() {
        assert!(matches!(parse("day abc"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_turn() {
        assert_eq!(parse("turn 42"), DebugCommand::SetTurn(42));
    }

    #[test]
    fn parse_skip_day() {
        assert_eq!(parse("skip_day"), DebugCommand::SkipDay);
    }

    #[test]
    fn parse_event() {
        assert_eq!(
            parse("event event.raid.small"),
            DebugCommand::TriggerEvent("event.raid.small".into())
        );
    }

    #[test]
    fn parse_event_missing_id() {
        assert!(matches!(parse("event"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_end_event() {
        assert_eq!(parse("end_event"), DebugCommand::EndEvent);
    }

    #[test]
    fn parse_kill_all() {
        assert_eq!(parse("kill_all"), DebugCommand::KillAllEnemies);
    }

    #[test]
    fn parse_heal() {
        assert_eq!(parse("heal"), DebugCommand::Heal);
    }

    #[test]
    fn parse_god_on() {
        assert_eq!(parse("god on"), DebugCommand::GodMode(true));
    }

    #[test]
    fn parse_god_off() {
        assert_eq!(parse("god off"), DebugCommand::GodMode(false));
    }

    #[test]
    fn parse_god_bad_arg() {
        assert!(matches!(parse("god maybe"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_god_no_arg() {
        assert!(matches!(parse("god"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_survivor_single_name() {
        assert_eq!(
            parse("survivor Mara"),
            DebugCommand::SpawnSurvivor("Mara".into())
        );
    }

    #[test]
    fn parse_survivor_multi_word_name() {
        assert_eq!(
            parse("survivor Old Man Jenkins"),
            DebugCommand::SpawnSurvivor("Old Man Jenkins".into())
        );
    }

    #[test]
    fn parse_survivor_missing_name() {
        assert!(matches!(parse("survivor"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_task() {
        assert_eq!(
            parse("task 0 gathering"),
            DebugCommand::AssignTask(0, "gathering".into())
        );
    }

    #[test]
    fn parse_task_multi_word() {
        assert_eq!(
            parse("task 2 station work"),
            DebugCommand::AssignTask(2, "station work".into())
        );
    }

    #[test]
    fn parse_task_missing_args() {
        assert!(matches!(parse("task"), DebugCommand::Unknown(_)));
        assert!(matches!(parse("task 0"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_task_bad_index() {
        assert!(matches!(
            parse("task abc gathering"),
            DebugCommand::Unknown(_)
        ));
    }

    #[test]
    fn parse_spawn() {
        assert_eq!(
            parse("spawn blueprint.rat 5 3"),
            DebugCommand::SpawnEntity("blueprint.rat".into(), 5, 3)
        );
    }

    #[test]
    fn parse_spawn_negative_coords() {
        assert_eq!(
            parse("spawn blueprint.rat -1 -2"),
            DebugCommand::SpawnEntity("blueprint.rat".into(), -1, -2)
        );
    }

    #[test]
    fn parse_spawn_missing_coords() {
        assert!(matches!(
            parse("spawn blueprint.rat"),
            DebugCommand::Unknown(_)
        ));
        assert!(matches!(
            parse("spawn blueprint.rat 5"),
            DebugCommand::Unknown(_)
        ));
    }

    #[test]
    fn parse_spawn_missing_all() {
        assert!(matches!(parse("spawn"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_spawn_bad_coords() {
        assert!(matches!(
            parse("spawn blueprint.rat abc 5"),
            DebugCommand::Unknown(_)
        ));
    }

    #[test]
    fn parse_goto() {
        assert_eq!(parse("goto 5 10"), DebugCommand::Teleport(5, 10));
    }

    #[test]
    fn parse_goto_negative() {
        assert_eq!(parse("goto -3 -7"), DebugCommand::Teleport(-3, -7));
    }

    #[test]
    fn parse_goto_missing() {
        assert!(matches!(parse("goto"), DebugCommand::Unknown(_)));
        assert!(matches!(parse("goto 5"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_shelter() {
        assert_eq!(parse("shelter"), DebugCommand::GotoShelter);
    }

    #[test]
    fn parse_blueprints() {
        assert_eq!(parse("blueprints"), DebugCommand::ListBlueprints);
    }

    #[test]
    fn parse_events() {
        assert_eq!(parse("events"), DebugCommand::ListEvents);
    }

    #[test]
    fn parse_stats() {
        assert_eq!(parse("stats"), DebugCommand::Stats);
    }

    #[test]
    fn parse_help() {
        assert_eq!(parse("help"), DebugCommand::Help);
    }

    #[test]
    fn parse_clear() {
        assert_eq!(parse("clear"), DebugCommand::Clear);
    }

    #[test]
    fn parse_unknown_command() {
        assert!(matches!(parse("foobar"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_unknown_command_with_args() {
        assert!(matches!(parse("foobar 123 xyz"), DebugCommand::Unknown(_)));
    }

    #[test]
    fn parse_leading_whitespace() {
        assert_eq!(
            parse("  supplies 10"),
            DebugCommand::AddResource(PoolKind::Supplies, 10)
        );
        assert_eq!(parse("\t\theal"), DebugCommand::Heal);
    }

    #[test]
    fn parse_trailing_whitespace() {
        assert_eq!(parse("heal  "), DebugCommand::Heal);
    }

    #[test]
    fn parse_zero_amount() {
        assert_eq!(
            parse("supplies 0"),
            DebugCommand::AddResource(PoolKind::Supplies, 0)
        );
    }

    #[test]
    fn parse_large_teleport() {
        assert_eq!(
            parse("goto 9999 -9999"),
            DebugCommand::Teleport(9999, -9999)
        );
    }
}
