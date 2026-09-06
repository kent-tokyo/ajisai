use ajisai_transforms::default_registry;
use rust_i18n::t;

pub fn list_transforms(json_output: bool) {
    let registry = default_registry();
    let names = registry.list();
    if json_output {
        let transforms: Vec<_> = names
            .iter()
            .map(|name| {
                serde_json::json!({
                    "type_name": name,
                    "manifest_version": 1,
                    "buffering": "unspecified",
                "capabilities": [],
                    "deterministic": true
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({"schema_version": 1, "transforms": transforms})
        );
        return;
    }
    println!(
        "{}",
        t!("list.header", count = names.len().to_string().as_str())
    );
    for name in names {
        println!("  - {}", name);
    }
}
