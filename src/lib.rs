pub mod core;
pub mod game;
pub mod ui;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod determinism_tests {
    use crate::game::colony::mapgen::generate_shelter;
    use crate::game::dungeon::bsp::generate_floor;
    use crate::game::factions::generate_factions;
    use crate::game::overworld::graphgen::generate_overworld;
    use crate::game::overworld::weather::roll_weather;

    #[test]
    fn test_same_seed_same_dungeon() {
        let seed = 42u64;
        let d1 = generate_floor(80, 50, seed);
        let d2 = generate_floor(80, 50, seed);

        assert_eq!(d1.rooms.len(), d2.rooms.len());
        assert_eq!(d1.spawn_point, d2.spawn_point);
        assert_eq!(d1.width, d2.width);
        assert_eq!(d1.height, d2.height);
        for (r1, r2) in d1.rooms.iter().zip(d2.rooms.iter()) {
            assert_eq!(r1.x, r2.x);
            assert_eq!(r1.y, r2.y);
            assert_eq!(r1.w, r2.w);
            assert_eq!(r1.h, r2.h);
        }
        assert_eq!(d1.tiles, d2.tiles);
    }

    #[test]
    fn test_same_seed_same_overworld() {
        let g1 = generate_overworld(42);
        let g2 = generate_overworld(42);

        assert_eq!(g1.nodes.len(), g2.nodes.len());
        assert_eq!(g1.roads.len(), g2.roads.len());
        for (n1, n2) in g1.nodes.iter().zip(g2.nodes.iter()) {
            assert_eq!(n1.node_type, n2.node_type);
            assert_eq!(n1.name, n2.name);
            assert_eq!(n1.x, n2.x);
            assert_eq!(n1.y, n2.y);
        }
        // Roads come from a HashSet so iteration order is unstable;
        // sort by (from, to) before comparing.
        let mut r1: Vec<_> = g1.roads.iter().map(|r| (r.from, r.to)).collect();
        let mut r2: Vec<_> = g2.roads.iter().map(|r| (r.from, r.to)).collect();
        r1.sort();
        r2.sort();
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_same_seed_same_weather() {
        for day in 0..10 {
            assert_eq!(
                roll_weather(42, day),
                roll_weather(42, day),
                "Weather mismatch on day {day}"
            );
        }
    }

    #[test]
    fn test_same_seed_same_factions() {
        let f1 = generate_factions(42, 10);
        let f2 = generate_factions(42, 10);

        assert_eq!(f1.0.len(), f2.0.len());
        for (a, b) in f1.0.iter().zip(f2.0.iter()) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.archetype, b.archetype);
            assert_eq!(a.disposition, b.disposition);
            assert_eq!(a.home_node, b.home_node);
        }
    }

    #[test]
    fn test_same_seed_same_shelter() {
        let s1 = generate_shelter(42);
        let s2 = generate_shelter(42);

        assert_eq!(s1.rooms.len(), s2.rooms.len());
        assert_eq!(s1.width, s2.width);
        assert_eq!(s1.height, s2.height);
        assert_eq!(s1.spawn_point, s2.spawn_point);
        for (a, b) in s1.rooms.iter().zip(s2.rooms.iter()) {
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.rect.x, b.rect.x);
            assert_eq!(a.rect.y, b.rect.y);
            assert_eq!(a.rect.w, b.rect.w);
            assert_eq!(a.rect.h, b.rect.h);
        }
        assert_eq!(s1.tiles, s2.tiles);
    }
}
