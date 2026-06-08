# Ajisai 0.1.0 Release Notes

**Release Date**: 2026-06-08  
**Status**: GA Candidate (Phase 6B-6C Complete)

---

## 🎉 Major Milestone: Electron GUI is Live!

After 7 days of intensive development, **Phase 6B (GUI Migration) and Phase 6C (Polish & Optimization) are complete**. The Electron + React GUI is production-ready, with all features from the legacy egui implementation now available in a modern, high-performance desktop app.

---

## What's New in 0.1.0

### GUI (Electron + React)
✅ **JSON-RPC Sidecar Server**
- Rust core runs as a background process
- Stdin/stdout JSON-RPC protocol (no external dependencies)
- 6 core methods: ping, get_transforms, run_pipeline, validate, load, save

✅ **Visual Pipeline Editor**
- DAG canvas with @xyflow/react v12
- Drag-drop node placement
- Real-time edge drawing and validation
- Zoom/pan/fit controls

✅ **Transform Palette**
- 50+ transforms grouped by category (I/O, Transform, Join/Lookup, Variables/Flow)
- Click to add nodes to canvas
- Default config provided for each transform type

✅ **Per-Transform Form Definitions**
- 50 transforms with typed config forms
- Field types: string, number, boolean, select, textarea, JSON editor
- Real-time validation
- Fallback to JSON editor for unknown types

✅ **Undo/Redo Stack**
- Ctrl+Z / Ctrl+Y keyboard shortcuts
- 50-snapshot history (matches egui behavior)
- Automatic stack management

✅ **Pipeline Execution**
- Real-time progress tracking
- Node status indicators (idle, running, done, error)
- Streaming log output
- Auto-sync with Rust core

✅ **Auto-Save**
- localStorage persistence
- Pipeline restored on app startup
- No accidental work loss

✅ **Menu Bar**
- File menu: New, Open, Save, Exit
- Pipeline menu: Run, Clear Log
- Full keyboard shortcut support (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+R)

### Rust Core (crates/server)
- New `ajisai-server` binary for IPC bridge
- Edition 2024 support
- Single stdout writer for frame-safe JSON-RPC
- Full streaming capability

### Build & Distribution
- electron-builder integration
- macOS (DMG, ZIP), Windows (NSIS, Portable), Linux (AppImage, DEB)
- electron-updater for auto-update support
- GitHub Releases configuration

---

## Performance

**Bundle Size**
- JavaScript: 606.94 kB (includes @xyflow/react, React, Zustand)
- CSS: 32.82 kB (dark theme + component styles)
- Total: ~640 kB (gzipped ~180 kB)

**Optimization**
- React.memo() on Canvas component
- useMemo() hooks to prevent re-renders
- Efficiently handles 50+ node pipelines

---

## Compatibility

✅ **Backward Compatible**
- Legacy egui GUI still builds (`cargo build --workspace`)
- All .hpl, .json pipeline files work unchanged
- Rust CLI (`ajisai-cli`) unaffected

✅ **Apache Hop Compatible**
- Reads/writes .hpl files natively
- Supports .hwf, .ktr, .dtsx formats via hop-compat
- All 50+ transforms compatible

---

## Getting Started

### Development Mode
```bash
cd electron
npm install
npm run dev
```

This launches the Electron app with hot-reload and DevTools.

### Build for Distribution
```bash
cd electron
npm run build
# Outputs: out/Ajisai-0.1.0.dmg (macOS), Ajisai-0.1.0.exe (Windows), ajisai-0.1.0.AppImage (Linux)
```

### CLI (No GUI)
```bash
cargo run --bin ajisai-cli -- run -p path/to/pipeline.hpl
```

---

## Known Limitations

- Code signing not enabled (unsigned builds for now)
- Auto-update requires GitHub release with binary
- Progress notifications are MVP (start/done per node, not per-row)
- 50-transform category display is static (from formDescriptors.ts)

---

## What's Next (Phase 6D+)

### Phase 6D: egui GUI Deprecation
- Retire legacy `crates/gui` from workspace
- Archive to reference branch

### Phase 7+: Future Enhancements
- Custom transform schema generation
- Pipeline execution metrics dashboard
- Data lineage tracking
- Advanced analytics

---

## Technical Details

### Architecture
```
ajisai-cli (Rust)
ajisai-server (Rust) ← Electron (React) via JSON-RPC stdio
ajisai-core (Rust)
ajisai-transforms (Rust, 50+ transforms)
ajisai-hop-compat (Rust, .hpl/.hwf/.ktr/.dtsx)
```

### IPC Protocol (JSON-RPC 2.0)
```
Request:  {"id": 1, "method": "run_pipeline", "params": {...}}
Response: {"id": 1, "result": {...}}
Notify:   {"method": "pipeline/progress", "params": {...}}
```

### State Management (Zustand)
- Single store: `usePipelineStore`
- Immutable updates with automatic undo/redo
- localStorage auto-sync
- TypeScript types generated from Rust model

---

## Contributors

- **Rust Core**: Full Phase 1-5 transforms (50+ complete)
- **Electron GUI**: Phase 6B-6C (Canvas, Forms, Undo/Redo, Auto-Save)
- **Infrastructure**: JSON-RPC bridge, electron-builder, GitHub Releases

---

## License

MIT OR Apache-2.0

---

## Feedback & Reporting

- **Bug Reports**: GitHub Issues
- **Feature Requests**: GitHub Discussions
- **Contributions**: Fork + Pull Request welcome!

---

**Version**: 0.1.0  
**Release Type**: GA Candidate  
**Build Date**: 2026-06-08  
**Status**: ✅ Ready for Testing & Feedback
