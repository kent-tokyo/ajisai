export type FieldType = 'string' | 'number' | 'boolean' | 'select' | 'textarea' | 'stringArray' | 'json'

export interface FormTemplate {
  name: string
  label: string
  description: string
  config: Record<string, unknown>
}

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
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      group: 'Output',
      validations: [
        { type: 'pattern', value: /\.csv$/i, message: 'Must be .csv file' }
      ],
      examples: ['output.csv', '/tmp/results.csv'],
    },
    {
      key: 'delimiter',
      label: 'Delimiter',
      type: 'string',
      default: ',',
      placeholder: ',',
      group: 'Format',
      validations: [
        { type: 'maxLength', value: 1 }
      ],
      examples: [',', '|', ';'],
    },
    {
      key: 'header',
      label: 'Write header row',
      type: 'boolean',
      default: true,
      group: 'Format',
    },
  ],

  JsonFileInput: [
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      group: 'File',
      validations: [
        { type: 'pattern', value: /\.json(l)?$/i, message: 'Must be .json or .jsonl file' }
      ],
      examples: ['data.json', 'records.jsonl'],
    },
    {
      key: 'format',
      label: 'Format',
      type: 'select',
      options: ['array', 'jsonl'],
      default: 'array',
      group: 'Format',
      help: 'array: [{...}, {...}] | jsonl: one JSON object per line',
    },
  ],

  JsonFileOutput: [
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      group: 'File',
      examples: ['output.json', 'results.jsonl'],
    },
    {
      key: 'format',
      label: 'Format',
      type: 'select',
      options: ['array', 'jsonl'],
      default: 'array',
      group: 'Format',
    },
    {
      key: 'pretty',
      label: 'Pretty print',
      type: 'boolean',
      default: true,
      group: 'Format',
      help: 'Indent JSON for readability',
    },
  ],

  ExcelFileInput: [
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      group: 'File',
      validations: [
        { type: 'pattern', value: /\.xlsx?$/i, message: 'Must be .xls or .xlsx file' }
      ],
      examples: ['data.xlsx', 'report.xls'],
    },
    {
      key: 'sheet_name',
      label: 'Sheet name',
      type: 'string',
      group: 'Options',
      placeholder: 'Sheet1',
      examples: ['Sheet1', 'Data', 'Sales'],
    },
    {
      key: 'has_header',
      label: 'First row is header',
      type: 'boolean',
      default: true,
      group: 'Options',
    },
  ],

  ExcelFileOutput: [
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      group: 'File',
      examples: ['output.xlsx'],
    },
    {
      key: 'sheet_name',
      label: 'Sheet name',
      type: 'string',
      default: 'Sheet1',
      group: 'Options',
      placeholder: 'Sheet1',
    },
    {
      key: 'header',
      label: 'Write header row',
      type: 'boolean',
      default: true,
      group: 'Options',
    },
  ],

  ParquetFileInput: [
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      validations: [
        { type: 'pattern', value: /\.parquet$/i, message: 'Must be .parquet file' }
      ],
      examples: ['data.parquet', 'large_dataset.parquet'],
      help: 'Columnar binary format for big data',
    },
  ],

  ParquetFileOutput: [
    {
      key: 'filename',
      label: 'File path',
      type: 'string',
      required: true,
      group: 'File',
      examples: ['output.parquet'],
    },
    {
      key: 'compression',
      label: 'Compression',
      type: 'select',
      options: ['uncompressed', 'snappy', 'gzip'],
      default: 'snappy',
      group: 'Options',
      help: 'snappy: fast; gzip: best compression',
    },
  ],

  XmlFileInput: [
    { key: 'filename', label: 'File path', type: 'string', required: true },
    { key: 'row_element', label: 'Row element', type: 'string' },
  ],

  TableInput: [
    {
      key: 'connection_url',
      label: 'Connection URL',
      type: 'string',
      required: true,
      group: 'Connection',
      placeholder: 'sqlite://data.db',
      validations: [
        { type: 'pattern', value: /^(sqlite|postgresql|mysql|sql):\/\//, message: 'Invalid connection URL' }
      ],
      examples: ['sqlite://data.db', 'postgresql://user:pass@localhost/db', 'mysql://root:pass@localhost/mydb'],
    },
    {
      key: 'sql',
      label: 'SQL Query',
      type: 'textarea',
      rows: 6,
      required: true,
      group: 'Query',
      examples: [
        'SELECT * FROM users WHERE active = 1',
        'SELECT id, name, email FROM customers LIMIT 1000',
      ],
      help: 'SQL SELECT statement',
    },
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
    {
      key: 'url',
      label: 'URL',
      type: 'string',
      required: true,
      group: 'Request',
      examples: ['https://api.example.com/data', 'http://localhost:8080/users'],
      validations: [
        { type: 'pattern', value: /^https?:\/\//, message: 'Must start with http:// or https://' }
      ],
    },
    {
      key: 'method',
      label: 'HTTP Method',
      type: 'select',
      options: ['GET', 'POST', 'PUT', 'DELETE'],
      default: 'GET',
      group: 'Request',
    },
    {
      key: 'headers',
      label: 'Headers (JSON)',
      type: 'json',
      group: 'Options',
      dependsOn: { field: 'method', value: 'POST' },
      examples: ['{"Content-Type": "application/json", "Authorization": "Bearer token"}'],
      help: 'Request headers (shown for POST method only)',
    },
    {
      key: 'body',
      label: 'Request Body',
      type: 'textarea',
      rows: 4,
      group: 'Options',
      dependsOn: { field: 'method', value: 'POST' },
      help: 'Request payload (shown for POST method only)',
    },
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
    {
      key: 'fields',
      label: 'Fields to select (JSON)',
      type: 'json',
      examples: [
        '["id", "name", "email"]',
        '[{"name": "id"}, {"name": "first_name", "alias": "firstName"}]',
      ],
      help: 'Array of field names or field objects with optional alias',
    },
  ],

  SortRows: [
    {
      key: 'keys',
      label: 'Sort keys (JSON)',
      type: 'json',
      required: true,
      examples: [
        '[{"field": "age", "order": "asc"}]',
        '[{"field": "name", "order": "asc"}, {"field": "date", "order": "desc"}]',
      ],
      help: 'Sort by field(s): [{"field": "name", "order": "asc|desc"}]',
      validations: [
        { type: 'custom', fn: (v) => {
          try { const arr = JSON.parse(v); return Array.isArray(arr); } catch { return false; }
        }, message: 'Must be valid JSON array' }
      ],
    },
  ],

  AddConstants: [
    { key: 'fields', label: 'Fields (JSON)', type: 'json' },
  ],

  AddSequence: [
    {
      key: 'field_name',
      label: 'Field name',
      type: 'string',
      required: true,
      group: 'Sequence',
      validations: [
        { type: 'pattern', value: /^[a-zA-Z_][a-zA-Z0-9_]*$/, message: 'Invalid field name' }
      ],
      examples: ['id', 'row_number', 'sequence'],
    },
    {
      key: 'start',
      label: 'Start value',
      type: 'number',
      default: 1,
      group: 'Sequence',
      validations: [
        { type: 'min', value: 0 }
      ],
    },
  ],

  CalculatorStep: [
    {
      key: 'calculations',
      label: 'Calculations (JSON)',
      type: 'json',
      examples: [
        '[{"output_field": "total", "formula": "price * quantity"}]',
        '[{"output_field": "age", "formula": "year(now()) - birth_year"}]',
      ],
      help: 'Array of {output_field, formula} objects with math expressions',
    },
  ],

  Deduplicate: [
    {
      key: 'key_fields',
      label: 'Key fields (JSON)',
      type: 'json',
      examples: [
        '["id"]',
        '["user_id", "email"]',
      ],
      help: 'Deduplicate based on these fields (keeps first occurrence)',
    },
  ],

  IfNull: [
    { key: 'field', label: 'Field', type: 'string', required: true },
    { key: 'default_value', label: 'Default value', type: 'string' },
  ],

  StringOperations: [
    {
      key: 'operations',
      label: 'Operations (JSON)',
      type: 'json',
      examples: [
        '[{"field": "name", "operation": "upper", "output": "name_upper"}]',
        '[{"field": "text", "operation": "trim"}, {"field": "text", "operation": "lower", "output": "text_lower"}]',
      ],
      help: 'Operations: upper, lower, trim, length, substring, etc.',
    },
  ],

  ReplaceInString: [
    {
      key: 'replacements',
      label: 'Replacements (JSON)',
      type: 'json',
      examples: [
        '[{"field": "email", "from": "@oldomain.com", "to": "@newdomain.com"}]',
      ],
      help: 'Replace patterns: [{field, from, to}, ...]',
    },
  ],

  ConcatFields: [
    {
      key: 'input_fields',
      label: 'Input fields (JSON)',
      type: 'json',
      group: 'Fields',
      required: true,
      examples: ['["first_name", "last_name"]'],
      help: 'Fields to concatenate',
    },
    {
      key: 'separator',
      label: 'Separator',
      type: 'string',
      default: ' ',
      group: 'Format',
      examples: [' ', ', ', '-'],
    },
    {
      key: 'output_field',
      label: 'Output field',
      type: 'string',
      required: true,
      group: 'Output',
      examples: ['full_name'],
    },
  ],

  SplitFieldToRows: [
    {
      key: 'field',
      label: 'Field to split',
      type: 'string',
      required: true,
      group: 'Input',
      help: 'Field containing delimited values',
    },
    {
      key: 'delimiter',
      label: 'Delimiter',
      type: 'string',
      default: ',',
      group: 'Options',
      examples: [',', '|', ';'],
    },
  ],

  MemoryGroupBy: [
    {
      key: 'group_fields',
      label: 'Group by (JSON)',
      type: 'json',
      group: 'Grouping',
      examples: [
        '["category"]',
        '["department", "year"]',
      ],
      help: 'Fields to group by',
    },
    {
      key: 'aggregations',
      label: 'Aggregations (JSON)',
      type: 'json',
      group: 'Aggregation',
      examples: [
        '[{"field": "amount", "aggregation": "sum", "output": "total"}]',
        '[{"field": "id", "aggregation": "count", "output": "count"}, {"field": "salary", "aggregation": "avg", "output": "avg_salary"}]',
      ],
      help: 'Array of {field, aggregation, output} — supported: sum, count, avg, min, max',
    },
  ],

  AppendStreams: [
    { key: 'description', label: 'Note', type: 'string', placeholder: 'Multiple inputs will be merged' },
  ],

  RowNormaliser: [
    {
      key: 'value_field',
      label: 'Value field',
      type: 'string',
      help: 'Field containing key-value pairs to normalize',
    },
  ],

  RowDenormaliser: [
    {
      key: 'key_fields',
      label: 'Key fields (JSON)',
      type: 'json',
      examples: ['["id", "date"]'],
      help: 'Group by these fields before denormalizing',
    },
  ],

  WriteToLog: [
    {
      key: 'level',
      label: 'Log level',
      type: 'select',
      options: ['info', 'warn', 'error'],
      default: 'info',
      group: 'Logging',
    },
    {
      key: 'message',
      label: 'Message template',
      type: 'string',
      group: 'Logging',
      examples: ['Processing row: {id}', 'Error: {error_message}'],
      help: 'Template with {field} placeholders',
    },
  ],

  CloneRow: [
    { key: 'num_copies', label: 'Number of copies', type: 'number', required: true },
  ],

  FieldSplitter: [
    {
      key: 'field',
      label: 'Field to split',
      type: 'string',
      required: true,
      help: 'Field containing delimited values to split into multiple columns',
    },
    {
      key: 'delimiter',
      label: 'Delimiter',
      type: 'string',
      default: ',',
      examples: [',', '|', ':'],
    },
  ],

  UniqueRows: [
    {
      key: 'key_fields',
      label: 'Key fields (JSON)',
      type: 'json',
      examples: ['["id"]', '["customer_id", "product_id"]'],
      help: 'Unique rows based on these fields (keeps first)',
    },
  ],

  NumberRange: [
    {
      key: 'field',
      label: 'Field',
      type: 'string',
      required: true,
      help: 'Numeric field to bin into ranges',
    },
    {
      key: 'ranges',
      label: 'Ranges (JSON)',
      type: 'json',
      examples: [
        '[{"min": 0, "max": 18, "label": "child"}, {"min": 18, "max": 65, "label": "adult"}]',
      ],
      help: 'Range bins with min, max, label',
    },
  ],

  ValueMapper: [
    {
      key: 'mappings',
      label: 'Mappings (JSON)',
      type: 'json',
      examples: [
        '[{"field": "status", "mappings": {"A": "Active", "I": "Inactive"}}]',
      ],
      help: 'Map values: [{field, mappings: {old: new}}]',
    },
  ],

  ExecuteSQL: [
    {
      key: 'sql',
      label: 'SQL statement',
      type: 'textarea',
      rows: 8,
      required: true,
      group: 'SQL',
      examples: [
        'UPDATE users SET status = \'active\' WHERE created_at > NOW() - INTERVAL \'7 days\'',
      ],
      help: 'DML statement (INSERT, UPDATE, DELETE)',
    },
    {
      key: 'connection_url',
      label: 'Connection URL',
      type: 'string',
      group: 'Connection',
      examples: ['postgresql://localhost/mydb'],
    },
  ],

  Dummy: [
    { key: 'description', label: 'Note', type: 'string', placeholder: 'Pass-through transform' },
  ],

  Abort: [
    {
      key: 'condition',
      label: 'Abort condition (JSON)',
      type: 'json',
      examples: ['{"field": "error_count", "op": ">", "value": 10}'],
      help: 'Condition to abort pipeline execution',
    },
  ],

  RegexEval: [
    {
      key: 'field',
      label: 'Field',
      type: 'string',
      required: true,
      group: 'Regex',
      help: 'Field to match regex against',
    },
    {
      key: 'pattern',
      label: 'Regex pattern',
      type: 'string',
      required: true,
      group: 'Regex',
      examples: ['^[A-Z][a-z]+$', '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}'],
    },
    {
      key: 'output_field',
      label: 'Output field',
      type: 'string',
      group: 'Output',
      help: 'Boolean field for match result',
    },
  ],

  ScriptStep: [
    {
      key: 'script',
      label: 'Rhai script',
      type: 'textarea',
      rows: 10,
      required: true,
      group: 'Script',
      examples: [
        'let sum = field1 + field2;\nreturn {field3: sum};',
      ],
      help: 'Rhai script: access fields as variables, return object',
    },
    {
      key: 'output_fields',
      label: 'Output fields (JSON)',
      type: 'json',
      group: 'Output',
      examples: ['["result", "status"]'],
      help: 'Fields defined in script output',
    },
  ],

  // === Join / Lookup ===
  StreamLookup: [
    {
      key: 'lookup_transform',
      label: 'Lookup transform ID',
      type: 'string',
      required: true,
      group: 'Lookup',
      help: 'ID of the lookup node (use node label)',
    },
    {
      key: 'key_field',
      label: 'Key field',
      type: 'string',
      required: true,
      group: 'Keys',
      help: 'Field in main stream to match on',
    },
    {
      key: 'lookup_key_field',
      label: 'Lookup key field',
      type: 'string',
      required: true,
      group: 'Keys',
      help: 'Field in lookup stream to match against',
    },
    {
      key: 'return_fields',
      label: 'Return fields (JSON)',
      type: 'json',
      group: 'Output',
      examples: ['["user_id", "email", "phone"]'],
      help: 'Fields to return from lookup',
    },
  ],

  MergeJoin: [
    {
      key: 'left_key',
      label: 'Left key field',
      type: 'string',
      required: true,
      group: 'Join Keys',
    },
    {
      key: 'right_key',
      label: 'Right key field',
      type: 'string',
      required: true,
      group: 'Join Keys',
    },
    {
      key: 'join_type',
      label: 'Join type',
      type: 'select',
      options: ['inner', 'left', 'right', 'full'],
      default: 'inner',
      group: 'Options',
      help: 'inner: matching only; left: all left rows; right: all right rows; full: all rows',
    },
    {
      key: 'right_prefix',
      label: 'Right prefix',
      type: 'string',
      placeholder: 'r_',
      group: 'Options',
      default: 'r_',
      help: 'Prefix for right table column names (avoid conflicts)',
    },
  ],

  DatabaseLookup: [
    {
      key: 'connection_url',
      label: 'Connection URL',
      type: 'string',
      required: true,
      group: 'Connection',
      examples: ['postgresql://user:pass@localhost/db'],
    },
    {
      key: 'sql',
      label: 'SQL query',
      type: 'textarea',
      rows: 6,
      required: true,
      group: 'Query',
      examples: ['SELECT id, name, email FROM users WHERE id = ?'],
      help: 'Use ? for key_field placeholder',
    },
    {
      key: 'key_field',
      label: 'Key field',
      type: 'string',
      required: true,
      group: 'Mapping',
      help: 'Main stream field to lookup (replaces ? in SQL)',
    },
    {
      key: 'return_fields',
      label: 'Return fields (JSON)',
      type: 'json',
      group: 'Output',
      examples: ['["name", "email", "phone"]'],
    },
  ],

  // === Variables / Flow ===
  SetVariable: [
    {
      key: 'variable_name',
      label: 'Variable name',
      type: 'string',
      required: true,
      validations: [
        { type: 'pattern', value: /^[a-zA-Z_][a-zA-Z0-9_]*$/, message: 'Invalid variable name' }
      ],
      examples: ['total_rows', 'start_date'],
    },
    {
      key: 'variable_value_field',
      label: 'Value field',
      type: 'string',
      required: true,
      help: 'Field value to store in variable',
    },
  ],

  GetVariable: [
    {
      key: 'variable_name',
      label: 'Variable name',
      type: 'string',
      required: true,
      validations: [
        { type: 'pattern', value: /^[a-zA-Z_][a-zA-Z0-9_]*$/, message: 'Invalid variable name' }
      ],
    },
    {
      key: 'output_field',
      label: 'Output field',
      type: 'string',
      required: true,
      help: 'Field to store retrieved variable value',
    },
  ],

  SwitchCase: [
    {
      key: 'field',
      label: 'Field to switch on',
      type: 'string',
      required: true,
      group: 'Switch',
      help: 'Field value determines which output hop',
    },
    {
      key: 'cases',
      label: 'Cases (JSON)',
      type: 'json',
      required: true,
      group: 'Cases',
      examples: [
        '[{"value": "A", "target_hop": "hop_a"}, {"value": "B", "target_hop": "hop_b"}]',
      ],
      help: 'Cases with value and target_hop',
    },
  ],
}

// === FORM TEMPLATES (よく使う設定プリセット) ===

export const FORM_TEMPLATES: Record<string, FormTemplate[]> = {
  CsvFileInput: [
    {
      name: 'csv_default',
      label: 'CSV (Default)',
      description: 'Standard CSV with header',
      config: { delimiter: ',', has_header: true, encoding: 'UTF-8' },
    },
    {
      name: 'csv_pipe',
      label: 'CSV (Pipe-delimited)',
      description: 'Pipe-separated values',
      config: { delimiter: '|', has_header: true, encoding: 'UTF-8' },
    },
    {
      name: 'csv_tab',
      label: 'CSV (Tab-delimited)',
      description: 'Tab-separated values',
      config: { delimiter: '\t', has_header: true, encoding: 'UTF-8' },
    },
  ],

  RestClient: [
    {
      name: 'rest_get_json',
      label: 'GET JSON',
      description: 'Simple GET request',
      config: { method: 'GET' },
    },
    {
      name: 'rest_post_json',
      label: 'POST JSON',
      description: 'POST with JSON headers and body',
      config: {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      },
    },
  ],

  TableOutput: [
    {
      name: 'table_insert',
      label: 'Insert mode',
      description: 'Append rows to table',
      config: { mode: 'insert', batch_size: 1000 },
    },
    {
      name: 'table_upsert',
      label: 'Upsert mode',
      description: 'Update if exists, insert otherwise',
      config: { mode: 'upsert', batch_size: 1000 },
    },
    {
      name: 'table_replace',
      label: 'Replace mode',
      description: 'Truncate and rewrite',
      config: { mode: 'replace', batch_size: 5000 },
    },
  ],

  FilterRows: [
    {
      name: 'filter_equals',
      label: 'Equals filter',
      description: 'Filter by exact value match',
      config: { condition: '{"field": "status", "op": "==", "value": "active"}' },
    },
    {
      name: 'filter_range',
      label: 'Range filter',
      description: 'Filter by numeric range',
      config: { condition: '{"field": "age", "op": ">=", "value": 18}' },
    },
  ],

  JoinTwoInputs: [
    {
      name: 'join_inner',
      label: 'Inner join',
      description: 'Only matching rows',
      config: { join_type: 'inner' },
    },
    {
      name: 'join_left',
      label: 'Left join',
      description: 'All left rows + matches',
      config: { join_type: 'left' },
    },
  ],
}
