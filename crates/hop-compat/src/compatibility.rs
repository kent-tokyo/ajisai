use crate::{HopPipeline, HopWorkflow, map_transform_type};
use ajisai_core::TransformRegistry;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityStatus {
    Supported,
    SupportedWithDifference,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransformAssessment {
    pub source_type: String,
    pub ajisai_type: String,
    pub name: String,
    pub status: CompatibilityStatus,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HopCompatibilityReport {
    pub schema_version: u16,
    pub format: &'static str,
    pub pipeline: String,
    pub status: CompatibilityStatus,
    pub transforms: Vec<TransformAssessment>,
    pub differences: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowAssessment {
    pub name: String,
    pub action_type: String,
    pub status: CompatibilityStatus,
    pub reason_code: String,
}

pub fn assess_workflow(workflow: &HopWorkflow) -> Vec<WorkflowAssessment> {
    workflow
        .actions
        .iter()
        .map(|action| {
            let key = action.type_name.to_ascii_uppercase();
            let known = matches!(
                key.as_str(),
                "SHELL"
                    | "FILE_EXISTS"
                    | "FILEEXISTS"
                    | "CREATE_FOLDER"
                    | "CREATEFOLDER"
                    | "COPY_FILES"
                    | "COPYFILES"
                    | "DELETE_FILES"
                    | "DELETEFILES"
                    | "SET_VARIABLES"
                    | "SETVARIABLES"
                    | "HTTP"
            );
            let (status, reason_code) = if known {
                (CompatibilityStatus::Supported, "WORKFLOW_ACTION_NATIVE")
            } else {
                (
                    CompatibilityStatus::SupportedWithDifference,
                    "WORKFLOW_ACTION_AS_PIPELINE",
                )
            };
            WorkflowAssessment {
                name: action.name.clone(),
                action_type: action.type_name.clone(),
                status,
                reason_code: reason_code.into(),
            }
        })
        .collect()
}

/// Assess a parsed Hop pipeline without executing or touching external systems.
pub fn assess_pipeline(
    pipeline: &HopPipeline,
    registry: &TransformRegistry,
) -> HopCompatibilityReport {
    let available = registry.list();
    let transforms = pipeline
        .transforms
        .iter()
        .map(|transform| {
            let mapped = map_transform_type(&transform.type_name).to_owned();
            let (status, reason_code) = if !available.iter().any(|name| *name == mapped) {
                (CompatibilityStatus::Unsupported, "TRANSFORM_UNSUPPORTED")
            } else if mapped != transform.type_name {
                (
                    CompatibilityStatus::SupportedWithDifference,
                    "TRANSFORM_RENAMED",
                )
            } else {
                (CompatibilityStatus::Supported, "TRANSFORM_NATIVE")
            };
            TransformAssessment {
                source_type: transform.type_name.clone(),
                ajisai_type: mapped,
                name: transform.name.clone(),
                status,
                reason_code: reason_code.into(),
            }
        })
        .collect::<Vec<_>>();

    let mut differences = Vec::new();
    for transform in &transforms {
        if transform.status != CompatibilityStatus::Supported {
            differences.push(format!("{}:{}", transform.reason_code, transform.name));
        }
    }
    let status = if transforms
        .iter()
        .any(|item| item.status == CompatibilityStatus::Unsupported)
    {
        CompatibilityStatus::Unsupported
    } else if transforms
        .iter()
        .any(|item| item.status == CompatibilityStatus::SupportedWithDifference)
    {
        CompatibilityStatus::SupportedWithDifference
    } else {
        CompatibilityStatus::Supported
    };

    HopCompatibilityReport {
        schema_version: 1,
        format: "hop.compatibility",
        pipeline: pipeline.name.clone(),
        status,
        transforms,
        differences,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_hpl;
    use ajisai_core::TransformRegistry;
    use std::sync::Arc;

    #[test]
    fn report_classifies_supported_renamed_and_unsupported_transforms() {
        let pipeline = parse_hpl(
            r#"<pipeline><name>assessment</name><transform><name>a</name><type>CSVFileInput</type></transform><transform><name>b</name><type>DefinitelyUnknown</type></transform></pipeline>"#,
        )
        .unwrap();
        let mut registry = TransformRegistry::new();
        registry.register(
            "CsvFileInput",
            Arc::new(|_| Err(ajisai_core::AjisaiError::Config("test factory".into()))),
        );
        let report = assess_pipeline(&pipeline, &registry);
        assert_eq!(report.status, CompatibilityStatus::Unsupported);
        assert_eq!(report.transforms[0].reason_code, "TRANSFORM_RENAMED");
        assert_eq!(report.transforms[1].reason_code, "TRANSFORM_UNSUPPORTED");
    }
}
