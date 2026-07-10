---
name: add-component-resource
description: "Adding a new ECS component or resource — placement, registration, serde defaults, and save compatibility."
triggers:
  - "add component"
  - "new component"
  - "add resource"
  - "new resource"
  - "new field"
edges:
  - target: context/conventions.md
    condition: to verify naming and placement rules
  - target: context/architecture.md
    condition: to understand Tier 0 placement requirements
last_updated: 2026-04-05
---

# Add a Component or Resource

## Context

Load `context/conventions.md` for naming rules and the verify checklist. Components and shared resources live in Tier 0 (`components.rs` / `resources.rs`) — never in higher-tier modules unless module-private.

## Steps

1. **Decide: Component or Resource?**
   - **Component** — per-entity data (Health, Position, CombatStats, Enemy marker)
   - **Resource** — global singleton state (GameLog, GameTime, PendingAction)
2. **Name it** — PascalCase noun. Components are data, not behavior.
3. **Place it** — in `components.rs` (component) or `resources.rs` (shared resource). Module-private resources can live in their own module.
4. **Derive traits**:
   ```rust
   // Component
   #[derive(Component, Default, Clone, Debug, Serialize, Deserialize)]
   pub struct MyComponent {
       #[serde(default)]
       pub field: i32,
   }

   // Resource
   #[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
   pub struct MyResource {
       #[serde(default)]
       pub field: i32,
   }
   ```
5. **Save compatibility** — every field that will be serialized MUST have `#[serde(default)]` or `#[serde(default = "fn")]`
6. **Init resource** — if it's a Resource, insert it during setup:
   ```rust
   app.insert_resource(MyResource::default());
   ```

## Gotchas

- **Missing serde default**: old save files will fail to deserialize if new fields lack defaults
- **Component in wrong file**: shared components go in `components.rs` (Tier 0), not scattered across modules
- **Forgetting init**: Resources must be inserted — queries for `Res<MyResource>` will panic if the resource doesn't exist
- **Reflect**: add `#[derive(Reflect)]` if the component/resource needs to be visible in Bevy's inspector or save reflection

## Verify

- [ ] Placed in correct file (components.rs or resources.rs for shared, local for private)
- [ ] All serialized fields have `#[serde(default)]`
- [ ] PascalCase naming
- [ ] Resource is inserted during app setup
- [ ] Derives include Component/Resource + Serialize + Deserialize

## Debug

- **Panic on Res<T> access**: resource not inserted — add `app.insert_resource(T::default())`
- **Save file breaks after adding field**: missing `#[serde(default)]` on the new field
- **Query never matches**: entity doesn't have the component — check spawn/insert code
