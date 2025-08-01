use serde::{Deserialize, Serialize};

/// FHIR Observation status (simplified)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObservationStatus {
    Preliminary,
    Final,
    Amended,
    Cancelled,
}

/// FHIR CodeableConcept (very small subset)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeableConcept {
    pub code: String,
    pub display: Option<String>,
}

/// FHIR Quantity (value with unit)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

/// Simplified FHIR Observation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    pub id: Option<String>,
    pub status: ObservationStatus,
    pub code: CodeableConcept,
    pub value: Option<Quantity>,
}

