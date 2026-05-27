#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignoffInput {
    pub triage_gate_complete: bool,
    pub metrics_met: bool,
    pub full_gate_green: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignoffDecision {
    Accepted,
    Rejected,
}

pub fn evaluate_signoff(input: SignoffInput) -> SignoffDecision {
    if input.triage_gate_complete && input.metrics_met && input.full_gate_green {
        return SignoffDecision::Accepted;
    }

    SignoffDecision::Rejected
}
