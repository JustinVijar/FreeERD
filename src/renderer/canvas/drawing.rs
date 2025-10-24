use egui::{Color32, Stroke, Pos2, Rect, FontId, Align2};
use crate::renderer::graph::ErdGraph;
use crate::renderer::layout::LayoutEngine;
use petgraph::graph::NodeIndex;

pub struct DrawingContext<'a> {
    pub erd_graph: &'a ErdGraph,
    pub layout_engine: &'a LayoutEngine,
    pub zoom: f32,
    pub pan_offset: Pos2,
    pub selected_table: Option<NodeIndex>,
    #[allow(dead_code)]
    pub label_offsets: &'a [Option<(f32, f32)>],
    #[allow(dead_code)]
    pub cached_label_positions: &'a [Option<(f32, f32)>],
}

impl<'a> DrawingContext<'a> {
    pub fn transform_point(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(
            x * self.zoom + self.pan_offset.x,
            y * self.zoom + self.pan_offset.y,
        )
    }
}

pub fn draw_table(ctx: &DrawingContext, ui: &mut egui::Ui, node_idx: NodeIndex) {
    let table = &ctx.erd_graph.graph()[node_idx];
    let layout = ctx.layout_engine.get_node_layout(node_idx).unwrap();
    
    let pos = ctx.transform_point(layout.position.x, layout.position.y);
    let size = egui::vec2(layout.size.width * ctx.zoom, layout.size.height * ctx.zoom);
    
    let rect = Rect::from_min_size(pos, size);
    
    // Check if this table is selected
    let is_selected = ctx.selected_table == Some(node_idx);
    
    // Draw table background with shadow
    let shadow_rect = rect.translate(egui::vec2(3.0, 3.0));
    ui.painter().rect_filled(shadow_rect, 8.0, Color32::from_black_alpha(30));
    ui.painter().rect_filled(rect, 8.0, Color32::WHITE);
    
    // Use different colors for selected vs normal tables
    let border_color = if is_selected {
        Color32::from_rgb(255, 200, 0) // Golden yellow for selected
    } else {
        Color32::from_rgb(52, 152, 219) // Default blue
    };
    let border_width = if is_selected { 3.5 } else { 2.0 };
    
    ui.painter().rect_stroke(rect, 8.0, Stroke::new(border_width * ctx.zoom, border_color));
    
    // Draw header with glow effect if selected
    if is_selected {
        // Draw outer glow
        let glow_rect = rect.expand(4.0 * ctx.zoom);
        ui.painter().rect_stroke(glow_rect, 10.0, Stroke::new(2.0 * ctx.zoom, Color32::from_rgba_premultiplied(255, 200, 0, 100)));
    }
    
    let header_height = 40.0 * ctx.zoom;
    let header_rect = Rect::from_min_size(rect.min, egui::vec2(size.x, header_height));
    let header_color = if is_selected {
        Color32::from_rgb(255, 180, 0) // Brighter for selected
    } else {
        Color32::from_rgb(52, 152, 219)
    };
    ui.painter().rect_filled(header_rect, 8.0, header_color);
    
    // Draw table name
    let text_pos = header_rect.center();
    ui.painter().text(
        text_pos,
        Align2::CENTER_CENTER,
        &table.name,
        FontId::proportional(16.0 * ctx.zoom),
        Color32::WHITE,
    );
    
    // Draw columns
    let mut y_offset = header_height + 10.0 * ctx.zoom;
    let row_height = 25.0 * ctx.zoom;
    
    for column in &table.columns {
        let col_pos = Pos2::new(rect.min.x + 15.0 * ctx.zoom, rect.min.y + y_offset);
        
        // Column name
        let col_name = if column.attributes.contains(&"PK".to_string()) {
            format!("🔑 {}", column.name)
        } else {
            column.name.clone()
        };
        
        ui.painter().text(
            col_pos,
            Align2::LEFT_TOP,
            &col_name,
            FontId::proportional(12.0 * ctx.zoom),
            Color32::from_rgb(44, 62, 80),
        );
        
        // Data type and attributes
        let type_text = if column.attributes.is_empty() {
            column.data_type.clone()
        } else {
            format!("{} [{}]", column.data_type, column.attributes.join(","))
        };
        
        // Right-align the type text with padding from the right edge
        let type_pos = Pos2::new(rect.max.x - 15.0 * ctx.zoom, rect.min.y + y_offset);
        ui.painter().text(
            type_pos,
            Align2::RIGHT_TOP,
            &type_text,
            FontId::proportional(10.0 * ctx.zoom),
            Color32::from_rgb(127, 140, 141),
        );
        
        y_offset += row_height;
    }
}

pub fn draw_title(ctx: &DrawingContext, ui: &mut egui::Ui, title: &str, title_position: (f32, f32), drag_target: &super::types::DragTarget) {
    if title.is_empty() {
        return;
    }
    
    // Transform title position to screen coordinates
    let title_screen = ctx.transform_point(title_position.0, title_position.1);
    
    // Draw title background
    let font_size = 32.0 * ctx.zoom; // H1 size
    let padding = egui::vec2(20.0 * ctx.zoom, 12.0 * ctx.zoom);
    
    // Estimate text size
    let char_width = font_size * 0.6; // Approximate character width
    let text_width = title.len() as f32 * char_width;
    let text_height = font_size;
    
    let bg_rect = Rect::from_center_size(
        title_screen,
        egui::vec2(text_width + padding.x * 2.0, text_height + padding.y * 2.0)
    );
    
    // Draw background with slight shadow
    let shadow_rect = bg_rect.translate(egui::vec2(2.0, 2.0));
    ui.painter().rect_filled(shadow_rect, 6.0, Color32::from_black_alpha(40));
    
    // Background color based on drag state
    let bg_color = if *drag_target == super::types::DragTarget::Title {
        Color32::from_rgb(70, 130, 180) // Highlighted when dragging
    } else {
        Color32::from_rgb(52, 152, 219) // Normal blue
    };
    
    ui.painter().rect_filled(bg_rect, 6.0, bg_color);
    ui.painter().rect_stroke(bg_rect, 6.0, Stroke::new(2.0 * ctx.zoom, Color32::WHITE));
    
    // Draw title text
    ui.painter().text(
        title_screen,
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(font_size),
        Color32::WHITE,
    );
}

pub fn draw_edges(ctx: &DrawingContext, ui: &mut egui::Ui) {
    use crate::renderer::graph::RelationType;
    
    for edge_route in ctx.layout_engine.get_edge_routes() {
        if edge_route.points.len() < 2 {
            continue;
        }
        
        // Check if this edge is connected to the selected table
        let is_connected_to_selected = if let Some(selected_idx) = ctx.selected_table {
            let selected_table_name = &ctx.erd_graph.graph()[selected_idx].name;
            edge_route.from_table == *selected_table_name || edge_route.to_table == *selected_table_name
        } else {
            false
        };
        
        // Use brighter color and thicker stroke for selected connections
        let color = if is_connected_to_selected {
            Color32::from_rgb(255, 200, 0) // Golden yellow
        } else {
            Color32::from_rgb(52, 73, 94) // Default dark blue
        };
        let stroke_width = if is_connected_to_selected { 3.0 } else { 2.0 };
        let stroke = Stroke::new(stroke_width * ctx.zoom, color);
        
        let mut prev_point = None;
        
        // Draw the edge lines
        for point in &edge_route.points {
            let current = ctx.transform_point(point.x, point.y);
            
            if let Some(prev) = prev_point {
                ui.painter().line_segment([prev, current], stroke);
            }
            
            prev_point = Some(current);
        }
        
        // Draw relationship markers based on type
        // Always use the first and last points (which are at table boundaries)
        let start_idx = 0;
        let end_idx = edge_route.points.len() - 1;
        
        // Draw markers based on relationship type
        match edge_route.relationship_type {
            RelationType::OneToOne => {
                draw_one_marker(ctx, ui, &edge_route.points, start_idx);
                draw_one_marker(ctx, ui, &edge_route.points, end_idx);
            }
            RelationType::OneToMany => {
                draw_one_marker(ctx, ui, &edge_route.points, start_idx);
                draw_many_marker(ctx, ui, &edge_route.points, end_idx);
            }
            RelationType::ManyToOne => {
                draw_many_marker(ctx, ui, &edge_route.points, start_idx);
                draw_one_marker(ctx, ui, &edge_route.points, end_idx);
            }
            RelationType::ManyToMany => {
                draw_many_marker(ctx, ui, &edge_route.points, start_idx);
                draw_many_marker(ctx, ui, &edge_route.points, end_idx);
            }
        }
    }
}

/// Draw "one" marker (single perpendicular line)
pub fn draw_one_marker(ctx: &DrawingContext, ui: &mut egui::Ui, points: &[crate::renderer::layout::Point], idx: usize) {
    if points.len() < 2 {
        return;
    }
    
    let (pos, direction) = if idx == 0 {
        // At start, direction is from first to second point
        let p0 = ctx.transform_point(points[0].x, points[0].y);
        let p1 = ctx.transform_point(points[1].x, points[1].y);
        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        (p0, egui::vec2(dx / len, dy / len))
    } else {
        // At end, direction is from second-to-last to last point
        let p_prev = ctx.transform_point(points[idx - 1].x, points[idx - 1].y);
        let p_last = ctx.transform_point(points[idx].x, points[idx].y);
        let dx = p_last.x - p_prev.x;
        let dy = p_last.y - p_prev.y;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        (p_last, egui::vec2(dx / len, dy / len))
    };
    
    // Perpendicular direction
    let perp = egui::vec2(-direction.y, direction.x);
    let size = 10.0 * ctx.zoom;
    
    // Draw perpendicular line with thicker stroke for visibility
    ui.painter().line_segment(
        [pos + perp * size, pos - perp * size],
        Stroke::new(2.5 * ctx.zoom, Color32::from_rgb(52, 73, 94)),
    );
}

/// Draw "many" marker (crow's foot - three lines)
pub fn draw_many_marker(ctx: &DrawingContext, ui: &mut egui::Ui, points: &[crate::renderer::layout::Point], idx: usize) {
    if points.len() < 2 {
        return;
    }
    
    let (pos, direction) = if idx == 0 {
        // At start, direction is from first to second point
        let p0 = ctx.transform_point(points[0].x, points[0].y);
        let p1 = ctx.transform_point(points[1].x, points[1].y);
        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        (p0, egui::vec2(dx / len, dy / len))
    } else {
        // At end, direction is from second-to-last to last point
        let p_prev = ctx.transform_point(points[idx - 1].x, points[idx - 1].y);
        let p_last = ctx.transform_point(points[idx].x, points[idx].y);
        let dx = p_last.x - p_prev.x;
        let dy = p_last.y - p_prev.y;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        (p_last, egui::vec2(dx / len, dy / len))
    };
    
    // Perpendicular direction
    let perp = egui::vec2(-direction.y, direction.x);
    let size = 12.0 * ctx.zoom;
    let back_offset = 20.0 * ctx.zoom;
    
    // Base point (back from the endpoint along the edge direction)
    let base = if idx == 0 {
        pos + direction * back_offset
    } else {
        pos - direction * back_offset
    };
    
    // Draw three lines forming crow's foot with thicker strokes for visibility
    let crow_stroke = Stroke::new(2.5 * ctx.zoom, Color32::from_rgb(52, 73, 94));
    
    // Center line (pointing to the connection point)
    ui.painter().line_segment([base, pos], crow_stroke);
    
    // Left and right lines (spreading out from base)
    ui.painter().line_segment([base, pos + perp * size], crow_stroke);
    ui.painter().line_segment([base, pos - perp * size], crow_stroke);
}
