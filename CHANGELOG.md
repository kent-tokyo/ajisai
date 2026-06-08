# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-06-08

### Phase 6B: Electron GUI Migration - COMPLETE ✅

#### Phase 6B-0: Shared Type Extraction
- Moved `PipelineState`, `Node`, `Edge` from egui crate to `ajisai-core`
- Updated Rust edition to 2024
- Maintained egui GUI backward compatibility

#### Phase 6B-1: JSON-RPC Server
- Created `crates/server` with stdio JSON-RPC protocol
- Implemented 6 core methods: ping, get_transforms, run_pipeline, validate_pipeline, load_pipeline, save_pipeline
- Single stdout writer task prevents frame collision
- Full serde serialization support

#### Phase 6B-2: Electron Scaffold
- Created `electron/` directory with electron-vite setup
- Implemented main process with sidecar Rust binary management
- Added contextBridge API (`window.ajisai`)
- Zustand store for state management
- TypeScript type definitions for pipeline model

#### Phase 6B-3: Canvas + UI Components
- DAG visual editor using @xyflow/react v12
- Transform palette sidebar with categorization
- Properties panel with JSON config editor
- Log panel with auto-scroll and 500-line history
- Status bar with real-time execution indicators
- Responsive layout with VS Code dark theme

#### Phase 6B-4: File I/O + Keyboard Operations
- Menu bar with File / Pipeline menus
- Keyboard shortcuts: Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+R
- Pipeline execution with streaming progress
- Real-time log display (node status + server messages)

### Phase 6C: GUI Polish & Optimization - COMPLETE ✅

#### Phase 6C-1: Per-Transform Form Definitions
- 50+ transform type definitions with typed fields
- Auto-detection of form type (string, number, boolean, select, textarea, json)
- Real-time JSON validation
- FormRenderer for type-specific UI generation
- Fallback to JSON textarea for unknown transforms

#### Phase 6C-2: Undo/Redo Integration
- Zustand store with 50-snapshot undo/redo stack (egui parity)
- Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z key bindings
- Auto-clear redo stack on new mutation
- `canUndo()` / `canRedo()` state queries

#### Phase 6C-3: Window State Persistence
- localStorage auto-save on pipeline changes
- Pipeline restoration on app startup
- Auto-save eliminates accidental work loss

#### Phase 6C-4: Performance Optimization
- React.memo() on Canvas component
- useMemo() hooks to prevent unnecessary re-renders
- Bundle size: 606.94 kB JS + 32.82 kB CSS
- Efficient state updates with snapshot-based undo

### Phase 6C-5: Release Preparation - COMPLETE ✅

#### Build & Distribution
- electron-builder integration for cross-platform builds
- Targets: macOS (dmg, zip), Windows (nsis, portable), Linux (AppImage, deb)
- electron-updater for auto-update support
- GitHub Releases provider configuration (owner: k-nasa, repo: ajisai)

#### Platform-Specific Setup
- macOS: DMG + ZIP distribution (code signing stubbed: sign: false)
- Windows: NSIS + Portable installer (one-click disabled)
- Linux: AppImage + DEB package support

#### Auto-Update Infrastructure
- `electron-updater` listens for releases on GitHub
- Binary comparison for efficient updates
- Graceful error handling with console logging
- Dev mode bypasses auto-update checks

---

## Previous Phases (Completed)

### Phase 1-5A: Core Engine & 50+ Transforms ✅
### Phase 5B-5E: Advanced Transforms & Scripting ✅
### Phase 6A: Window Functions & Workflow Enhancements ✅

---

## Release Notes Format

For each release, document:
- **New transforms** added (if any)
- **UI improvements** (canvas, forms, panels)
- **Bug fixes** (with issue numbers if applicable)
- **Performance improvements** (bundle size, runtime)
- **Breaking changes** (if applicable)

Example:
```
## [0.2.0] - YYYY-MM-DD

### Added
- Per-transform form definitions (50+ transforms)
- Undo/Redo stack support

### Fixed
- Canvas performance with 50+ nodes
- Auto-save on every pipeline change

### Changed
- Bundle size optimized from X kB to Y kB
```

---

## Version Numbering

- **Patch (0.x.y)**: Bug fixes, minor UI tweaks
- **Minor (0.y.0)**: New transforms, new features
- **Major (x.0.0)**: Phase completion, architecture changes

Current: **0.1.0** — Phase 6 (GUI Rewrite) complete, ready for Phase 7+ planning

---

## Future Roadmap

### Phase 6D: egui GUI Deprecation
- Retire legacy `crates/gui` from workspace
- Archive to reference branch

### Phase 7: Enhanced Form Builder
- Custom transform config generator
- Per-field validation rules
- Form templates library

### Phase 8: Advanced Analytics
- Pipeline execution metrics dashboard
- Transform performance profiling
- Data lineage tracking

---

Last updated: 2026-06-08
Maintained by: Claude Code + Kentaro Tanabe
