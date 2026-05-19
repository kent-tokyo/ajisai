use crate::state::{Node, NodeStatus, PipelineState};
use egui::{Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

pub const NODE_W: f32 = 160.0;
pub const NODE_H: f32 = 48.0;
pub const PORT_R: f32 = 6.0;

const COL_NODE_BG: Color32 = Color32::from_rgb(45, 50, 65);
const COL_NODE_SEL: Color32 = Color32::from_rgb(70, 130, 200);
const COL_NODE_BORDER: Color32 = Color32::from_rgb(90, 100, 120);
const COL_PORT: Color32 = Color32::from_rgb(120, 200, 120);
const COL_EDGE: Color32 = Color32::from_rgb(160, 160, 200);
const COL_EDGE_ACTIVE: Color32 = Color32::from_rgb(100, 200, 255);
const COL_LABEL: Color32 = Color32::WHITE;
const COL_TYPE: Color32 = Color32::from_rgb(160, 170, 190);

/// Header accent color by transform category
pub fn category_color(type_name: &str) -> Color32 {
    match type_name {
        "CsvFileInput" | "CsvFileOutput" | "JsonFileInput" | "JsonFileOutput" | "TableInput"
        | "TableOutput" => Color32::from_rgb(200, 120, 40), // orange — I/O
        "StreamLookup" | "MergeJoin" | "DatabaseLookup" => {
            Color32::from_rgb(140, 80, 200) // purple — Join/Lookup
        }
        _ => COL_NODE_SEL, // blue — Transform
    }
}

/// Screen position of a node's output port
pub fn output_port(node: &Node, offset: Vec2, zoom: f32) -> Pos2 {
    Pos2::new(
        node.pos[0] * zoom + offset.x + NODE_W * zoom,
        node.pos[1] * zoom + offset.y + NODE_H * zoom * 0.5,
    )
}

/// Screen position of a node's input port
pub fn input_port(node: &Node, offset: Vec2, zoom: f32) -> Pos2 {
    Pos2::new(
        node.pos[0] * zoom + offset.x,
        node.pos[1] * zoom + offset.y + NODE_H * zoom * 0.5,
    )
}

/// Draw all edges as bezier curves
pub fn draw_edges(painter: &Painter, pipeline: &PipelineState, offset: Vec2, zoom: f32) {
    for edge in &pipeline.edges {
        let Some(from_node) = pipeline.node(&edge.from) else {
            continue;
        };
        let Some(to_node) = pipeline.node(&edge.to) else {
            continue;
        };

        let p0 = output_port(from_node, offset, zoom);
        let p3 = input_port(to_node, offset, zoom);
        let ctrl_dist = ((p3.x - p0.x).abs() * 0.5).max(60.0 * zoom);
        let p1 = Pos2::new(p0.x + ctrl_dist, p0.y);
        let p2 = Pos2::new(p3.x - ctrl_dist, p3.y);

        draw_bezier(painter, p0, p1, p2, p3, Stroke::new(2.0, COL_EDGE));
    }
}

/// Draw an in-progress edge from a port to the cursor
pub fn draw_connecting_edge(painter: &Painter, from: Pos2, to: Pos2) {
    let ctrl_dist = ((to.x - from.x).abs() * 0.5).max(60.0);
    let p1 = Pos2::new(from.x + ctrl_dist, from.y);
    let p2 = Pos2::new(to.x - ctrl_dist, to.y);
    draw_bezier(painter, from, p1, p2, to, Stroke::new(2.0, COL_EDGE_ACTIVE));
}

fn draw_bezier(painter: &Painter, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, stroke: Stroke) {
    const STEPS: usize = 32;
    let mut prev = p0;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let u = 1.0 - t;
        let pt = p0 * (u * u * u)
            + p1.to_vec2() * (3.0 * u * u * t)
            + p2.to_vec2() * (3.0 * u * t * t)
            + p3.to_vec2() * (t * t * t);
        painter.line_segment([prev, pt], stroke);
        prev = pt;
    }
}

/// Draw a single node with interaction handling.
/// `status` controls the execution-state dot painted in the top-right corner.
pub fn draw_node(
    ui: &mut egui::Ui,
    node: &mut Node,
    is_selected: bool,
    offset: Vec2,
    zoom: f32,
    status: Option<NodeStatus>,
) -> NodeInteraction {
    let nw = NODE_W * zoom;
    let nh = NODE_H * zoom;
    let pr = PORT_R * zoom;

    let rect = Rect::from_min_size(
        Pos2::new(node.pos[0] * zoom + offset.x, node.pos[1] * zoom + offset.y),
        Vec2::new(nw, nh),
    );

    let resp = ui.allocate_rect(rect, Sense::click_and_drag());

    let drag_started = resp.drag_started();
    if resp.dragged() {
        let delta = resp.drag_delta();
        node.pos[0] += delta.x / zoom;
        node.pos[1] += delta.y / zoom;
    }

    let painter = ui.painter();

    // Shadow
    painter.rect_filled(
        rect.translate(Vec2::new(3.0, 3.0)),
        6.0 * zoom,
        Color32::from_black_alpha(80),
    );

    // Body
    let border_color = if is_selected {
        COL_NODE_SEL
    } else {
        COL_NODE_BORDER
    };
    painter.rect_filled(rect, 6.0 * zoom, COL_NODE_BG);
    painter.rect_stroke(
        rect,
        6.0 * zoom,
        Stroke::new(if is_selected { 2.0 } else { 1.0 }, border_color),
        StrokeKind::Middle,
    );

    // Header accent bar (category-colored)
    let header_rect = Rect::from_min_size(rect.min, Vec2::new(nw, 4.0 * zoom));
    let corner_r = egui::CornerRadius {
        nw: (6.0 * zoom) as u8,
        ne: (6.0 * zoom) as u8,
        sw: 0,
        se: 0,
    };
    painter.rect_filled(header_rect, corner_r, category_color(&node.type_name));

    let label_size = (13.0 * zoom).max(6.0);
    let type_size = (10.0 * zoom).max(5.0);

    // Label + type name
    painter.text(
        rect.center() - Vec2::new(0.0, 6.0 * zoom),
        egui::Align2::CENTER_CENTER,
        &node.label,
        FontId::proportional(label_size),
        COL_LABEL,
    );
    painter.text(
        rect.center() + Vec2::new(0.0, 9.0 * zoom),
        egui::Align2::CENTER_CENTER,
        node.display_name(),
        FontId::proportional(type_size),
        COL_TYPE,
    );

    // Execution status dot (top-right corner)
    if let Some(ref s) = status {
        let dot_color = match s {
            NodeStatus::Running => {
                let t = ui.ctx().input(|i| i.time) as f32;
                let pulse = (t * 4.0).sin() * 0.5 + 0.5;
                Color32::from_rgb(
                    (200.0 + 55.0 * pulse) as u8,
                    (160.0 + 40.0 * pulse) as u8,
                    0,
                )
            }
            NodeStatus::Done => Color32::from_rgb(80, 210, 80),
            NodeStatus::Error => Color32::from_rgb(230, 70, 70),
            NodeStatus::Idle => Color32::TRANSPARENT,
        };
        if *s != NodeStatus::Idle {
            let dot = Pos2::new(rect.right() - 8.0 * zoom, rect.top() + 8.0 * zoom);
            painter.circle_filled(dot, 5.0 * zoom, dot_color);
            painter.circle_stroke(dot, 5.0 * zoom, Stroke::new(1.0, Color32::BLACK));
        }
    }

    // Ports
    let in_pos = input_port(node, offset, zoom);
    let out_pos = output_port(node, offset, zoom);
    painter.circle_filled(in_pos, pr, COL_PORT);
    painter.circle_stroke(in_pos, pr, Stroke::new(1.0, Color32::WHITE));
    painter.circle_filled(out_pos, pr, COL_PORT);
    painter.circle_stroke(out_pos, pr, Stroke::new(1.0, Color32::WHITE));

    // Port hit areas
    let out_rect = Rect::from_center_size(out_pos, Vec2::splat(pr * 2.5));
    let in_rect = Rect::from_center_size(in_pos, Vec2::splat(pr * 2.5));
    let out_resp = ui.allocate_rect(out_rect, Sense::click());
    let in_resp = ui.allocate_rect(in_rect, Sense::click());

    NodeInteraction {
        clicked: resp.clicked(),
        output_clicked: out_resp.clicked(),
        input_clicked: in_resp.clicked(),
        drag_started,
        right_clicked: resp.secondary_clicked(),
    }
}

#[derive(Default)]
pub struct NodeInteraction {
    pub clicked: bool,
    pub output_clicked: bool,
    pub input_clicked: bool,
    pub drag_started: bool,
    pub right_clicked: bool,
}
