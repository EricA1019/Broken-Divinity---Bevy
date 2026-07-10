---
name: add-system
description: "Adding a new Bevy ECS system to the project — from deciding where it lives through registration and testing."
triggers:
  - "add system"
  - "new system"
  - "create system"
  - "implement system"
edges:
  - target: context/architecture.md
    condition: to determine which tier the system belongs in
  - target: context/conventions.md
    condition: to verify code patterns and naming
last_updated: 2026-04-06
---

# Add a Bevy ECS System

## Context

Load `context/architecture.md` to check the tier diagram. Load `context/conventions.md` for the verify checklist. If the system involves game mechanics, also load the gameplay-mechanics Copilot skill. If it involves enemy AI, load the enemy-ai skill.

## Steps

1. **Determine tier** — which module does this system belong to? Check the 5-tier hierarchy in architecture.md. If the system needs a new module, follow the new-module checklist there.
2. **Name the system** — verb_noun format (`resolve_attack_events`, `tick_survivor_needs`, `spawn_enemies`)
3. **Write the function** — follow the system skeleton:
   ```rust
   pub fn my_system_name(
       mut query: Query<(&mut Component, &OtherComponent), With<Filter>>,
       mut log: ResMut<GameLog>,
   ) {
       let Ok((mut comp, other)) = query.single_mut() else { return; };
       // logic...
   }
   ```
4. **Register in main.rs** — group by AppState lifecycle, add state gating:
   ```rust
  .add_systems(Update, my_system.run_if(in_state(AppState::Dungeon)))
   ```
5. **Register messages** — if the system defines new Messages, add `app.add_message::<MyMessage>()` in main.rs
6. **Write a test** — add a `#[cfg(test)]` test using `MinimalPlugins` and `run_system_once`

## Gotchas

- **Tier violation**: double-check imports don't reach up to a higher tier. If they do, move the shared type down to Tier 0
- **Ungated system**: every system must be gated to at least one AppState. No bare `.add_systems(Update, sys)`
- **Events vs Messages**: Bevy 0.18 uses `#[derive(Message)]` — NOT `#[derive(Event)]`
- **Query unwrap**: `.unwrap()` on a query is an instant reject. Always use `let Ok(...) else { return; }`

## Verify

- [ ] System uses graceful query failure
- [ ] System is state-gated in main.rs
- [ ] Tier hierarchy respected (no upward imports)
- [ ] New Messages registered with `add_message`
- [ ] Naming follows verb_noun convention
- [ ] Test exists in `#[cfg(test)] mod tests`

## Debug

- **System never runs**: check AppState/TurnState gating — is the state actually entered?
- **Query returns nothing**: verify the entity has all required components in the query filter
- **Messages not received**: check both writer and reader use the same Message type, and `add_message` is registered
