import { useState, useEffect, useCallback } from 'react'
import { FORM_DESCRIPTORS, FORM_TEMPLATES, type FormField } from '../config/formDescriptors'
import { useFormHistory } from '../hooks/useFormHistory'
import type { Node as PipelineNode } from '../types/pipeline'
import './ConfigForm.css'

interface ConfigFormProps {
  node: PipelineNode | null
  onConfigChange: (id: string, config: Record<string, unknown>) => void
}

export function ConfigForm({ node, onConfigChange }: ConfigFormProps) {
  const [jsonText, setJsonText] = useState(() =>
    node ? JSON.stringify(node.config, null, 2) : '{}'
  )
  const [error, setError] = useState<string>('')
  const [showHistory, setShowHistory] = useState(false)
  const { addSnapshot, getHistoryFor } = useFormHistory()

  // Update JSON text when node changes
  useEffect(() => {
    if (node) {
      setJsonText(JSON.stringify(node.config, null, 2))
      // Save to history when config is set
      addSnapshot(node.type_name, node.config)
    }
  }, [node?.id])

  const fields = node ? FORM_DESCRIPTORS[node.type_name] : null
  const templates = node ? FORM_TEMPLATES[node.type_name] : null
  const history = node ? getHistoryFor(node.type_name) : []

  const handleFieldChange = useCallback(
    (key: string, value: unknown) => {
      if (!node) return
      const newConfig = { ...node.config, [key]: value }
      onConfigChange(node.id, newConfig)
      // Also update JSON text for display
      setJsonText(JSON.stringify(newConfig, null, 2))
    },
    [node, onConfigChange]
  )

  const applyTemplate = useCallback(
    (templateConfig: Record<string, unknown>) => {
      if (!node) return
      const newConfig = { ...node.config, ...templateConfig }
      onConfigChange(node.id, newConfig)
      setJsonText(JSON.stringify(newConfig, null, 2))
    },
    [node, onConfigChange]
  )

  const handleJsonChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const text = e.target.value
    setJsonText(text)

    try {
      const config = JSON.parse(text)
      setError('')
      if (node) {
        onConfigChange(node.id, config)
      }
    } catch (err) {
      setError((err as Error).message)
    }
  }

  if (!node) {
    return (
      <div className="config-form">
        <div className="form-empty">Select a transform to configure</div>
      </div>
    )
  }

  return (
    <div className="config-form">
      <div className="form-header">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '12px' }}>
          <div>
            <h4>{node.label}</h4>
            <span className="form-type">{node.type_name}</span>
          </div>
          <div style={{ display: 'flex', gap: '8px' }}>
            {templates && templates.length > 0 && (
              <select
                className="form-template-select"
                defaultValue=""
                onChange={(e) => {
                  if (e.target.value) {
                    const template = templates.find((t) => t.name === e.target.value)
                    if (template) applyTemplate(template.config)
                    e.target.value = ''
                  }
                }}
                title="Quick templates"
              >
                <option value="">Presets</option>
                {templates.map((t) => (
                  <option key={t.name} value={t.name}>
                    {t.label}
                  </option>
                ))}
              </select>
            )}
            {history.length > 0 && (
              <div style={{ position: 'relative' }}>
                <button
                  className="form-history-btn"
                  onClick={() => setShowHistory(!showHistory)}
                  title="Configuration history"
                >
                  ⏱ {history.length}
                </button>
                {showHistory && (
                  <div className="form-history-dropdown">
                    {history.slice(0, 10).map((snap) => (
                      <button
                        key={snap.id}
                        className="form-history-item"
                        onClick={() => {
                          applyTemplate(snap.config)
                          setShowHistory(false)
                        }}
                        title={new Date(snap.timestamp).toLocaleString()}
                      >
                        <span className="history-label">{snap.label}</span>
                        <span className="history-time">
                          {Math.round((Date.now() - snap.timestamp) / 1000)}s ago
                        </span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="form-body">
        {fields ? (
          <FormRenderer
            fields={fields}
            config={node.config}
            onChange={handleFieldChange}
          />
        ) : (
          <div className="form-group">
            <label className="form-label">Configuration (JSON)</label>
            <textarea
              className={`form-textarea ${error ? 'error' : ''}`}
              value={jsonText}
              onChange={handleJsonChange}
              spellCheck="false"
            />
            {error && <div className="form-error">{error}</div>}
          </div>
        )}
      </div>
    </div>
  )
}

interface FormRendererProps {
  fields: FormField[]
  config: Record<string, unknown>
  onChange: (key: string, value: unknown) => void
}

function shouldShowField(field: FormField, config: Record<string, unknown>): boolean {
  if (!field.dependsOn) return true
  const dependencyValue = config[field.dependsOn.field]
  return dependencyValue === field.dependsOn.value
}

function FormRenderer({ fields, config, onChange }: FormRendererProps) {
  // Group fields by 'group' property
  const groups = fields.reduce((acc, field) => {
    const groupName = field.group || 'General'
    if (!acc[groupName]) acc[groupName] = []
    acc[groupName].push(field)
    return acc
  }, {} as Record<string, FormField[]>)

  // Render ungrouped fields first, then grouped fields
  const ungroupedFields = (groups['General'] || []).filter(f => shouldShowField(f, config))
  const groupedFields = Object.entries(groups)
    .filter(([name]) => name !== 'General')
    .map(([name, fields]) => [name, fields.filter(f => shouldShowField(f, config))] as const)
    .filter(([, fields]) => fields.length > 0)

  return (
    <>
      {ungroupedFields.map((field) => (
        <FormFieldComponent
          key={field.key}
          field={field}
          value={config[field.key]}
          onChange={(value) => onChange(field.key, value)}
        />
      ))}

      {groupedFields.map(([groupName, groupFields]) => (
        <div key={groupName} className="form-group-section">
          <h5 className="form-group-title">{groupName}</h5>
          {groupFields.map((field) => (
            <FormFieldComponent
              key={field.key}
              field={field}
              value={config[field.key]}
              onChange={(value) => onChange(field.key, value)}
            />
          ))}
        </div>
      ))}
    </>
  )
}

interface FormFieldComponentProps {
  field: FormField
  value: unknown
  onChange: (value: unknown) => void
}

function validateField(field: FormField, value: unknown): string | null {
  if (!field.validations) return null

  for (const validation of field.validations) {
    if (validation.type === 'pattern') {
      const strValue = String(value)
      if (strValue && !validation.value.test(strValue)) {
        return validation.message
      }
    }
    if (validation.type === 'minLength') {
      const strValue = String(value)
      if (strValue.length < validation.value) {
        return validation.message || `Minimum ${validation.value} characters`
      }
    }
    if (validation.type === 'maxLength') {
      const strValue = String(value)
      if (strValue.length > validation.value) {
        return validation.message || `Maximum ${validation.value} characters`
      }
    }
    if (validation.type === 'min') {
      const numValue = Number(value)
      if (numValue < validation.value) {
        return validation.message || `Minimum value: ${validation.value}`
      }
    }
    if (validation.type === 'max') {
      const numValue = Number(value)
      if (numValue > validation.value) {
        return validation.message || `Maximum value: ${validation.value}`
      }
    }
    if (validation.type === 'enum') {
      if (!validation.value.includes(String(value))) {
        return validation.message || `Must be one of: ${validation.value.join(', ')}`
      }
    }
    if (validation.type === 'custom') {
      if (!validation.fn(value)) {
        return validation.message
      }
    }
  }
  return null
}

function FormFieldComponent({ field, value, onChange }: FormFieldComponentProps) {
  const [error, setError] = useState<string | null>(null)
  const displayValue = value !== undefined ? value : (field.default ?? '')

  const handleChange = (newValue: unknown) => {
    onChange(newValue)
    const validationError = validateField(field, newValue)
    setError(validationError)
  }

  return (
    <div className="form-group">
      <label className="form-label">
        {field.label}
        {field.required && <span className="form-required">*</span>}
      </label>

      {field.type === 'string' && (
        <input
          type="text"
          className={`form-input ${error ? 'error' : ''}`}
          placeholder={field.placeholder}
          value={(displayValue as string) || ''}
          onChange={(e) => handleChange(e.target.value)}
        />
      )}

      {field.type === 'number' && (
        <input
          type="number"
          className={`form-input ${error ? 'error' : ''}`}
          value={(displayValue as number) || 0}
          onChange={(e) => handleChange(Number(e.target.value))}
        />
      )}

      {field.type === 'boolean' && (
        <label className="form-checkbox">
          <input
            type="checkbox"
            checked={(displayValue as boolean) || false}
            onChange={(e) => handleChange(e.target.checked)}
          />
          <span>{field.label}</span>
        </label>
      )}

      {field.type === 'select' && field.options && (
        <select
          className={`form-select ${error ? 'error' : ''}`}
          value={(displayValue as string) || ''}
          onChange={(e) => handleChange(e.target.value)}
        >
          <option value="">-- Select --</option>
          {field.options.map((opt) => (
            <option key={opt} value={opt}>
              {opt}
            </option>
          ))}
        </select>
      )}

      {field.type === 'textarea' && (
        <textarea
          className={`form-textarea ${error ? 'error' : ''}`}
          rows={field.rows || 4}
          placeholder={field.placeholder}
          value={(displayValue as string) || ''}
          onChange={(e) => handleChange(e.target.value)}
          spellCheck="false"
        />
      )}

      {(field.type === 'json' || field.type === 'stringArray') && (
        <textarea
          className={`form-textarea form-json ${error ? 'error' : ''}`}
          rows={field.rows || 6}
          placeholder={`${field.label} (JSON)`}
          value={typeof displayValue === 'string' ? displayValue : JSON.stringify(displayValue || {}, null, 2)}
          onChange={(e) => {
            try {
              const parsed = JSON.parse(e.target.value)
              handleChange(parsed)
            } catch {
              // Allow invalid JSON while editing
              handleChange(e.target.value)
            }
          }}
          spellCheck="false"
        />
      )}

      {error && <div className="form-error">{error}</div>}
      {field.help && <div className="form-help">{field.help}</div>}
      {field.examples && (
        <div className="form-examples">
          <strong>Examples:</strong>
          <code>{field.examples.join(', ')}</code>
        </div>
      )}
    </div>
  )
}
