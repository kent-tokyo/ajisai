use crate::{
    canvas::{draw_connecting_edge, draw_edges, draw_node, output_port, NODE_H, NODE_W},
    config_form, runner,
    state::{Node, NodeStatus, PipelineState, UiState, UndoStack, TRANSFORM_CATEGORIES},
};
use egui::{Color32, FontId, Pos2, ScrollArea, Sense, Vec2};
use rust_i18n::t;
use std::sync::mpsc;

pub struct AjisaiApp {
    pipeline: PipelineState,
    ui: UiState,
    canvas_offset: Vec2,
    canvas_zoom: f32,
    run_rx: Option<mpsc::Receiver<String>>,
    undo: UndoStack,
}

impl Default for AjisaiApp {
    fn default() -> Self {
        Self {
            pipeline: PipelineState::new("Untitled Pipeline"),
            ui: UiState::default(),
            canvas_offset: Vec2::new(40.0, 40.0),
            canvas_zoom: 1.0,
            run_rx: None,
            undo: UndoStack::default(),
        }
    }
}

impl AjisaiApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self::default()
    }

    // ── Undo helpers ─────────────────────────────────────────────────────

    /// Snapshot current pipeline state onto the undo stack before a mutation.
    fn snapshot(&mut self) {
        self.undo.push(self.pipeline.clone());
    }

    fn do_undo(&mut self) {
        if let Some(prev) = self.undo.undo(self.pipeline.clone()) {
            self.pipeline = prev;
            self.ui.selected_node = None;
            self.ui.node_status.clear();
        }
    }

    fn do_redo(&mut self) {
        if let Some(next) = self.undo.redo(self.pipeline.clone()) {
            self.pipeline = next;
            self.ui.selected_node = None;
            self.ui.node_status.clear();
        }
    }

    // ── Keyboard shortcuts ────────────────────────────────────────────────

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Skip when a text field is active
        if ctx.wants_keyboard_input() {
            return;
        }

        // Delete / Backspace → remove selected node
        if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            if let Some(sel_id) = self.ui.selected_node.clone() {
                self.snapshot();
                self.pipeline.remove_node(&sel_id);
                self.ui.json_edit_buf.remove(&sel_id);
                self.ui.selected_node = None;
            }
        }

        // Ctrl+Z / ⌘+Z → undo
        if ctx.input(|i| {
            i.key_pressed(egui::Key::Z)
                && (i.modifiers.ctrl || i.modifiers.command)
                && !i.modifiers.shift
        }) {
            self.do_undo();
        }

        // Ctrl+Y / ⌘+Y or Ctrl+Shift+Z / ⌘+Shift+Z → redo
        if ctx.input(|i| {
            (i.key_pressed(egui::Key::Y) && (i.modifiers.ctrl || i.modifiers.command))
                || (i.key_pressed(egui::Key::Z)
                    && (i.modifiers.ctrl || i.modifiers.command)
                    && i.modifiers.shift)
        }) {
            self.do_redo();
        }

        // Ctrl+D / ⌘+D → duplicate selected node
        if ctx.input(|i| i.key_pressed(egui::Key::D) && (i.modifiers.ctrl || i.modifiers.command)) {
            self.duplicate_selected();
        }

        // Ctrl+S / ⌘+S → save (overwrite current file, or open save dialog)
        if ctx.input(|i| i.key_pressed(egui::Key::S) && (i.modifiers.ctrl || i.modifiers.command)) {
            if self.ui.current_file.is_some() {
                self.save_current_file();
            } else {
                self.save_hpl_dialog();
            }
        }
    }

    fn duplicate_selected(&mut self) {
        if let Some(sel_id) = self.ui.selected_node.clone() {
            // Clone all needed data before any mutable borrow
            if let Some(mut new_node) = self.pipeline.node(&sel_id).cloned() {
                let new_id = self.pipeline.next_id();
                new_node.label = format!("{} (copy)", new_node.label);
                new_node.id = new_id.clone();
                new_node.pos[0] += 24.0;
                new_node.pos[1] += 24.0;
                self.snapshot();
                self.pipeline.add_node(new_node);
                self.ui.selected_node = Some(new_id);
            }
        }
    }

    // ── Engine ───────────────────────────────────────────────────────────

    fn poll_engine(&mut self) {
        let done = if let Some(rx) = &self.run_rx {
            let mut finished = false;
            loop {
                match rx.try_recv() {
                    Ok(msg) if msg == "__DONE__" => {
                        finished = true;
                        break;
                    }
                    Ok(msg) => {
                        // Mark nodes done or error based on message prefix
                        let is_err = msg.starts_with("[ERROR]") || msg.starts_with("Error");
                        let status = if is_err {
                            NodeStatus::Error
                        } else {
                            NodeStatus::Done
                        };
                        for node in &self.pipeline.nodes {
                            self.ui.node_status.insert(node.id.clone(), status.clone());
                        }
                        self.ui.log(msg);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        finished = true;
                        break;
                    }
                }
            }
            finished
        } else {
            false
        };
        if done {
            self.ui.pipeline_running = false;
            self.run_rx = None;
        }
    }

    // ── Panels ───────────────────────────────────────────────────────────

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(t!("menu.file"), |ui| {
                    if ui.button(t!("menu.new")).clicked() {
                        self.snapshot();
                        self.pipeline = PipelineState::new("Untitled Pipeline");
                        self.ui.selected_node = None;
                        self.ui.log_lines.clear();
                        self.ui.node_status.clear();
                        self.ui.json_edit_buf.clear();
                        ui.close_menu();
                    }
                    if ui.button(t!("menu.open_hpl")).clicked() {
                        self.open_hpl_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(t!("menu.save_json")).clicked() {
                        self.save_json_dialog();
                        ui.close_menu();
                    }
                    if ui.button(t!("menu.save_hpl")).clicked() {
                        self.save_hpl_dialog();
                        ui.close_menu();
                    }
                });

                ui.menu_button(t!("menu.pipeline"), |ui| {
                    let running = self.ui.pipeline_running;
                    if ui
                        .add_enabled(!running, egui::Button::new(t!("menu.run")))
                        .clicked()
                    {
                        self.run_pipeline(ctx);
                        ui.close_menu();
                    }
                    if ui.button(t!("menu.validate")).clicked() {
                        self.validate_pipeline();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(self.undo.can_undo(), egui::Button::new(t!("menu.undo")))
                        .clicked()
                    {
                        self.do_undo();
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(self.undo.can_redo(), egui::Button::new(t!("menu.redo")))
                        .clicked()
                    {
                        self.do_redo();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(t!("menu.clear")).clicked() {
                        self.snapshot();
                        self.pipeline.nodes.clear();
                        self.pipeline.edges.clear();
                        self.ui.selected_node = None;
                        self.ui.node_status.clear();
                        self.ui.json_edit_buf.clear();
                        ui.close_menu();
                    }
                });

                ui.menu_button(t!("menu.language"), |ui| {
                    let cur = rust_i18n::locale().to_string();
                    if ui.selectable_label(cur == "en", "English").clicked() {
                        rust_i18n::set_locale("en");
                        ui.close_menu();
                    }
                    if ui.selectable_label(cur == "ja", "日本語").clicked() {
                        rust_i18n::set_locale("ja");
                        ui.close_menu();
                    }
                });

                ui.separator();
                // Inline pipeline name editing
                ui.add(
                    egui::TextEdit::singleline(&mut self.pipeline.name)
                        .desired_width(160.0)
                        .font(FontId::proportional(13.0)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.ui.pipeline_running {
                        ui.spinner();
                        ui.label(t!("status.running"));
                    } else {
                        let zoom_pct = (self.canvas_zoom * 100.0).round() as i32;
                        ui.label(t!(
                            "status.nodes",
                            n = self.pipeline.nodes.len().to_string().as_str(),
                            e = self.pipeline.edges.len().to_string().as_str(),
                            z = zoom_pct.to_string().as_str(),
                        ));
                    }
                });
            });
        });
    }

    fn left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("palette")
            .resizable(false)
            .exact_width(170.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(t!("panel.transforms")).strong());
                ui.separator();

                ScrollArea::vertical().show(ui, |ui| {
                    for (category, transforms) in TRANSFORM_CATEGORIES {
                        let collapsed = self.ui.collapsed_categories.contains(*category);
                        let arrow = if collapsed { "▶" } else { "▼" };
                        let header = format!("{} {}", arrow, category);

                        if ui
                            .selectable_label(
                                false,
                                egui::RichText::new(&header).strong().size(11.0),
                            )
                            .clicked()
                        {
                            if collapsed {
                                self.ui.collapsed_categories.remove(*category);
                            } else {
                                self.ui.collapsed_categories.insert(category.to_string());
                            }
                        }

                        if !collapsed {
                            for (type_name, display_name) in *transforms {
                                let dot_color = crate::canvas::category_color(type_name);
                                let btn_resp = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(*display_name).size(12.0),
                                    )
                                    .min_size(egui::vec2(155.0, 26.0)),
                                );
                                // Paint category dot beside the label
                                let dot_pos =
                                    Pos2::new(btn_resp.rect.left() + 6.0, btn_resp.rect.center().y);
                                ui.painter().circle_filled(dot_pos, 3.5, dot_color);

                                if btn_resp.clicked() {
                                    self.add_node(type_name);
                                }
                            }
                        }
                        ui.add_space(2.0);
                    }
                });
            });
    }

    fn right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("properties")
            .resizable(true)
            .min_width(200.0)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(t!("panel.properties")).strong());
                ui.separator();

                if let Some(sel_id) = self.ui.selected_node.clone() {
                    ScrollArea::vertical().show(ui, |ui| {
                        // Type badge + name
                        let type_name = self
                            .pipeline
                            .node(&sel_id)
                            .map(|n| n.type_name.clone())
                            .unwrap_or_default();
                        let badge_color = crate::canvas::category_color(&type_name);

                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 5.0, badge_color);
                            ui.label(egui::RichText::new(&type_name).monospace().size(11.0));
                        });
                        ui.add_space(4.0);

                        // Node label
                        ui.label(t!("prop.name"));
                        if let Some(node) = self.pipeline.node_mut(&sel_id) {
                            ui.text_edit_singleline(&mut node.label);
                        }
                        ui.add_space(8.0);

                        // Config form
                        ui.separator();
                        ui.label(t!("prop.config"));
                        ui.add_space(4.0);
                        if let Some(node) = self.pipeline.node_mut(&sel_id) {
                            config_form::show_config_form(
                                ui,
                                &type_name,
                                &mut node.config,
                                &sel_id,
                                &mut self.ui.json_edit_buf,
                            );
                        }

                        ui.add_space(8.0);
                        ui.separator();

                        // Outgoing connections (with ✕ to disconnect)
                        let out_edges: Vec<String> = self
                            .pipeline
                            .edges
                            .iter()
                            .filter(|e| e.from == sel_id)
                            .map(|e| e.to.clone())
                            .collect();
                        let in_edges: Vec<String> = self
                            .pipeline
                            .edges
                            .iter()
                            .filter(|e| e.to == sel_id)
                            .map(|e| e.from.clone())
                            .collect();

                        if !out_edges.is_empty() || !in_edges.is_empty() {
                            ui.label(
                                egui::RichText::new(t!("prop.connections"))
                                    .size(11.0)
                                    .weak(),
                            );
                            for to_id in &out_edges {
                                let to_label = self
                                    .pipeline
                                    .node(to_id)
                                    .map(|n| n.label.as_str())
                                    .unwrap_or(to_id.as_str())
                                    .to_owned();
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("→ {}", to_label)).size(11.0),
                                    );
                                    if ui.small_button("✕").clicked() {
                                        self.snapshot();
                                        self.pipeline.remove_edge(&sel_id, to_id);
                                    }
                                });
                            }
                            for from_id in &in_edges {
                                let from_label = self
                                    .pipeline
                                    .node(from_id)
                                    .map(|n| n.label.as_str())
                                    .unwrap_or(from_id.as_str());
                                ui.label(
                                    egui::RichText::new(format!("← {}", from_label)).size(11.0),
                                );
                            }
                            ui.add_space(4.0);
                            ui.separator();
                        }

                        // Duplicate + Delete
                        ui.horizontal(|ui| {
                            if ui.button(t!("prop.duplicate")).clicked() {
                                self.duplicate_selected();
                            }
                            if ui
                                .button(
                                    egui::RichText::new(t!("prop.delete"))
                                        .color(Color32::from_rgb(220, 80, 80)),
                                )
                                .clicked()
                            {
                                let id = sel_id.clone();
                                self.snapshot();
                                self.pipeline.remove_node(&id);
                                self.ui.json_edit_buf.remove(&id);
                                self.ui.selected_node = None;
                            }
                        });
                    });
                } else {
                    ui.add_space(6.0);
                    ui.label(t!("prop.pipeline_name"));
                    ui.text_edit_singleline(&mut self.pipeline.name);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.colored_label(Color32::GRAY, t!("prop.no_selection"));
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(t!("prop.shortcuts_hint"))
                            .size(10.0)
                            .weak(),
                    );
                }
            });
    }

    fn log_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .min_height(80.0)
            .default_height(140.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t!("panel.log")).strong());
                    if ui.small_button(t!("panel.log_clear")).clicked() {
                        self.ui.log_lines.clear();
                    }
                });
                ui.separator();

                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for line in &self.ui.log_lines {
                        let color = if line.starts_with("[ERROR]") || line.starts_with("Error") {
                            Color32::from_rgb(255, 100, 100)
                        } else if line.starts_with("[OK]")
                            || line.contains("completed")
                            || line.contains("完了")
                        {
                            Color32::from_rgb(100, 220, 100)
                        } else {
                            Color32::LIGHT_GRAY
                        };
                        ui.colored_label(color, egui::RichText::new(line).monospace().size(11.0));
                    }
                });
            });
    }

    fn canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(28, 30, 38)))
            .show(ctx, |ui| {
                let canvas_rect = ui.max_rect();
                let painter = ui.painter_at(canvas_rect);

                // Scroll-to-zoom toward cursor
                let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll_delta != 0.0 {
                    if let Some(cursor) = ctx.input(|i| i.pointer.hover_pos()) {
                        if canvas_rect.contains(cursor) {
                            let old_zoom = self.canvas_zoom;
                            let new_zoom =
                                (old_zoom * (1.0 + scroll_delta * 0.002)).clamp(0.2, 4.0);
                            let world = (cursor.to_vec2() - self.canvas_offset) / old_zoom;
                            self.canvas_offset = cursor.to_vec2() - world * new_zoom;
                            self.canvas_zoom = new_zoom;
                        }
                    }
                }

                self.draw_grid(&painter, canvas_rect);
                draw_edges(
                    &painter,
                    &self.pipeline,
                    self.canvas_offset,
                    self.canvas_zoom,
                );

                if let Some(from_id) = &self.ui.connecting_from.clone() {
                    if let Some(from_node) = self.pipeline.node(from_id) {
                        let from_pos = output_port(from_node, self.canvas_offset, self.canvas_zoom);
                        if let Some(cursor) = ctx.input(|i| i.pointer.hover_pos()) {
                            draw_connecting_edge(&painter, from_pos, cursor);
                        }
                    }
                }

                let canvas_resp = ui.allocate_rect(canvas_rect, Sense::click_and_drag());

                if canvas_resp.dragged_by(egui::PointerButton::Middle)
                    || (canvas_resp.dragged_by(egui::PointerButton::Primary)
                        && ctx.input(|i| i.modifiers.alt))
                {
                    self.canvas_offset += canvas_resp.drag_delta();
                }

                if canvas_resp.clicked() {
                    self.ui.selected_node = None;
                    self.ui.connecting_from = None;
                }
                if canvas_resp.secondary_clicked() {
                    self.ui.connecting_from = None;
                }

                let ids: Vec<String> = self.pipeline.nodes.iter().map(|n| n.id.clone()).collect();

                for node_id in &ids {
                    let is_selected = self.ui.selected_node.as_deref() == Some(node_id.as_str());
                    let connecting_from = self.ui.connecting_from.clone();
                    let status = self.ui.node_status.get(node_id.as_str()).cloned();

                    if let Some(node) = self.pipeline.nodes.iter_mut().find(|n| n.id == *node_id) {
                        let interaction = draw_node(
                            ui,
                            node,
                            is_selected,
                            self.canvas_offset,
                            self.canvas_zoom,
                            status,
                        );

                        if interaction.drag_started {
                            self.snapshot();
                        }

                        if interaction.clicked
                            && !interaction.output_clicked
                            && !interaction.input_clicked
                        {
                            // Snapshot before changing selection so that text
                            // edits made in the properties panel can be undone.
                            if self.ui.selected_node.as_deref() != Some(node_id.as_str()) {
                                self.snapshot();
                            }
                            self.ui.selected_node = Some(node_id.clone());
                            self.ui.connecting_from = None;
                        }
                        if interaction.output_clicked {
                            self.ui.connecting_from = Some(node_id.clone());
                        }
                        if interaction.input_clicked {
                            if let Some(from_id) = connecting_from {
                                if from_id != *node_id {
                                    let to = node_id.clone();
                                    self.snapshot();
                                    self.pipeline.add_edge(from_id, to);
                                    self.ui.connecting_from = None;
                                }
                            }
                        }
                        if interaction.right_clicked {
                            self.snapshot();
                            self.pipeline.remove_node(node_id);
                            self.ui.json_edit_buf.remove(node_id.as_str());
                            if self.ui.selected_node.as_deref() == Some(node_id) {
                                self.ui.selected_node = None;
                            }
                        }
                    }
                }

                painter.text(
                    canvas_rect.right_bottom() - Vec2::new(8.0, 8.0),
                    egui::Align2::RIGHT_BOTTOM,
                    t!("canvas.hint"),
                    FontId::proportional(10.0),
                    Color32::from_white_alpha(60),
                );
            });
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let step = 40.0 * self.canvas_zoom;
        let off_x = self.canvas_offset.x % step;
        let off_y = self.canvas_offset.y % step;
        let col = Color32::from_white_alpha(12);

        let mut x = rect.left() + off_x;
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                egui::Stroke::new(0.5, col),
            );
            x += step;
        }
        let mut y = rect.top() + off_y;
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                egui::Stroke::new(0.5, col),
            );
            y += step;
        }
    }

    // ── Actions ──────────────────────────────────────────────────────────

    fn add_node(&mut self, type_name: &str) {
        let id = self.pipeline.next_id();
        let node_count = self.pipeline.nodes.len() as f32;
        let pos = [
            60.0 + (node_count % 4.0) * (NODE_W + 40.0),
            60.0 + (node_count / 4.0).floor() * (NODE_H + 60.0),
        ];
        let mut node = Node::new(&id, type_name, pos);
        node.config = default_config(type_name);
        self.snapshot();
        self.pipeline.add_node(node);
        self.ui.selected_node = Some(id);
        self.ui
            .log(t!("log.added", node_type = type_name).to_string());
    }

    fn validate_pipeline(&mut self) {
        if self.pipeline.nodes.is_empty() {
            self.ui.log(t!("log.validate_empty").to_string());
            return;
        }
        self.ui.log(
            t!(
                "log.validate_ok",
                name = self.pipeline.name.as_str(),
                nodes = self.pipeline.nodes.len().to_string().as_str(),
                edges = self.pipeline.edges.len().to_string().as_str(),
            )
            .to_string(),
        );

        // Check for isolated nodes (no input AND no output edges)
        for node in &self.pipeline.nodes {
            let has_connection = self
                .pipeline
                .edges
                .iter()
                .any(|e| e.from == node.id || e.to == node.id);
            if !has_connection {
                self.ui
                    .log(format!("[WARN] Node '{}' has no connections", node.label));
            }
        }

        // Cycle detection via Kahn's algorithm (topological sort)
        let mut in_degree: std::collections::HashMap<&str, usize> = self
            .pipeline
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), 0usize))
            .collect();
        for edge in &self.pipeline.edges {
            if let Some(deg) = in_degree.get_mut(edge.to.as_str()) {
                *deg += 1;
            }
        }
        let mut queue: std::collections::VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();
        let mut processed = 0usize;
        while let Some(id) = queue.pop_front() {
            processed += 1;
            for edge in &self.pipeline.edges {
                if edge.from == id {
                    if let Some(deg) = in_degree.get_mut(edge.to.as_str()) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(edge.to.as_str());
                        }
                    }
                }
            }
        }
        if processed < self.pipeline.nodes.len() {
            self.ui.log("[ERROR] Pipeline contains a cycle.".to_owned());
        }
    }

    fn run_pipeline(&mut self, ctx: &egui::Context) {
        if self.ui.pipeline_running {
            return;
        }
        if self.pipeline.nodes.is_empty() {
            self.ui.log(t!("log.run_empty").to_string());
            return;
        }

        // Mark all nodes as running
        for node in &self.pipeline.nodes {
            self.ui
                .node_status
                .insert(node.id.clone(), NodeStatus::Running);
        }

        let (tx, rx) = mpsc::channel::<String>();
        self.run_rx = Some(rx);
        self.ui.pipeline_running = true;
        self.ui
            .log(t!("log.run_start", name = self.pipeline.name.as_str()).to_string());

        let pipeline = self.pipeline.clone();
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");

            rt.block_on(async move {
                match runner::build_and_run(&pipeline).await {
                    Ok(stats) => {
                        let _ = tx.send(
                            t!(
                                "log.run_ok",
                                name = pipeline.name.as_str(),
                                ms = stats.elapsed_ms.to_string().as_str(),
                            )
                            .to_string(),
                        );
                    }
                    Err(e) => {
                        let _ = tx
                            .send(t!("log.run_error", error = e.to_string().as_str()).to_string());
                    }
                }
                let _ = tx.send("__DONE__".into());
            });

            ctx.request_repaint();
        });
    }

    fn open_hpl_dialog(&mut self) {
        if let Some(path) = rfd_pick_file(Some("hpl")) {
            match ajisai_hop_compat::load_pipeline_file(&path) {
                Ok(hop) => {
                    let mut ps = PipelineState::new(&hop.name);
                    for (i, tr) in hop.transforms.iter().enumerate() {
                        let cols = 3usize;
                        let pos = [
                            40.0 + (i % cols) as f32 * (NODE_W + 60.0),
                            40.0 + (i / cols) as f32 * (NODE_H + 80.0),
                        ];
                        let mut node = Node::new(&tr.name, &tr.type_name, pos);
                        node.label = tr.name.clone();
                        node.config = serde_json::to_value(&tr.attributes).unwrap_or_default();
                        ps.add_node(node);
                    }
                    for h in &hop.order {
                        if h.enabled.unwrap_or(true) {
                            ps.add_edge(&h.from, &h.to);
                        }
                    }
                    self.snapshot();
                    self.pipeline = ps;
                    self.ui.current_file = Some(path.clone());
                    self.ui.json_edit_buf.clear();
                    let path_str = path.display().to_string();
                    self.ui
                        .log(t!("log.open_ok", path = path_str.as_str()).to_string());
                }
                Err(e) => {
                    let err_str = e.to_string();
                    self.ui
                        .log(t!("log.open_error", error = err_str.as_str()).to_string());
                }
            }
        }
    }

    fn save_json_dialog(&mut self) {
        if let Some(path) = rfd_save_file("ajisai_pipeline.json", "json") {
            let json = serde_json::to_string_pretty(&self.pipeline).unwrap_or_default();
            let path_str = path.display().to_string();
            if let Err(e) = std::fs::write(&path, json) {
                let err_str = e.to_string();
                self.ui
                    .log(t!("log.save_error", error = err_str.as_str()).to_string());
            } else {
                self.ui
                    .log(t!("log.save_ok", path = path_str.as_str()).to_string());
            }
        }
    }

    fn save_hpl_dialog(&mut self) {
        let default_name = format!("{}.hpl", self.pipeline.name.replace(' ', "_"));
        if let Some(path) = rfd_save_file(&default_name, "hpl") {
            match ajisai_hop_compat::write_hpl_file(&self.pipeline_to_hop(), &path) {
                Ok(()) => {
                    self.ui.current_file = Some(path.clone());
                    let path_str = path.display().to_string();
                    self.ui
                        .log(t!("log.save_ok", path = path_str.as_str()).to_string());
                }
                Err(e) => {
                    let err_str = e.to_string();
                    self.ui
                        .log(t!("log.save_error", error = err_str.as_str()).to_string());
                }
            }
        }
    }

    fn save_current_file(&mut self) {
        if let Some(path) = self.ui.current_file.clone() {
            match ajisai_hop_compat::write_hpl_file(&self.pipeline_to_hop(), &path) {
                Ok(()) => {
                    let path_str = path.display().to_string();
                    self.ui
                        .log(t!("log.save_ok", path = path_str.as_str()).to_string());
                }
                Err(e) => {
                    let err_str = e.to_string();
                    self.ui
                        .log(t!("log.save_error", error = err_str.as_str()).to_string());
                }
            }
        }
    }

    fn pipeline_to_hop(&self) -> ajisai_hop_compat::HopPipeline {
        use ajisai_hop_compat::model::{HopHop, HopPipeline, HopPipelineInfo, HopTransform};
        let transforms = self
            .pipeline
            .nodes
            .iter()
            .map(|n| HopTransform {
                name: n.label.clone(),
                type_name: n.type_name.clone(),
                description: None,
                xloc: Some(n.pos[0] as i32),
                yloc: Some(n.pos[1] as i32),
                attributes: match &n.config {
                    serde_json::Value::Object(m) => {
                        m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                    }
                    _ => Default::default(),
                },
            })
            .collect();

        // Build id → label map so hop elements reference display names
        let id_to_label: std::collections::HashMap<&str, &str> = self
            .pipeline
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n.label.as_str()))
            .collect();

        let order = self
            .pipeline
            .edges
            .iter()
            .map(|e| HopHop {
                from: id_to_label
                    .get(e.from.as_str())
                    .copied()
                    .unwrap_or(e.from.as_str())
                    .to_owned(),
                to: id_to_label
                    .get(e.to.as_str())
                    .copied()
                    .unwrap_or(e.to.as_str())
                    .to_owned(),
                enabled: Some(true),
            })
            .collect();

        HopPipeline {
            name: self.pipeline.name.clone(),
            transforms,
            order,
            info: HopPipelineInfo { description: None },
        }
    }
}

impl eframe::App for AjisaiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_engine();
        self.handle_shortcuts(ctx);

        // Reflect current filename in window title
        let title = match &self.ui.current_file {
            Some(p) => format!(
                "{} — {}",
                self.pipeline.name,
                p.file_name().unwrap_or_default().to_string_lossy()
            ),
            None => self.pipeline.name.clone(),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        if self.ui.pipeline_running {
            ctx.request_repaint();
        }

        self.menu_bar(ctx);
        self.log_panel(ctx);
        self.left_panel(ctx);
        self.right_panel(ctx);
        self.canvas(ctx);
    }
}

fn default_config(type_name: &str) -> serde_json::Value {
    match type_name {
        "CsvFileInput" => {
            serde_json::json!({ "filename": "input.csv",  "delimiter": ",", "has_header": true })
        }
        "CsvFileOutput" => {
            serde_json::json!({ "filename": "output.csv", "delimiter": ",", "header": true })
        }
        "JsonFileInput" => serde_json::json!({ "filename": "input.json",  "format": "array" }),
        "JsonFileOutput" => {
            serde_json::json!({ "filename": "output.json", "format": "array", "pretty": true })
        }
        "TableInput" => {
            serde_json::json!({ "connection_url": "sqlite://data.db", "sql": "SELECT * FROM table_name" })
        }
        "TableOutput" => {
            serde_json::json!({ "connection_url": "sqlite://data.db", "table": "table_name", "mode": "insert", "batch_size": 0 })
        }
        "FilterRows" => serde_json::json!({ "condition": null }),
        "SelectValues" => serde_json::json!({ "fields": [] }),
        "SortRows" => serde_json::json!({ "keys": [{ "field": "id", "ascending": true }] }),
        "AddConstants" => serde_json::json!({ "fields": [] }),
        "CalculatorStep" => serde_json::json!({ "calculations": [] }),
        "StreamLookup" => {
            serde_json::json!({ "lookup_transform": "", "key_field": "id", "lookup_key_field": "id", "return_fields": [] })
        }
        "MergeJoin" => {
            serde_json::json!({ "left_key": "id", "right_key": "id", "join_type": "inner", "right_prefix": "r_" })
        }
        "Deduplicate" => serde_json::json!({ "key_fields": [] }),
        "DatabaseLookup" => {
            serde_json::json!({ "connection_url": "sqlite://data.db", "sql": "SELECT * FROM t WHERE id = ?", "key_field": "id", "return_fields": [] })
        }
        _ => serde_json::Value::Object(serde_json::Map::new()),
    }
}

fn rfd_pick_file(_ext: Option<&str>) -> Option<std::path::PathBuf> {
    #[cfg(feature = "rfd")]
    {
        let mut dialog = rfd::FileDialog::new();
        if let Some(e) = _ext {
            dialog = dialog.add_filter(e, &[e]);
        }
        dialog.pick_file()
    }
    #[cfg(not(feature = "rfd"))]
    None
}

fn rfd_save_file(_default_name: &str, _ext: &str) -> Option<std::path::PathBuf> {
    #[cfg(feature = "rfd")]
    {
        rfd::FileDialog::new()
            .set_file_name(_default_name)
            .add_filter(_ext, &[_ext])
            .save_file()
    }
    #[cfg(not(feature = "rfd"))]
    None
}
