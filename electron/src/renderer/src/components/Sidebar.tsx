import { useState } from 'react'
import { usePipelineStore } from '../store/pipelineStore'
import { ChevronIcon } from './Icons'
import type { TransformCategory } from '../types/pipeline'
import './Sidebar.css'

export function Sidebar() {
  const { categories, addNode } = usePipelineStore()
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set())

  const toggleCategory = (catName: string) => {
    const newCollapsed = new Set(collapsed)
    if (newCollapsed.has(catName)) {
      newCollapsed.delete(catName)
    } else {
      newCollapsed.add(catName)
    }
    setCollapsed(newCollapsed)
  }

  const handleTransformClick = (typeName: string) => {
    // Add node at a default position (center-ish)
    addNode(typeName, [300 + Math.random() * 100, 200 + Math.random() * 100])
  }

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h3>Transforms</h3>
      </div>

      <div className="sidebar-content">
        {categories.length === 0 ? (
          <div className="empty-state">Loading transforms...</div>
        ) : (
          categories.map((category: TransformCategory) => (
            <div key={category.name} className="category-section">
              <button
                className={`category-header ${collapsed.has(category.name) ? 'collapsed' : ''}`}
                onClick={() => toggleCategory(category.name)}
              >
                <span className="chevron">
                  <ChevronIcon size={14} color="var(--text-dim)" />
                </span>
                <span className="category-name">{category.name}</span>
                <span className="count">({category.transforms.length})</span>
              </button>

              {!collapsed.has(category.name) && (
                <div className="transform-list">
                  {category.transforms.map((transform) => (
                    <button
                      key={transform.type_name}
                      className="transform-item"
                      onClick={() => handleTransformClick(transform.type_name)}
                      title={transform.type_name}
                    >
                      <span className="transform-name">{transform.display_name}</span>
                      <span className="type-hint">{transform.type_name}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  )
}
