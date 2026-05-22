pub mod abort;
pub mod add_sequence;
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
pub mod analytic_query;
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
pub mod script_step;
pub mod select;
pub mod set_variable;
pub mod sort;
pub mod split_field;
pub mod stream_lookup;
pub mod string_ops;
pub mod switch_case;
pub mod unique_rows;
mod utils;
pub mod value_mapper;
pub mod write_to_file;
pub mod write_to_log;
pub mod xml_file_input;
pub mod xml_file_output;

pub use abort::Abort;
pub use add_sequence::AddSequence;
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
pub use analytic_query::AnalyticQuery;
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
    reg.register(
        "AppendStreams",
        Arc::new(|v| append_streams::AppendStreams::from_json(v)),
    );
    reg.register(
        "WriteToLog",
        Arc::new(|v| write_to_log::WriteToLog::from_json(v)),
    );
    reg.register(
        "GenerateRows",
        Arc::new(|v| generate_rows::GenerateRows::from_json(v)),
    );
    reg.register(
        "MemoryGroupBy",
        Arc::new(|v| memory_group_by::MemoryGroupBy::from_json(v)),
    );
    reg.register(
        "SwitchCase",
        Arc::new(|v| switch_case::SwitchCase::from_json(v)),
    );
    reg.register(
        "ExcelFileInput",
        Arc::new(|v| excel_file_input::ExcelFileInput::from_json(v)),
    );
    reg.register(
        "ExcelFileOutput",
        Arc::new(|v| excel_file_output::ExcelFileOutput::from_json(v)),
    );
    reg.register(
        "AddSequence",
        Arc::new(|v| add_sequence::AddSequence::from_json(v)),
    );
    reg.register(
        "SetVariable",
        Arc::new(|v| set_variable::SetVariable::from_json(v)),
    );
    reg.register(
        "GetVariable",
        Arc::new(|v| get_variable::GetVariable::from_json(v)),
    );
    reg.register(
        "XmlFileInput",
        Arc::new(|v| xml_file_input::XmlFileInput::from_json(v)),
    );
    reg.register(
        "XmlFileOutput",
        Arc::new(|v| xml_file_output::XmlFileOutput::from_json(v)),
    );
    reg.register(
        "RestClient",
        Arc::new(|v| rest_client::RestClient::from_json(v)),
    );
    reg.register(
        "ParquetFileInput",
        Arc::new(|v| parquet_file_input::ParquetFileInput::from_json(v)),
    );
    reg.register(
        "ParquetFileOutput",
        Arc::new(|v| parquet_file_output::ParquetFileOutput::from_json(v)),
    );
    reg.register(
        "RowNormaliser",
        Arc::new(|v| row_normaliser::RowNormaliser::from_json(v)),
    );
    reg.register(
        "RowDenormaliser",
        Arc::new(|v| row_denormaliser::RowDenormaliser::from_json(v)),
    );
    reg.register(
        "GetFileNames",
        Arc::new(|v| get_file_names::GetFileNames::from_json(v)),
    );
    reg.register(
        "LoadFileContent",
        Arc::new(|v| load_file_content::LoadFileContent::from_json(v)),
    );
    reg.register(
        "WriteToFile",
        Arc::new(|v| write_to_file::WriteToFile::from_json(v)),
    );
    reg.register("Dummy", Arc::new(|v| dummy::Dummy::from_json(v)));
    reg.register("Abort", Arc::new(|v| abort::Abort::from_json(v)));
    reg.register(
        "RegexEval",
        Arc::new(|v| regex_eval::RegexEval::from_json(v)),
    );
    reg.register("CloneRow", Arc::new(|v| clone_row::CloneRow::from_json(v)));
    reg.register(
        "FieldSplitter",
        Arc::new(|v| field_splitter::FieldSplitter::from_json(v)),
    );
    reg.register(
        "UniqueRows",
        Arc::new(|v| unique_rows::UniqueRows::from_json(v)),
    );
    reg.register(
        "NumberRange",
        Arc::new(|v| number_range::NumberRange::from_json(v)),
    );
    reg.register(
        "ValueMapper",
        Arc::new(|v| value_mapper::ValueMapper::from_json(v)),
    );
    reg.register(
        "ExecuteSQL",
        Arc::new(|v| execute_sql::ExecuteSQL::from_json(v)),
    );
    reg.register(
        "ScriptStep",
        Arc::new(|v| script_step::ScriptStep::from_json(v)),
    );
    reg.register(
        "PipelineExecutor",
        Arc::new(|v| pipeline_executor::PipelineExecutor::from_json(v)),
    );
    reg.register(
        "JsonFieldInput",
        Arc::new(|v| json_field_input::JsonFieldInput::from_json(v)),
    );
    reg.register(
        "JsonFieldOutput",
        Arc::new(|v| json_field_output::JsonFieldOutput::from_json(v)),
    );
    reg.register(
        "AnalyticQuery",
        Arc::new(|v| analytic_query::AnalyticQuery::from_json(v)),
    );

    reg
}
