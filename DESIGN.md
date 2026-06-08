# DESIGN.md — Ajisai GUI Design Guidelines

> **Claude Code daily context**: colors · typography · layout dimensions · design tokens.
>
> Research basis: VSCode (dark theme), @xyflow/react (DAG visualization), Zustand (state management);
> ETL tools: Talend, Apache NiFi, Airbyte (pipeline UX), egui (legacy Rust GUI).

---

## 0. Design Philosophy

Primary users: **data engineers, ETL developers, data scientists**.

| Principle | Meaning |
|-----------|---------|
| **Data pipeline clarity** | DAG visualization must show data flow unambiguously. Node relationships and status (idle/running/done/error) must be instantly readable. |
| **VS Code familiarity** | Dark theme and keyboard shortcuts familiar to dev users. Ctrl+Z/Y/N/O/S/R work intuitively. |
| **Performance at scale** | Handle 50+ node pipelines without lag. React.memo and useMemo prevent re-render storms. |
| **Form clarity** | 50 transform types with type-aware forms (string/number/boolean/select/textarea/json). Defaults provided, JSON fallback for unknowns. |
| **Observable execution** | Real-time log streaming, per-node status dots, row counts. Execution is transparent. |
| **Discoverable** | Every keyboard shortcut visible in menus. Tooltips on hover. Status bar hints. |
| **Cross-platform** | Electron builds for macOS/Windows/Linux with consistent UX — no OS-specific compromises. |

---

## 1. Color Palette (VS Code Dark Theme)

### 1.1 Semantic UI Colors

| Role | Value | Meaning |
|------|-------|---------|
| **Editor BG** | `#1e1e1e` | Canvas and main work area background |
| **Sidebar BG** | `#252526` | Left sidebar and panels |
| **Activity Bar BG** | `#333333` | Far-left icon bar |
| **Accent** | `#007acc` | Selection, hover, active state, run button |
| **Success** | `#6a9955` | Node done, execution complete |
| **Warning** | `#dcdcaa` | Node preparing, progress indicator |
| **Error** | `#f48771` | Node failed, validation error |
| **Idle** | `#858585` | Node idle, not executed |
| **Text Primary** | `#cccccc` | Labels, menu text |
| **Text Secondary** | `#858585` | Hints, disabled text |
| **Border** | `#3e3e42` | Separators, panel borders |

### 1.2 Transform Category Colors (Canvas)

Visual distinction for node types in DAG:

| Category | Color | Usage |
|----------|-------|-------|
| **I/O** | `#4EC9B0` | CsvFileInput, ExcelOutput, JsonFileInput, etc. |
| **Transform** | `#CE9178` | FilterRows, SelectValues, SortRows, etc. |
| **Join/Lookup** | `#9CDCFE` | JoinTwoInputs, LookupTable, MergeStreams, etc. |
| **Variables/Flow** | `#C586C0` | SetVariable, GetVariable, SwitchCase, etc. |

### 1.3 Node Status Indicators

Pulsing animation for running state:

| Status | Color | Style |
|--------|-------|-------|
| **Idle** | `#858585` | 6px dot, solid |
| **Running** | `#dcdcaa` | 6px dot, pulsing @ 1s interval |
| **Done** | `#6a9955` | 6px dot, solid checkmark overlay |
| **Error** | `#f48771` | 6px dot, solid ✕ overlay |

### 1.4 Color Rules

- Dark theme only (VS Code aesthetic). Light mode support deferred to 0.2.0.
- All text on background must meet WCAG AA (contrast >= 4.5:1).
- Category colors chosen to be distinct on `#1e1e1e` and `#252526` backgrounds.
- Edge connections use `#3e3e42` (border color) with `#007acc` highlight on hover/selection.
- Selection outline: 2px `#007acc` border + subtle glow shadow.

---

## 2. Typography

### 2.1 Font Stack (React/Electron)

System font stack for cross-platform consistency:

```
[1] -apple-system, BlinkMacSystemFont (macOS)
[2] "Segoe UI" (Windows)
[3] Ubuntu, sans-serif (Linux)
[4] "Noto Sans CJK JP" / "Hiragino Kaku Gothic" (CJK fallback)
[5] monospace (for code/JSON)
```

### 2.2 Type Scale

| Role | Size | Usage |
|------|------|-------|
| **Menu/Label** | 13 px | Menu bar, form labels |
| **Body** | 12 px | Sidebar text, properties |
| **Caption** | 11 px | Help text, hints |
| **Monospace** | 12 px | JSON configs, log output, coordinates |
| **Node Label** | 12 px | Canvas node titles |

### 2.3 Rules

- UI labels use Sentence case (`"Run pipeline"` correct, `"Run Pipeline"` incorrect).
- JSON output and log messages always use monospace font.
- Node labels auto-truncate at 20 chars with ellipsis (`...`).

---

## 3. Layout

### 3.1 Main Window Structure (Electron App)

```
+-----------------------------------------------------------------------+
| Menu bar: File | Pipeline | Help                                   | <- 24 px
+-----------------------------------------------------------------------+
| Activity bar | Sidebar (Transform palette)                              |
| (icons)      | - CsvFileInput                                              |
| 48px         | - CsvFileOutput                                             |
|              | - FilterRows                                                |
|              | - SelectValues                                              |
|              | [... 50+ transforms]                                        |
+-------+--------+--------------------------------+------------------------+
|       |        |                                |                        |
|  Act. | Side  |  Canvas (DAG Editor)          | Properties Panel       |
| Bar   | bar   |  - Node dragging              | - Config Form          |
| 48px  | 240px | - Edge connections            | - JSON editor          |
|       |       | - Zoom / Pan / Fit            | - Field values         |
|       |       |                                | 280px                  |
|       |       |                                |                        |
+-------+--------+--------------------------------+------------------------+
| Log Panel (auto-scroll, 500 line max, resizable)                      | <- ~160px
+-----------------------------------------------------------------------+
| Status bar: "Running 3/5 nodes | 1250 rows read | ✓ Pipeline OK"      | <- 24 px
+-----------------------------------------------------------------------+
```

| Region | Dimension | Contents |
|--------|-----------|----------|
| **Activity Bar** | 48 px (fixed) | Icons: Sidebar toggle, settings |
| **Sidebar** | 240 px (default, resizable) | Transform palette (50+ types), search |
| **Canvas** | Flexible | DAG editor, @xyflow/react, zoom/pan/fit controls |
| **Properties** | 280 px (fixed) | ConfigForm, JSON editor, default values |
| **Log Panel** | ~160 px (resizable) | Execution logs, auto-scroll, clear button |
| **Status Bar** | 24 px (fixed) | Execution state, row counts, pipeline name |

### 3.2 Grid and Spacing

- Base unit: **4 px**
- Element spacing: 4 / 8 / 16 / 24 px
- Border radius: buttons/inputs 4 px, panels 0 px (VS Code style)
- Padding: form fields 8 px, panels 16 px

### 3.3 Canvas-Specific

- Canvas zoom: 10% to 200% (default 100%)
- Node size: 80 x 48 px (fixed)
- Node padding: 8 px (label + status dot)
- Edge stroke: 2 px
- Grid: off by default (toggle via View menu)

---

## 4. Dark Mode (Current)

- **Dark theme only** (VS Code aesthetic). Light mode deferred to 0.2.0.
- Default theme: `#1e1e1e` editor background, `#252526` sidebar.
- Runtime persistence: theme preference stored in localStorage.
- All color values defined in `electron/src/renderer/src/index.css` as CSS variables.

---

## 5. Internationalization (i18n)

- **Supported**: Japanese (ja) and English (en)
- **Implementation**: i18n strings in component props (not yet extracted to resource files)
- **Future (0.2.0)**: Extract to `src/i18n/en.json` / `src/i18n/ja.json`
- **Switching**: Future menu option `View > Language > English / 日本語` (instant, no restart)
- **Key format** (future): dot-separated, e.g. `menu.file.open`, `transform.csvInput`

---

## Appendix A: Design Tokens (CSS Variables)

### Colors (electron/src/renderer/src/index.css)

```css
:root {
  /* Base theme */
  --bg-editor: #1e1e1e;
  --bg-sidebar: #252526;
  --bg-actbar: #333333;
  --bg-input: #3e3e42;
  
  /* Text colors */
  --text-primary: #cccccc;
  --text-secondary: #858585;
  
  /* Accent & Status */
  --accent: #007acc;
  --status-idle: #858585;
  --status-running: #dcdcaa;
  --status-done: #6a9955;
  --status-error: #f48771;
  
  /* Category colors */
  --category-io: #4EC9B0;
  --category-transform: #CE9178;
  --category-join: #9CDCFE;
  --category-variable: #C586C0;
  
  /* Borders */
  --border: #3e3e42;
}
```

### Layout Dimensions (TypeScript)

```typescript
export const LAYOUT = {
  activityBar: 48,        // px, fixed
  sidebar: 240,           // px, default (resizable)
  sidebarMin: 200,        // px
  sidebarMax: 400,        // px
  properties: 280,        // px, fixed
  logPanel: 160,          // px, default (resizable)
  menuBar: 24,            // px, fixed
  statusBar: 24,          // px, fixed
  
  // Node & Canvas
  nodeWidth: 80,          // px
  nodeHeight: 48,         // px
  nodeRadius: 4,          // px
  edgeStroke: 2,          // px
  
  // Spacing (4px grid)
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  
  // Interaction
  dragThreshold: 4,       // px
  zoomMin: 0.1,           // 10%
  zoomMax: 2.0,           // 200%
  zoomDefault: 1.0,       // 100%
  undoStackMax: 50,       // snapshots
  logPanelMaxLines: 500,  // line cap
}
```

### Animations

```css
/* Node status pulse (running state) */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
.node-status-running {
  animation: pulse 1s infinite;
}

/* Transition durations */
--transition-short: 150ms;  /* hover effects */
--transition-medium: 300ms; /* panel collapse */
--transition-long: 500ms;   /* theme switch */
```
