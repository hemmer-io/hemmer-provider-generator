//! Confidence scoring for analysis results

use serde::{Deserialize, Serialize};

/// Confidence scores for each analyzed field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceReport {
    /// Overall weighted confidence (0.0-1.0)
    pub overall: f32,

    /// Confidence in crate pattern detection
    pub crate_pattern: f32,

    /// Confidence in client type pattern detection
    pub client_type: f32,

    /// Confidence in config crate detection
    pub config_crate: f32,

    /// Confidence in config attributes detection
    pub config_attrs: f32,

    /// Confidence in error categorization
    pub error_categorization: f32,
}

impl ConfidenceReport {
    /// Create a new confidence report with individual scores
    pub fn new(
        crate_pattern: f32,
        client_type: f32,
        config_crate: f32,
        config_attrs: f32,
        error_categorization: f32,
    ) -> Self {
        let overall = Self::calculate_overall(
            crate_pattern,
            client_type,
            config_crate,
            config_attrs,
            error_categorization,
        );

        Self {
            overall,
            crate_pattern,
            client_type,
            config_crate,
            config_attrs,
            error_categorization,
        }
    }

    /// Calculate weighted overall confidence
    /// Weights: crate(0.3) + client(0.3) + config_crate(0.15) + config_attrs(0.05) + errors(0.2)
    fn calculate_overall(
        crate_pattern: f32,
        client_type: f32,
        config_crate: f32,
        config_attrs: f32,
        error_categorization: f32,
    ) -> f32 {
        const CRATE_WEIGHT: f32 = 0.3;
        const CLIENT_WEIGHT: f32 = 0.3;
        const CONFIG_CRATE_WEIGHT: f32 = 0.15;
        const CONFIG_ATTRS_WEIGHT: f32 = 0.05;
        const ERROR_WEIGHT: f32 = 0.2;

        (crate_pattern * CRATE_WEIGHT
            + client_type * CLIENT_WEIGHT
            + config_crate * CONFIG_CRATE_WEIGHT
            + config_attrs * CONFIG_ATTRS_WEIGHT
            + error_categorization * ERROR_WEIGHT)
            .clamp(0.0, 1.0)
    }

    /// Get confidence level as a human-readable string
    pub fn level(&self) -> &'static str {
        match self.overall {
            x if x >= 0.8 => "HIGH",
            x if x >= 0.6 => "MEDIUM",
            _ => "LOW",
        }
    }

    /// Check if a field needs manual review (confidence < 0.7)
    pub fn needs_review(&self, field: &str) -> bool {
        let score = match field {
            "crate_pattern" => self.crate_pattern,
            "client_type" => self.client_type,
            "config_crate" => self.config_crate,
            "config_attrs" => self.config_attrs,
            "error_categorization" => self.error_categorization,
            _ => 0.0,
        };
        score < 0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_calculation() {
        let report = ConfidenceReport::new(0.95, 0.90, 0.85, 0.70, 0.60);

        // Expected: 0.95*0.3 + 0.90*0.3 + 0.85*0.15 + 0.70*0.05 + 0.60*0.2
        //         = 0.285 + 0.27 + 0.1275 + 0.035 + 0.12 = 0.8375
        assert!((report.overall - 0.8375).abs() < 0.001);
        assert_eq!(report.level(), "HIGH");
    }

    #[test]
    fn test_confidence_levels() {
        assert_eq!(
            ConfidenceReport::new(0.9, 0.9, 0.9, 0.9, 0.9).level(),
            "HIGH"
        );
        assert_eq!(
            ConfidenceReport::new(0.7, 0.7, 0.7, 0.7, 0.7).level(),
            "MEDIUM"
        );
        assert_eq!(
            ConfidenceReport::new(0.5, 0.5, 0.5, 0.5, 0.5).level(),
            "LOW"
        );
    }

    #[test]
    fn test_needs_review() {
        let report = ConfidenceReport::new(0.95, 0.90, 0.65, 0.50, 0.60);

        assert!(!report.needs_review("crate_pattern"));
        assert!(!report.needs_review("client_type"));
        assert!(report.needs_review("config_crate"));
        assert!(report.needs_review("config_attrs"));
        assert!(report.needs_review("error_categorization"));
    }
}
