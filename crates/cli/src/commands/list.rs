use ajisai_transforms::default_registry;
use rust_i18n::t;

pub fn list_transforms() {
    let registry = default_registry();
    let names = registry.list();
    println!(
        "{}",
        t!("list.header", count = names.len().to_string().as_str())
    );
    for name in names {
        println!("  - {}", name);
    }
}
