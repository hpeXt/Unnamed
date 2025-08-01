use super::fhir::Observation;

/// Very small in-memory aggregator for health observations
#[derive(Default)]
pub struct HealthDataAggregator {
    observations: Vec<Observation>,
}

impl HealthDataAggregator {
    /// Create new empty aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an observation to the aggregator
    pub fn add_observation(&mut self, obs: Observation) {
        self.observations.push(obs);
    }

    /// Retrieve all stored observations
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }
}

