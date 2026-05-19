pub mod calculator;
pub mod constants;
pub mod csv;
pub mod db;
pub mod deduplicate;
pub mod filter;
pub mod if_null;
pub mod json;
pub mod merge_join;
pub mod registry;
pub mod select;
pub mod sort;
pub mod split_field;
pub mod stream_lookup;
pub mod string_ops;

pub use calculator::CalculatorStep;
pub use constants::AddConstants;
pub use csv::{CsvFileInput, CsvFileOutput};
pub use db::{DatabaseLookup, TableInput, TableOutput};
pub use deduplicate::Deduplicate;
pub use filter::FilterRows;
pub use if_null::IfNull;
pub use json::{JsonFileInput, JsonFileOutput};
pub use merge_join::MergeJoin;
pub use registry::TransformRegistry;
pub use select::SelectValues;
pub use sort::SortRows;
pub use split_field::SplitFieldToRows;
pub use stream_lookup::StreamLookup;
pub use string_ops::{ConcatFields, ReplaceInString, StringOperations};

use std::sync::Arc;

/// Build a registry pre-loaded with all built-in transforms
pub fn default_registry() -> TransformRegistry {
    let mut reg = TransformRegistry::new();

    reg.register(
        "CsvFileInput",
        Arc::new(|v| csv::input::CsvFileInput::from_json(v)),
    );
    reg.register(
        "CsvFileOutput",
        Arc::new(|v| csv::output::CsvFileOutput::from_json(v)),
    );
    reg.register(
        "JsonFileInput",
        Arc::new(|v| json::input::JsonFileInput::from_json(v)),
    );
    reg.register(
        "JsonFileOutput",
        Arc::new(|v| json::output::JsonFileOutput::from_json(v)),
    );
    reg.register("FilterRows", Arc::new(|v| filter::FilterRows::from_json(v)));
    reg.register(
        "SelectValues",
        Arc::new(|v| select::SelectValues::from_json(v)),
    );
    reg.register("SortRows", Arc::new(|v| sort::SortRows::from_json(v)));
    reg.register(
        "AddConstants",
        Arc::new(|v| constants::AddConstants::from_json(v)),
    );
    reg.register(
        "Deduplicate",
        Arc::new(|v| deduplicate::Deduplicate::from_json(v)),
    );
    reg.register(
        "StreamLookup",
        Arc::new(|v| stream_lookup::StreamLookup::from_json(v)),
    );
    reg.register(
        "CalculatorStep",
        Arc::new(|v| calculator::CalculatorStep::from_json(v)),
    );
    reg.register(
        "MergeJoin",
        Arc::new(|v| merge_join::MergeJoin::from_json(v)),
    );
    reg.register(
        "TableInput",
        Arc::new(|v| db::input::TableInput::from_json(v)),
    );
    reg.register(
        "TableOutput",
        Arc::new(|v| db::output::TableOutput::from_json(v)),
    );
    reg.register(
        "DatabaseLookup",
        Arc::new(|v| db::lookup::DatabaseLookup::from_json(v)),
    );
    reg.register("IfNull", Arc::new(|v| if_null::IfNull::from_json(v)));
    reg.register(
        "StringOperations",
        Arc::new(|v| string_ops::StringOperations::from_json(v)),
    );
    reg.register(
        "ReplaceInString",
        Arc::new(|v| string_ops::ReplaceInString::from_json(v)),
    );
    reg.register(
        "ConcatFields",
        Arc::new(|v| string_ops::ConcatFields::from_json(v)),
    );
    reg.register(
        "SplitFieldToRows",
        Arc::new(|v| split_field::SplitFieldToRows::from_json(v)),
    );

    reg
}
