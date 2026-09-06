use crate::model::hop_pipeline::HopPipeline;
use ajisai_core::AjisaiError;

/// Serialize a HopPipeline to Apache Hop .hpl XML
pub fn write_hpl(pipeline: &HopPipeline) -> Result<String, AjisaiError> {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<pipeline>\n");

    // <info>
    out.push_str("  <info>\n");
    out.push_str(&format!(
        "    <name>{}</name>\n",
        xml_escape(&pipeline.name)
    ));
    if let Some(desc) = &pipeline.info.description {
        out.push_str(&format!(
            "    <description>{}</description>\n",
            xml_escape(desc)
        ));
    }
    out.push_str("    <pipeline_version/>\n");
    out.push_str("  </info>\n\n");

    // <transform> elements
    for t in &pipeline.transforms {
        out.push_str("  <transform>\n");
        out.push_str(&format!("    <name>{}</name>\n", xml_escape(&t.name)));
        out.push_str(&format!("    <type>{}</type>\n", xml_escape(&t.type_name)));
        out.push_str(&format!(
            "    <description>{}</description>\n",
            xml_escape(t.description.as_deref().unwrap_or(""))
        ));
        out.push_str("    <distribute>Y</distribute>\n");
        out.push_str("    <custom_distribution/>\n");
        out.push_str("    <copies>1</copies>\n");

        // Write config attributes as child elements (skip keys that aren't valid XML names)
        for (key, value) in &t.attributes {
            if validate_xml_name(key).is_err() {
                continue;
            }
            let val_str = json_value_to_string(value);
            out.push_str(&format!("    <{0}>{1}</{0}>\n", key, xml_escape(&val_str),));
        }

        // GUI position
        out.push_str("    <GUI>\n");
        out.push_str(&format!("      <xloc>{}</xloc>\n", t.xloc.unwrap_or(100)));
        out.push_str(&format!("      <yloc>{}</yloc>\n", t.yloc.unwrap_or(100)));
        out.push_str("    </GUI>\n");
        out.push_str("  </transform>\n\n");
    }

    // <order>
    out.push_str("  <order>\n");
    for hop in &pipeline.order {
        out.push_str("    <hop>\n");
        out.push_str(&format!("      <from>{}</from>\n", xml_escape(&hop.from)));
        out.push_str(&format!("      <to>{}</to>\n", xml_escape(&hop.to)));
        let enabled = if hop.enabled.unwrap_or(true) {
            "Y"
        } else {
            "N"
        };
        out.push_str(&format!("      <enabled>{}</enabled>\n", enabled));
        out.push_str("    </hop>\n");
    }
    out.push_str("  </order>\n");

    out.push_str("</pipeline>\n");
    Ok(out)
}

/// Write a HopPipeline to a .hpl file on disk
pub fn write_hpl_file(pipeline: &HopPipeline, path: &std::path::Path) -> Result<(), AjisaiError> {
    let xml = write_hpl(pipeline)?;
    std::fs::write(path, xml).map_err(AjisaiError::Io)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Validate that `name` is a legal XML element name.
/// Must be non-empty, start with letter or underscore, and contain only
/// ASCII letters, digits, underscores, hyphens, or periods.
fn validate_xml_name(name: &str) -> Result<(), AjisaiError> {
    if name.is_empty() {
        return Err(AjisaiError::Config(
            "XML element name cannot be empty".into(),
        ));
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(AjisaiError::Config(format!(
            "XML element name '{}' must start with a letter or underscore",
            name
        )));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(AjisaiError::Config(format!(
            "XML element name '{}' contains invalid characters",
            name
        )));
    }
    Ok(())
}

fn json_value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => {
            if *b {
                "Y".into()
            } else {
                "N".into()
            }
        }
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::pipeline::parse_hpl;

    #[test]
    fn writes_and_parses_hop_pipeline_contract() {
        let mut pipeline = HopPipeline {
            name: "compat-fixture".into(),
            ..HopPipeline::default()
        };
        pipeline
            .transforms
            .push(crate::model::hop_pipeline::HopTransform {
                name: "input".into(),
                type_name: "CSVFileInput".into(),
                description: Some("source".into()),
                xloc: Some(10),
                yloc: Some(20),
                attributes: [("filename".into(), serde_json::json!("input.csv"))]
                    .into_iter()
                    .collect(),
            });
        pipeline.order.push(crate::model::hop_pipeline::HopHop {
            from: "input".into(),
            to: "output".into(),
            enabled: Some(true),
            error_hop: Some(false),
        });
        let parsed = parse_hpl(&write_hpl(&pipeline).unwrap()).unwrap();
        assert_eq!(parsed.name, "compat-fixture");
        assert_eq!(parsed.transforms[0].type_name, "CSVFileInput");
        assert_eq!(parsed.transforms[0].attributes["filename"], "input.csv");
        assert_eq!(parsed.order[0].from, "input");
    }

    #[test]
    fn parses_repository_hop_fixtures() {
        for path in [
            "../../tests/fixtures/sample.hpl",
            "../../tests/fixtures/showcase.hpl",
        ] {
            let xml = std::fs::read_to_string(path).unwrap();
            let parsed = parse_hpl(&xml).unwrap();
            assert!(!parsed.name.is_empty(), "fixture name missing: {path}");
            assert!(
                !parsed.transforms.is_empty(),
                "fixture transforms missing: {path}"
            );
            assert!(!parsed.order.is_empty(), "fixture hops missing: {path}");
        }
    }
}
