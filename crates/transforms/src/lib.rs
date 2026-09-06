pub mod abort;
pub mod add_sequence;
pub mod analytic_query;
pub mod append_streams;
pub mod calculator;
pub mod clone_row;
pub mod constants;
pub mod csv;
pub mod db;
pub mod deduplicate;
pub mod dummy;
pub mod excel_file_input;
pub mod excel_file_output;
pub mod execute_sql;
pub mod field_splitter;
pub mod filter;
pub mod generate_rows;
pub mod get_file_names;
pub mod get_variable;
pub mod if_null;
pub mod json;
pub mod json_field_input;
pub mod json_field_output;
pub mod load_file_content;
pub mod memory_group_by;
pub mod merge_join;
pub mod number_range;
pub mod parquet_file_input;
pub mod parquet_file_output;
mod parquet_utils;
pub mod pipeline_executor;
pub mod regex_eval;
pub mod registry;
pub mod rest_client;
pub mod row_denormaliser;
pub mod row_normaliser;
#[cfg(feature = "scripting")]
pub mod script_step;
pub mod select;
pub mod set_variable;
pub mod sort;
pub mod split_field;
pub mod stream_lookup;
pub mod string_ops;
pub mod switch_case;
pub mod unique_rows;
pub mod utils;
pub mod value_mapper;
pub mod write_to_file;
pub mod write_to_log;
pub mod xml_file_input;
pub mod xml_file_output;

pub use abort::Abort;
pub use add_sequence::AddSequence;
pub use analytic_query::AnalyticQuery;
pub use append_streams::AppendStreams;
pub use calculator::CalculatorStep;
pub use clone_row::CloneRow;
pub use constants::AddConstants;
pub use csv::{CsvFileInput, CsvFileOutput};
pub use db::{DatabaseLookup, TableInput, TableOutput};
pub use deduplicate::Deduplicate;
pub use dummy::Dummy;
pub use excel_file_input::ExcelFileInput;
pub use excel_file_output::ExcelFileOutput;
pub use execute_sql::ExecuteSQL;
pub use field_splitter::FieldSplitter;
pub use filter::FilterRows;
pub use generate_rows::GenerateRows;
pub use get_file_names::GetFileNames;
pub use get_variable::GetVariable;
pub use if_null::IfNull;
pub use json::{JsonFileInput, JsonFileOutput};
pub use json_field_input::JsonFieldInput;
pub use json_field_output::JsonFieldOutput;
pub use load_file_content::LoadFileContent;
pub use memory_group_by::MemoryGroupBy;
pub use merge_join::MergeJoin;
pub use number_range::NumberRange;
pub use parquet_file_input::ParquetFileInput;
pub use parquet_file_output::ParquetFileOutput;
pub use pipeline_executor::PipelineExecutor;
pub use regex_eval::RegexEval;
pub use registry::TransformRegistry;
pub use rest_client::RestClient;
pub use row_denormaliser::RowDenormaliser;
pub use row_normaliser::RowNormaliser;
#[cfg(feature = "scripting")]
pub use script_step::ScriptStep;
pub use select::SelectValues;
pub use set_variable::SetVariable;
pub use sort::SortRows;
pub use split_field::SplitFieldToRows;
pub use stream_lookup::StreamLookup;
pub use string_ops::{ConcatFields, ReplaceInString, StringOperations};
pub use switch_case::SwitchCase;
pub use unique_rows::UniqueRows;
pub use value_mapper::ValueMapper;
pub use write_to_file::WriteToFile;
pub use write_to_log::WriteToLog;
pub use xml_file_input::XmlFileInput;
pub use xml_file_output::XmlFileOutput;

use std::sync::Arc;

/// Build a registry pre-loaded with all built-in transforms
pub fn default_registry() -> TransformRegistry {
    let mut reg = TransformRegistry::new();

    reg.register(
        "CsvFileInput",
        Arc::new(csv::input::CsvFileInput::from_json),
    );
    reg.register(
        "CsvFileOutput",
        Arc::new(csv::output::CsvFileOutput::from_json),
    );
    reg.register(
        "JsonFileInput",
        Arc::new(json::input::JsonFileInput::from_json),
    );
    reg.register(
        "JsonFileOutput",
        Arc::new(json::output::JsonFileOutput::from_json),
    );
    reg.register("FilterRows", Arc::new(filter::FilterRows::from_json));
    reg.register("SelectValues", Arc::new(select::SelectValues::from_json));
    reg.register("SortRows", Arc::new(sort::SortRows::from_json));
    reg.register("AddConstants", Arc::new(constants::AddConstants::from_json));
    reg.register("Deduplicate", Arc::new(deduplicate::Deduplicate::from_json));
    reg.register(
        "StreamLookup",
        Arc::new(stream_lookup::StreamLookup::from_json),
    );
    reg.register(
        "CalculatorStep",
        Arc::new(calculator::CalculatorStep::from_json),
    );
    reg.register("MergeJoin", Arc::new(merge_join::MergeJoin::from_json));
    reg.register("TableInput", Arc::new(db::input::TableInput::from_json));
    reg.register("TableOutput", Arc::new(db::output::TableOutput::from_json));
    reg.register(
        "DatabaseLookup",
        Arc::new(db::lookup::DatabaseLookup::from_json),
    );
    reg.register("IfNull", Arc::new(if_null::IfNull::from_json));
    reg.register(
        "StringOperations",
        Arc::new(string_ops::StringOperations::from_json),
    );
    reg.register(
        "ReplaceInString",
        Arc::new(string_ops::ReplaceInString::from_json),
    );
    reg.register(
        "ConcatFields",
        Arc::new(string_ops::ConcatFields::from_json),
    );
    reg.register(
        "SplitFieldToRows",
        Arc::new(split_field::SplitFieldToRows::from_json),
    );
    reg.register(
        "AppendStreams",
        Arc::new(append_streams::AppendStreams::from_json),
    );
    reg.register("WriteToLog", Arc::new(write_to_log::WriteToLog::from_json));
    reg.register(
        "GenerateRows",
        Arc::new(generate_rows::GenerateRows::from_json),
    );
    reg.register(
        "MemoryGroupBy",
        Arc::new(memory_group_by::MemoryGroupBy::from_json),
    );
    reg.register("SwitchCase", Arc::new(switch_case::SwitchCase::from_json));
    reg.register(
        "ExcelFileInput",
        Arc::new(excel_file_input::ExcelFileInput::from_json),
    );
    reg.register(
        "ExcelFileOutput",
        Arc::new(excel_file_output::ExcelFileOutput::from_json),
    );
    reg.register(
        "AddSequence",
        Arc::new(add_sequence::AddSequence::from_json),
    );
    reg.register(
        "SetVariable",
        Arc::new(set_variable::SetVariable::from_json),
    );
    reg.register(
        "GetVariable",
        Arc::new(get_variable::GetVariable::from_json),
    );
    reg.register(
        "XmlFileInput",
        Arc::new(xml_file_input::XmlFileInput::from_json),
    );
    reg.register(
        "XmlFileOutput",
        Arc::new(xml_file_output::XmlFileOutput::from_json),
    );
    reg.register("RestClient", Arc::new(rest_client::RestClient::from_json));
    reg.register(
        "ParquetFileInput",
        Arc::new(parquet_file_input::ParquetFileInput::from_json),
    );
    reg.register(
        "ParquetFileOutput",
        Arc::new(parquet_file_output::ParquetFileOutput::from_json),
    );
    reg.register(
        "RowNormaliser",
        Arc::new(row_normaliser::RowNormaliser::from_json),
    );
    reg.register(
        "RowDenormaliser",
        Arc::new(row_denormaliser::RowDenormaliser::from_json),
    );
    reg.register(
        "GetFileNames",
        Arc::new(get_file_names::GetFileNames::from_json),
    );
    reg.register(
        "LoadFileContent",
        Arc::new(load_file_content::LoadFileContent::from_json),
    );
    reg.register(
        "WriteToFile",
        Arc::new(write_to_file::WriteToFile::from_json),
    );
    reg.register("Dummy", Arc::new(dummy::Dummy::from_json));
    reg.register("Abort", Arc::new(abort::Abort::from_json));
    reg.register("RegexEval", Arc::new(regex_eval::RegexEval::from_json));
    reg.register("CloneRow", Arc::new(clone_row::CloneRow::from_json));
    reg.register(
        "FieldSplitter",
        Arc::new(field_splitter::FieldSplitter::from_json),
    );
    reg.register("UniqueRows", Arc::new(unique_rows::UniqueRows::from_json));
    reg.register(
        "NumberRange",
        Arc::new(number_range::NumberRange::from_json),
    );
    reg.register(
        "ValueMapper",
        Arc::new(value_mapper::ValueMapper::from_json),
    );
    reg.register("ExecuteSQL", Arc::new(execute_sql::ExecuteSQL::from_json));
    #[cfg(feature = "scripting")]
    reg.register("ScriptStep", Arc::new(script_step::ScriptStep::from_json));
    reg.register(
        "PipelineExecutor",
        Arc::new(pipeline_executor::PipelineExecutor::from_json),
    );
    reg.register(
        "JsonFieldInput",
        Arc::new(json_field_input::JsonFieldInput::from_json),
    );
    reg.register(
        "JsonFieldOutput",
        Arc::new(json_field_output::JsonFieldOutput::from_json),
    );
    reg.register(
        "AnalyticQuery",
        Arc::new(analytic_query::AnalyticQuery::from_json),
    );

    reg
}
