export type FieldType = 'string' | 'number' | 'boolean' | 'select' | 'textarea' | 'stringArray' | 'json'

export type ValidationRule =
  | { type: 'pattern', value: RegExp, message: string }
  | { type: 'minLength', value: number, message?: string }
  | { type: 'maxLength', value: number, message?: string }
  | { type: 'min', value: number, message?: string }
  | { type: 'max', value: number, message?: string }
  | { type: 'enum', value: string[], message?: string }
  | { type: 'custom', fn: (value: any) => boolean, message: string }

export interface FormField {
  key: string
  label: string
  type: FieldType
  required?: boolean
  default?: any
  options?: string[]
  placeholder?: string
  rows?: number
  help?: string
  group?: string
  validations?: ValidationRule[]
  dependsOn?: { field: string, value: any }
  examples?: string[]
}

export const FORM_DESCRIPTORS: Record<string, FormField[]> = {
  // === I/O ===
  CsvFileInput: [
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      group: 'File',
      validations: [
        { type: 'pattern', value: /\.(csv|txt)$/i, message: 'Must be .csv or .txt file' }
      ],
      examples: ['data.csv', '/home/user/input.txt'],
      help: 'Path to CSV file (absolute or relative)',
    },
    {
      key: 'delimiter',
      label: 'Delimiter',
      type: 'string',
      default: ',',
      placeholder: ',',
      group: 'Options',
      validations: [
        { type: 'maxLength', value: 1, message: 'Delimiter must be single character' }
      ],
      examples: [',', '|', '\t'],
    },
    {
      key: 'has_header',
      label: 'Has header row',
      type: 'boolean',
      default: true,
      group: 'Options',
      help: 'First row contains column names',
    },
    {
      key: 'encoding',
      label: 'Encoding',
      type: 'string',
      default: 'UTF-8',
      placeholder: 'UTF-8',
      group: 'Options',
      examples: ['UTF-8', 'ISO-8859-1', 'Windows-1252'],
    },
  ],

  CsvFileOutput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'delimiter', label: 'Delimiter', type: 'string', placeholder: ',' },
    { key: 'header', label: 'Write header', type: 'boolean' },
  ],

  JsonFileInput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'format', label: 'Format', type: 'select', options: ['array', 'jsonl'] },
  ],

  JsonFileOutput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'format', label: 'Format', type: 'select', options: ['array', 'jsonl'] },
    { key: 'pretty', label: 'Pretty print', type: 'boolean' },
  ],

  ExcelFileInput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'sheet_name', label: 'Sheet name', type: 'string' },
    { key: 'has_header', label: 'Has header', type: 'boolean' },
  ],

  ExcelFileOutput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'sheet_name', label: 'Sheet name', type: 'string' },
    { key: 'header', label: 'Write header', type: 'boolean' },
  ],

  ParquetFileInput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
  ],

  ParquetFileOutput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'compression', label: 'Compression', type: 'select', options: ['uncompressed', 'snappy', 'gzip'] },
  ],

  XmlFileInput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'row_element', label: 'Row element', type: 'string' },
  ],

  TableInput: [
    { key: 'connection_url', label: 'Connection URL', type: 'string', required: true, placeholder: 'sqlite://data.db' },
    { key: 'sql', label: 'SQL Query', type: 'textarea', rows: 4, required: true },
  ],

  TableOutput: [
    {
      key: 'connection_url',
      label: 'Connection URL',
      type: 'string',
      required: true,
      group: 'Connection',
      examples: ['sqlite://data.db', 'postgresql://user:pass@localhost/db'],
      help: 'Database connection string',
    },
    {
      key: 'table',
      label: 'Table name',
      type: 'string',
      required: true,
      group: 'Target',
      validations: [
        { type: 'pattern', value: /^[a-zA-Z_][a-zA-Z0-9_]*$/, message: 'Invalid table name (must start with letter/underscore)' }
      ],
      examples: ['users', 'order_items'],
    },
    {
      key: 'mode',
      label: 'Write mode',
      type: 'select',
      options: ['insert', 'upsert', 'replace'],
      default: 'insert',
      group: 'Options',
      help: 'insert: append rows; upsert: update if exists; replace: truncate & rewrite',
    },
    {
      key: 'batch_size',
      label: 'Batch size',
      type: 'number',
      default: 1000,
      group: 'Options',
      validations: [
        { type: 'min', value: 1, message: 'Batch size must be >= 1' },
        { type: 'max', value: 100000, message: 'Batch size must be <= 100000' }
      ],
    },
  ],

  GenerateRows: [
    { key: 'num_rows', label: 'Number of rows', type: 'number', required: true },
  ],

  RestClient: [
    { key: 'url', label: 'URL', type: 'string', required: true },
    { key: 'method', label: 'Method', type: 'select', options: ['GET', 'POST', 'PUT', 'DELETE'] },
    { key: 'headers', label: 'Headers (JSON)', type: 'json' },
    { key: 'body', label: 'Body', type: 'textarea', rows: 4 },
  ],

  GetFileNames: [
    { key: 'directory', label: 'Directory', type: 'string', required: true },
    { key: 'pattern', label: 'File pattern', type: 'string', placeholder: '*.csv' },
  ],

  LoadFileContent: [
    { key: 'filename_field', label: 'Filename field', type: 'string', required: true },
    { key: 'output_field', label: 'Output field', type: 'string', required: true },
  ],

  WriteToFile: [
    { key: 'filename_field', label: 'Filename field', type: 'string', required: true },
    { key: 'content_field', label: 'Content field', type: 'string', required: true },
  ],

  PipelineExecutor: [
    { key: 'filepath', label: 'Pipeline file path', type: 'string', required: true },
  ],

  // === Transform ===
  FilterRows: [
    {
      key: 'condition',
      label: 'Condition (JSON)',
      type: 'json',
      group: 'Filtering',
      required: true,
      examples: [
        '{"field": "age", "op": ">", "value": 18}',
        '{"and": [{"field": "status", "op": "==", "value": "active"}, {"field": "score", "op": ">=", "value": 70}]}',
      ],
      help: 'Filter expression: {"field": "name", "op": "contains", "value": "pattern"}',
      validations: [
        { type: 'custom', fn: (v) => {
          try { JSON.parse(v); return true; } catch { return false; }
        }, message: 'Must be valid JSON' }
      ],
    },
  ],

  SelectValues: [
    { key: 'fields', label: 'Fields (JSON)', type: 'json' },
  ],

  SortRows: [
    { key: 'keys', label: 'Sort keys (JSON)', type: 'json', required: true },
  ],

  AddConstants: [
    { key: 'fields', label: 'Fields (JSON)', type: 'json' },
  ],

  AddSequence: [
    { key: 'field_name', label: 'Field name', type: 'string', required: true },
    { key: 'start', label: 'Start value', type: 'number' },
  ],

  CalculatorStep: [
    { key: 'calculations', label: 'Calculations (JSON)', type: 'json' },
  ],

  Deduplicate: [
    { key: 'key_fields', label: 'Key fields (JSON)', type: 'json' },
  ],

  IfNull: [
    { key: 'field', label: 'Field', type: 'string', required: true },
    { key: 'default_value', label: 'Default value', type: 'string' },
  ],

  StringOperations: [
    { key: 'operations', label: 'Operations (JSON)', type: 'json' },
  ],

  ReplaceInString: [
    { key: 'replacements', label: 'Replacements (JSON)', type: 'json' },
  ],

  ConcatFields: [
    { key: 'input_fields', label: 'Input fields (JSON)', type: 'json' },
    { key: 'separator', label: 'Separator', type: 'string' },
    { key: 'output_field', label: 'Output field', type: 'string', required: true },
  ],

  SplitFieldToRows: [
    { key: 'field', label: 'Field', type: 'string', required: true },
    { key: 'delimiter', label: 'Delimiter', type: 'string' },
  ],

  MemoryGroupBy: [
    { key: 'group_fields', label: 'Group by (JSON)', type: 'json' },
    { key: 'aggregations', label: 'Aggregations (JSON)', type: 'json' },
  ],

  AppendStreams: [
    { key: 'description', label: 'Note', type: 'string', placeholder: 'Multiple inputs will be merged' },
  ],

  RowNormaliser: [
    { key: 'value_field', label: 'Value field', type: 'string' },
  ],

  RowDenormaliser: [
    { key: 'key_fields', label: 'Key fields (JSON)', type: 'json' },
  ],

  WriteToLog: [
    { key: 'level', label: 'Log level', type: 'select', options: ['info', 'warn', 'error'] },
    { key: 'message', label: 'Message template', type: 'string' },
  ],

  CloneRow: [
    { key: 'num_copies', label: 'Number of copies', type: 'number', required: true },
  ],

  FieldSplitter: [
    { key: 'field', label: 'Field to split', type: 'string', required: true },
    { key: 'delimiter', label: 'Delimiter', type: 'string' },
  ],

  UniqueRows: [
    { key: 'key_fields', label: 'Key fields (JSON)', type: 'json' },
  ],

  NumberRange: [
    { key: 'field', label: 'Field', type: 'string', required: true },
    { key: 'ranges', label: 'Ranges (JSON)', type: 'json' },
  ],

  ValueMapper: [
    { key: 'mappings', label: 'Mappings (JSON)', type: 'json' },
  ],

  ExecuteSQL: [
    { key: 'sql', label: 'SQL statement', type: 'textarea', rows: 6, required: true },
    { key: 'connection_url', label: 'Connection URL', type: 'string' },
  ],

  Dummy: [
    { key: 'description', label: 'Note', type: 'string', placeholder: 'Pass-through transform' },
  ],

  Abort: [
    { key: 'condition', label: 'Condition (JSON)', type: 'json' },
  ],

  RegexEval: [
    { key: 'field', label: 'Field', type: 'string', required: true },
    { key: 'pattern', label: 'Regex pattern', type: 'string', required: true },
    { key: 'output_field', label: 'Output field', type: 'string' },
  ],

  ScriptStep: [
    { key: 'script', label: 'Rhai script', type: 'textarea', rows: 8, required: true },
    { key: 'output_fields', label: 'Output fields (JSON)', type: 'json' },
  ],

  // === Join / Lookup ===
  StreamLookup: [
    { key: 'lookup_transform', label: 'Lookup transform ID', type: 'string', required: true },
    { key: 'key_field', label: 'Key field', type: 'string', required: true },
    { key: 'lookup_key_field', label: 'Lookup key field', type: 'string', required: true },
    { key: 'return_fields', label: 'Return fields (JSON)', type: 'json' },
  ],

  MergeJoin: [
    { key: 'left_key', label: 'Left key field', type: 'string', required: true },
    { key: 'right_key', label: 'Right key field', type: 'string', required: true },
    { key: 'join_type', label: 'Join type', type: 'select', options: ['inner', 'left', 'right', 'full'] },
    { key: 'right_prefix', label: 'Right prefix', type: 'string', placeholder: 'r_' },
  ],

  DatabaseLookup: [
    { key: 'connection_url', label: 'Connection URL', type: 'string', required: true },
    { key: 'sql', label: 'SQL query', type: 'textarea', rows: 4, required: true },
    { key: 'key_field', label: 'Key field', type: 'string', required: true },
    { key: 'return_fields', label: 'Return fields (JSON)', type: 'json' },
  ],

  // === Variables / Flow ===
  SetVariable: [
    { key: 'variable_name', label: 'Variable name', type: 'string', required: true },
    { key: 'variable_value_field', label: 'Value field', type: 'string', required: true },
  ],

  GetVariable: [
    { key: 'variable_name', label: 'Variable name', type: 'string', required: true },
    { key: 'output_field', label: 'Output field', type: 'string', required: true },
  ],

  SwitchCase: [
    { key: 'field', label: 'Field to switch on', type: 'string', required: true },
    { key: 'cases', label: 'Cases (JSON)', type: 'json', required: true },
  ],
}
