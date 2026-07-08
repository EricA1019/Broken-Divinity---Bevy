use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedBy(pub Entity);
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainedIn(pub Entity);
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquippedBy(pub Entity);
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SummonedBy(pub Entity);
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LocationOwned(pub String);
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct FactionMember(pub String);

/// Build an entity list from a filtered query. Helper for tests.
pub(crate) fn iter_filtered<C>(world: &mut World) -> Vec<(Entity, C)>
where
    C: bevy_ecs::component::Component + Clone,
{
    let mut q = world.query_filtered::<(Entity, &C), With<C>>();
    q.iter(world).map(|(e, c)| (e, c.clone())).collect()
}

pub fn entities_owned_by(owner: Entity, world: &mut World) -> Vec<Entity> {
    iter_filtered::<OwnedBy>(world)
        .into_iter()
        .filter(|(_, o)| o.0 == owner)
        .map(|(e, _)| e)
        .collect()
}

pub fn items_in_container(container: Entity, world: &mut World) -> Vec<Entity> {
    iter_filtered::<ContainedIn>(world)
        .into_iter()
        .filter(|(_, c)| c.0 == container)
        .map(|(e, _)| e)
        .collect()
}

pub fn equipment_for(equipper: Entity, world: &mut World) -> Vec<Entity> {
    iter_filtered::<EquippedBy>(world)
        .into_iter()
        .filter(|(_, eq)| eq.0 == equipper)
        .map(|(e, _)| e)
        .collect()
}

pub fn summons_for(summoner: Entity, world: &mut World) -> Vec<Entity> {
    iter_filtered::<SummonedBy>(world)
        .into_iter()
        .filter(|(_, s)| s.0 == summoner)
        .map(|(e, _)| e)
        .collect()
}

pub fn entities_in_location(location_id: &str, world: &mut World) -> Vec<Entity> {
    iter_filtered::<LocationOwned>(world)
        .into_iter()
        .filter(|(_, l)| l.0 == location_id)
        .map(|(e, _)| e)
        .collect()
}

pub fn validate_relationships(world: &mut World) -> Vec<String> {
    let mut errors = Vec::new();
    for (id, ci) in &iter_filtered::<ContainedIn>(world) {
        if !world.entities().contains(ci.0) {
            errors.push(format!(
                "Entity {id:?} has ContainedIn({:?}) but container missing",
                ci.0
            ));
        }
    }
    for (id, eq) in &iter_filtered::<EquippedBy>(world) {
        if !world.entities().contains(eq.0) {
            errors.push(format!(
                "Entity {id:?} has EquippedBy({:?}) but equipper missing",
                eq.0
            ));
        }
    }
    for (id, s) in &iter_filtered::<SummonedBy>(world) {
        if !world.entities().contains(s.0) {
            errors.push(format!(
                "Entity {id:?} has SummonedBy({:?}) but summoner missing",
                s.0
            ));
        }
    }
    errors
}

pub fn would_create_cycle(container: Entity, item: Entity, world: &World, max_depth: u32) -> bool {
    let mut current = container;
    for _ in 0..max_depth {
        if current == item {
            return true;
        }
        match world.get::<ContainedIn>(current) {
            Some(p) => current = p.0,
            None => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> World {
        World::new()
    }

    #[test]
    fn item_can_be_contained() {
        let mut w = setup();
        let c = w.spawn_empty().id();
        let item = w.spawn(ContainedIn(c)).id();
        assert!(items_in_container(c, &mut w).contains(&item));
    }
    #[test]
    fn item_can_be_equipped() {
        let mut w = setup();
        let p = w.spawn_empty().id();
        let wep = w.spawn(EquippedBy(p)).id();
        assert!(equipment_for(p, &mut w).contains(&wep));
    }
    #[test]
    fn equipped_item_can_be_queried() {
        let mut w = setup();
        let p = w.spawn_empty().id();
        w.spawn(EquippedBy(p));
        w.spawn(EquippedBy(p));
        assert_eq!(equipment_for(p, &mut w).len(), 2);
    }
    #[test]
    fn contained_item_can_be_queried() {
        let mut w = setup();
        let c = w.spawn_empty().id();
        w.spawn(ContainedIn(c));
        w.spawn(ContainedIn(c));
        assert_eq!(items_in_container(c, &mut w).len(), 2);
    }
    #[test]
    fn containment_cycle_is_rejected() {
        let mut w = setup();
        let a = w.spawn_empty().id();
        let b = w.spawn(ContainedIn(a)).id();
        w.entity_mut(a).insert(ContainedIn(b));
        assert!(would_create_cycle(b, a, &w, 10));
    }
    #[test]
    fn summon_has_summoner() {
        let mut w = setup();
        let s = w.spawn_empty().id();
        let s2 = w.spawn(SummonedBy(s)).id();
        assert!(summons_for(s, &mut w).contains(&s2));
    }
    #[test]
    fn location_owned_entity_can_be_queried() {
        let mut w = setup();
        let e = w.spawn(LocationOwned("loc.1".into())).id();
        assert!(entities_in_location("loc.1", &mut w).contains(&e));
        assert!(entities_in_location("loc.2", &mut w).is_empty());
    }
    #[test]
    fn validate_relationships_catches_missing() {
        let mut w = setup();
        let m = w.spawn_empty().id();
        w.despawn(m);
        w.spawn(ContainedIn(m));
        assert!(
            validate_relationships(&mut w)
                .iter()
                .any(|e| e.contains("ContainedIn"))
        );
    }
}
