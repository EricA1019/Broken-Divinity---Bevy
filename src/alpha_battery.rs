#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefectSeverity {
    P0,
    P1,
    P2,
    P3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Defect {
    severity: DefectSeverity,
    open: bool,
}

impl Defect {
    pub fn open(severity: DefectSeverity) -> Self {
        Self {
            severity,
            open: true,
        }
    }

    pub fn closed(severity: DefectSeverity) -> Self {
        Self {
            severity,
            open: false,
        }
    }
}

pub fn triage_gate_passes(defects: &[Defect]) -> bool {
    !defects
        .iter()
        .any(|defect| defect.open && matches!(defect.severity, DefectSeverity::P0 | DefectSeverity::P1))
}
