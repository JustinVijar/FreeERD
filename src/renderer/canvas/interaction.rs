use egui::{Pos2, Rect};

#[allow(dead_code)]

pub fn get_label_position_world(
    edge_route: &crate::renderer::layout::EdgeRoute,
    cached_positions: &[Option<(f32, f32)>],
    label_offsets: &[Option<(f32, f32)>],
    idx: usize,
) -> Option<(f32, f32)> {
    if let Some(Some((offset_x, offset_y))) = label_offsets.get(idx) {
        // Label has custom position
        if let Some((mid_x, mid_y)) = get_edge_middle_point_simple(&edge_route.points) {
            Some((mid_x + offset_x, mid_y + offset_y))
        } else {
            None
        }
    } else {
        // Label uses automatic positioning
        // Use cached position if available, otherwise use middle point
        cached_positions.get(idx).and_then(|p| *p)
            .or_else(|| get_edge_middle_point_simple(&edge_route.points))
    }
}

pub fn get_edge_middle_point_simple(points: &[crate::renderer::layout::Point]) -> Option<(f32, f32)> {
    if points.len() < 2 {
        return None;
    }
    
    let mut total_length = 0.0;
    let mut segment_lengths = Vec::new();
    
    for i in 1..points.len() {
        let dx = points[i].x - points[i - 1].x;
        let dy = points[i].y - points[i - 1].y;
        let length = (dx * dx + dy * dy).sqrt();
        segment_lengths.push(length);
        total_length += length;
    }
    
    let target_length = total_length / 2.0;
    let mut accumulated = 0.0;
    
    for i in 0..segment_lengths.len() {
        let seg_len = segment_lengths[i];
        if accumulated + seg_len >= target_length {
            let t = (target_length - accumulated) / seg_len;
            let p1 = &points[i];
            let p2 = &points[i + 1];
            let mid_x = p1.x + (p2.x - p1.x) * t;
            let mid_y = p1.y + (p2.y - p1.y) * t;
            return Some((mid_x, mid_y));
        }
        accumulated += seg_len;
    }
    
    None
}

pub fn check_title_click(
    pos: Pos2,
    title_screen: Pos2,
    title: &str,
    zoom: f32,
) -> bool {
    let title_width = (title.len() as f32 * 20.0 + 40.0) * zoom; // H1 is bigger
    let title_height = 50.0 * zoom;
    let title_rect = Rect::from_center_size(
        title_screen, 
        egui::vec2(title_width, title_height)
    );
    
    title_rect.contains(pos)
}

pub fn check_label_click(
    pos: Pos2,
    label_screen: Pos2,
    label_text: &str,
    zoom: f32,
) -> bool {
    // Estimate based on text length
    let label_width = (label_text.len() as f32 * 7.0 + 40.0) * zoom;
    let label_height = 28.0 * zoom;
    let padding = egui::vec2(12.0 * zoom, 8.0 * zoom);
    
    let label_rect = Rect::from_center_size(
        label_screen, 
        egui::vec2(label_width + padding.x * 2.0, label_height + padding.y * 2.0)
    );
    
    label_rect.contains(pos)
}

pub fn check_table_click(
    pos: Pos2,
    layout: &crate::renderer::layout::NodeLayout,
    zoom: f32,
    pan_offset: Pos2,
) -> bool {
    let screen_pos = Pos2::new(
        layout.position.x * zoom + pan_offset.x,
        layout.position.y * zoom + pan_offset.y,
    );
    let screen_size = egui::vec2(layout.size.width * zoom, layout.size.height * zoom);
    let table_rect = Rect::from_min_size(screen_pos, screen_size);
    
    table_rect.contains(pos)
}
