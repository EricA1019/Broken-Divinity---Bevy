use bevy::prelude::*;

const BRP_STALE_ENTITY_ERROR_CODE: &str = "brp.stale_entity";
const BRP_STALE_ENTITY_HINT: &str =
    "Entity no longer exists. Refresh entity selection before reading or mutating state.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrpEntityDiagnostics {
    pub code: &'static str,
    pub operation: &'static str,
    pub entity: Entity,
    pub hint: &'static str,
}

impl BrpEntityDiagnostics {
    fn stale_entity(entity: Entity, operation: &'static str) -> Self {
        Self {
            code: BRP_STALE_ENTITY_ERROR_CODE,
            operation,
            entity,
            hint: BRP_STALE_ENTITY_HINT,
        }
    }

    pub fn to_log_message(&self) -> String {
        format!(
            "[{}] operation={} entity={:?} hint={}",
            self.code, self.operation, self.entity, self.hint
        )
    }
}

/// Guard BRP-style entity operations against stale/despawned entities.
pub fn validate_brp_entity_access(
    world: &World,
    entity: Entity,
    operation: &'static str,
) -> Result<(), BrpEntityDiagnostics> {
    if world.get_entity(entity).is_ok() {
        Ok(())
    } else {
        Err(BrpEntityDiagnostics::stale_entity(entity, operation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_OPERATION: &str = "qa.read_components";

    #[test]
    fn validate_brp_entity_access_accepts_live_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let result = validate_brp_entity_access(&world, entity, TEST_OPERATION);

        assert!(
            result.is_ok(),
            "Expected live entity access validation to succeed"
        );
    }

    #[test]
    fn validate_brp_entity_access_returns_structured_diagnostic_for_stale_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let _ = world.despawn(entity);

        let diagnostic = validate_brp_entity_access(&world, entity, TEST_OPERATION)
            .expect_err("Expected stale entity access to return diagnostics");

        assert_eq!(diagnostic.code, BRP_STALE_ENTITY_ERROR_CODE);
        assert_eq!(diagnostic.operation, TEST_OPERATION);
        assert_eq!(diagnostic.entity, entity);
        assert_eq!(diagnostic.hint, BRP_STALE_ENTITY_HINT);
        assert!(
            diagnostic
                .to_log_message()
                .contains(BRP_STALE_ENTITY_ERROR_CODE),
            "Expected diagnostic log line to include stable error code"
        );
    }
}
