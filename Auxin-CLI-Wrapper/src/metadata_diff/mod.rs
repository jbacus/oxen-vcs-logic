// Auxin-CLI-Wrapper/src/metadata_diff/mod.rs
//
// Metadata diff module for Logic Pro projects

pub mod diff_engine;
pub mod diff_types;
pub mod report_generator;

pub use diff_engine::diff_metadata;
pub use diff_types::*;
pub use report_generator::ReportGenerator;

use crate::logic_parser::LogicProjectData;

/// High-level API for metadata diffing
pub struct MetadataDiffer;

impl MetadataDiffer {
    /// Compare two Logic Pro projects and generate a diff
    pub fn compare(project_a: &LogicProjectData, project_b: &LogicProjectData) -> MetadataDiff {
        diff_metadata(project_a, project_b)
    }

    /// Generate a human-readable report from a diff
    pub fn generate_report(diff: &MetadataDiff) -> String {
        ReportGenerator::new().generate_report(diff)
    }

    /// Generate a report with custom options
    pub fn generate_report_with_options(
        diff: &MetadataDiff,
        use_color: bool,
        verbose: bool,
    ) -> String {
        ReportGenerator::new()
            .with_color(use_color)
            .with_verbose(verbose)
            .generate_report(diff)
    }

    /// Generate JSON output from a diff
    pub fn to_json(diff: &MetadataDiff) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic_parser::LogicProjectData;

    fn create_test_project(tempo: f32) -> LogicProjectData {
        LogicProjectData {
            tempo,
            sample_rate: 48000,
            key_signature: "C Major".to_string(),
            time_signature: (4, 4),
            bit_depth: 24,
            tracks: vec![],
            automation: vec![],
            plugins: vec![],
            logic_version: "11.0.0".to_string(),
        }
    }

    #[test]
    fn test_compare_projects() {
        let project_a = create_test_project(120.0);
        let project_b = create_test_project(128.0);

        let diff = MetadataDiffer::compare(&project_a, &project_b);
        assert!(diff.has_changes());
    }

    #[test]
    fn test_generate_report() {
        let project_a = create_test_project(120.0);
        let project_b = create_test_project(128.0);

        let diff = MetadataDiffer::compare(&project_a, &project_b);
        let report = MetadataDiffer::generate_report(&diff);

        assert!(!report.is_empty());
        assert!(report.contains("Tempo"));
    }

    #[test]
    fn test_json_output() {
        let project_a = create_test_project(120.0);
        let project_b = create_test_project(128.0);

        let diff = MetadataDiffer::compare(&project_a, &project_b);
        let json = MetadataDiffer::to_json(&diff).unwrap();

        assert!(!json.is_empty());
        assert!(json.contains("global_changes"));
    }
}
