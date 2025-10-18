use egui::{Color32, Stroke, Pos2, Rect, FontId, Align2};
use super::graph::ErdGraph;
use super::layout::LayoutEngine;
use petgraph::graph::NodeIndex;

#[derive(Debug, Clone, PartialEq)]
enum DragTarget {
    None,
    Table(NodeIndex),
    Label(usize), // Index into edge_routes
    Title,
}

pub struct ErdCanvas {
    erd_graph: ErdGraph,
    layout_engine: LayoutEngine,
    zoom: f32,
    pan_offset: Pos2,
    initial_pan_set: bool,
    drag_target: DragTarget,
    drag_offset: Pos2,
    label_offsets: Vec<Option<(f32, f32)>>, // Custom offsets for labels in world coordinates
    cached_label_positions: Vec<Option<(f32, f32)>>, // Cached actual label positions in world coordinates
    selected_table: Option<NodeIndex>, // Currently selected table for highlighting
    title: String,
    title_position: (f32, f32), // Title position in world coordinates
}

impl ErdCanvas {
    pub fn new(erd_graph: ErdGraph, title: String) -> Self {
        let mut layout_engine = LayoutEngine::new();
        layout_engine.compute_layout(&erd_graph);
        
        // Initialize label offsets (None means use automatic positioning)
        let label_count = layout_engine.get_edge_routes().len();
        let label_offsets = vec![None; label_count];
        
        // Calculate title position at the top center of the graph
        let mut min_y = f32::MAX;
        let mut center_x = 0.0;
        let mut count = 0;
        
        for node_idx in erd_graph.graph().node_indices() {
            if let Some(layout) = layout_engine.get_node_layout(node_idx) {
                min_y = min_y.min(layout.position.y);
                center_x += layout.position.x + layout.size.width / 2.0;
                count += 1;
            }
        }
        
        if count > 0 {
            center_x /= count as f32;
        }
        
        // Position title above the topmost table
        let title_position = (center_x, min_y - 80.0);
        
        Self {
            erd_graph,
            layout_engine,
            zoom: 1.0,
            pan_offset: Pos2::ZERO,
            initial_pan_set: false,
            drag_target: DragTarget::None,
            drag_offset: Pos2::ZERO,
            label_offsets: label_offsets.clone(),
            cached_label_positions: vec![None; label_count],
            selected_table: None,
            title,
            title_position,
        }
    }
    
    /// Calculate the bounding box of all nodes
    fn calculate_bounds(&self) -> Option<Rect> {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        for node_idx in self.erd_graph.graph().node_indices() {
            if let Some(layout) = self.layout_engine.get_node_layout(node_idx) {
                min_x = min_x.min(layout.position.x);
                min_y = min_y.min(layout.position.y);
                max_x = max_x.max(layout.position.x + layout.size.width);
                max_y = max_y.max(layout.position.y + layout.size.height);
            }
        }
        
        if min_x < f32::MAX {
            Some(Rect::from_min_max(
                Pos2::new(min_x, min_y),
                Pos2::new(max_x, max_y),
            ))
        } else {
            None
        }
    }
    
    /// Center the graph in the viewport
    fn center_graph(&mut self, viewport_size: egui::Vec2) {
        if let Some(bounds) = self.calculate_bounds() {
            let graph_center = bounds.center();
            let viewport_center = Pos2::new(viewport_size.x / 2.0, viewport_size.y / 2.0);
            
            // Calculate offset to center the graph
            self.pan_offset = viewport_center - graph_center.to_vec2();
            
            // Add some padding
            let padding = 50.0;
            
            // Calculate zoom to fit graph in viewport with padding
            let graph_width = bounds.width();
            let graph_height = bounds.height();
            
            if graph_width > 0.0 && graph_height > 0.0 {
                let zoom_x = (viewport_size.x - padding * 2.0) / graph_width;
                let zoom_y = (viewport_size.y - padding * 2.0) / graph_height;
                self.zoom = zoom_x.min(zoom_y).min(1.0); // Don't zoom in beyond 1.0
                
                // Recalculate pan offset with zoom
                let scaled_graph_center = Pos2::new(
                    graph_center.x * self.zoom,
                    graph_center.y * self.zoom,
                );
                self.pan_offset = viewport_center - scaled_graph_center.to_vec2();
            }
        }
    }
    
    fn draw_table(&self, ui: &mut egui::Ui, node_idx: petgraph::graph::NodeIndex) {
        let table = &self.erd_graph.graph()[node_idx];
        let layout = self.layout_engine.get_node_layout(node_idx).unwrap();
        
        let pos = self.transform_point(layout.position.x, layout.position.y);
        let size = egui::vec2(layout.size.width * self.zoom, layout.size.height * self.zoom);
        
        let rect = Rect::from_min_size(pos, size);
        
        // Check if this table is selected
        let is_selected = self.selected_table == Some(node_idx);
        
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
        
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(border_width * self.zoom, border_color));
        
        // Draw header with glow effect if selected
        if is_selected {
            // Draw outer glow
            let glow_rect = rect.expand(4.0 * self.zoom);
            ui.painter().rect_stroke(glow_rect, 10.0, Stroke::new(2.0 * self.zoom, Color32::from_rgba_premultiplied(255, 200, 0, 100)));
        }
        
        let header_height = 40.0 * self.zoom;
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
            FontId::proportional(16.0 * self.zoom),
            Color32::WHITE,
        );
        
        // Draw columns
        let mut y_offset = header_height + 10.0 * self.zoom;
        let row_height = 25.0 * self.zoom;
        
        for column in &table.columns {
            let col_pos = Pos2::new(rect.min.x + 15.0 * self.zoom, rect.min.y + y_offset);
            
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
                FontId::proportional(12.0 * self.zoom),
                Color32::from_rgb(44, 62, 80),
            );
            
            // Data type and attributes
            let type_text = if column.attributes.is_empty() {
                column.data_type.clone()
            } else {
                format!("{} [{}]", column.data_type, column.attributes.join(","))
            };
            
            // Right-align the type text with padding from the right edge
            let type_pos = Pos2::new(rect.max.x - 15.0 * self.zoom, rect.min.y + y_offset);
            ui.painter().text(
                type_pos,
                Align2::RIGHT_TOP,
                &type_text,
                FontId::proportional(10.0 * self.zoom),
                Color32::from_rgb(127, 140, 141),
            );
            
            y_offset += row_height;
        }
    }
    
    fn draw_title(&self, ui: &mut egui::Ui) {
        if self.title.is_empty() {
            return;
        }
        
        // Transform title position to screen coordinates
        let title_screen = self.transform_point(self.title_position.0, self.title_position.1);
        
        // Draw title background
        let font_size = 32.0 * self.zoom; // H1 size
        let padding = egui::vec2(20.0 * self.zoom, 12.0 * self.zoom);
        
        // Estimate text size
        let char_width = font_size * 0.6; // Approximate character width
        let text_width = self.title.len() as f32 * char_width;
        let text_height = font_size;
        
        let bg_rect = Rect::from_center_size(
            title_screen,
            egui::vec2(text_width + padding.x * 2.0, text_height + padding.y * 2.0)
        );
        
        // Draw background with slight shadow
        let shadow_rect = bg_rect.translate(egui::vec2(2.0, 2.0));
        ui.painter().rect_filled(shadow_rect, 6.0, Color32::from_black_alpha(40));
        
        // Background color based on drag state
        let bg_color = if self.drag_target == DragTarget::Title {
            Color32::from_rgb(70, 130, 180) // Highlighted when dragging
        } else {
            Color32::from_rgb(52, 152, 219) // Normal blue
        };
        
        ui.painter().rect_filled(bg_rect, 6.0, bg_color);
        ui.painter().rect_stroke(bg_rect, 6.0, Stroke::new(2.0 * self.zoom, Color32::WHITE));
        
        // Draw title text
        ui.painter().text(
            title_screen,
            Align2::CENTER_CENTER,
            &self.title,
            FontId::proportional(font_size),
            Color32::WHITE,
        );
    }
    
    fn draw_edges(&self, ui: &mut egui::Ui) {
        use super::graph::RelationType;
        
        for edge_route in self.layout_engine.get_edge_routes() {
            if edge_route.points.len() < 2 {
                continue;
            }
            
            // Check if this edge is connected to the selected table
            let is_connected_to_selected = if let Some(selected_idx) = self.selected_table {
                let selected_table_name = &self.erd_graph.graph()[selected_idx].name;
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
            let stroke = Stroke::new(stroke_width * self.zoom, color);
            
            let mut prev_point = None;
            
            // Draw the edge lines
            for point in &edge_route.points {
                let current = self.transform_point(point.x, point.y);
                
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
                    // println!("  -> Drawing OneToOne markers");
                    self.draw_one_marker(ui, &edge_route.points, start_idx);
                    self.draw_one_marker(ui, &edge_route.points, end_idx);
                }
                RelationType::OneToMany => {
                    // println!("  -> Drawing OneToMany markers");
                    self.draw_one_marker(ui, &edge_route.points, start_idx);
                    self.draw_many_marker(ui, &edge_route.points, end_idx);
                }
                RelationType::ManyToOne => {
                    // println!("  -> Drawing ManyToOne markers");
                    self.draw_many_marker(ui, &edge_route.points, start_idx);
                    self.draw_one_marker(ui, &edge_route.points, end_idx);
                }
                RelationType::ManyToMany => {
                    // println!("  -> Drawing ManyToMany markers at indices {} and {}", start_idx, end_idx);
                    self.draw_many_marker(ui, &edge_route.points, start_idx);
                    self.draw_many_marker(ui, &edge_route.points, end_idx);
                }
            }
        }
    }
    
    /// Draw labels for all edges on top of everything
    fn draw_edge_labels(&mut self, ui: &mut egui::Ui) {
        // Clone the routes to avoid borrowing issues
        let all_routes = self.layout_engine.get_edge_routes().to_vec();
        
        for (idx, edge_route) in all_routes.iter().enumerate() {
            if edge_route.points.len() < 2 {
                continue;
            }
            
            // Draw label at the middle of the edge with relationship type and table names
            self.draw_edge_label(ui, edge_route, &all_routes, idx);
        }
    }
    
    /// Draw label at the middle of the edge with a pointer line and relationship type
    fn draw_edge_label(&mut self, ui: &mut egui::Ui, edge_route: &super::layout::EdgeRoute, all_edge_routes: &[super::layout::EdgeRoute], edge_idx: usize) {
        let points = &edge_route.points;
        if points.len() < 2 {
            return;
        }
        
        // Find middle point in WORLD coordinates (before zoom transformation)
        let mut total_length = 0.0;
        let mut segment_lengths = Vec::new();
        
        for i in 1..points.len() {
            let dx = points[i].x - points[i - 1].x;
            let dy = points[i].y - points[i - 1].y;
            let length = (dx * dx + dy * dy).sqrt();
            segment_lengths.push(length);
            total_length += length;
        }
        
        // Find the segment containing the middle point in world coordinates
        let target_length = total_length / 2.0;
        let mut accumulated = 0.0;
        let mut middle_point_world = None;
        let mut normal_world = egui::vec2(0.0, -1.0);
        
        for i in 0..segment_lengths.len() {
            let seg_len = segment_lengths[i];
            if accumulated + seg_len >= target_length {
                let t = (target_length - accumulated) / seg_len;
                let p1 = &points[i];
                let p2 = &points[i + 1];
                
                // Calculate middle point in world coordinates
                let mid_x = p1.x + (p2.x - p1.x) * t;
                let mid_y = p1.y + (p2.y - p1.y) * t;
                middle_point_world = Some((mid_x, mid_y));
                
                // Calculate normal in world coordinates
                let dx = p2.x - p1.x;
                let dy = p2.y - p1.y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.0 {
                    normal_world = egui::vec2(-dy / len, dx / len);
                }
                break;
            }
            accumulated += seg_len;
        }
        
        if let Some((world_x, world_y)) = middle_point_world {
            // Check if there's a custom offset for this label (from dragging)
            let (label_world_x, label_world_y) = if let Some(Some((offset_x, offset_y))) = self.label_offsets.get(edge_idx) {
                // Use custom position
                (world_x + offset_x, world_y + offset_y)
            } else {
                // Use automatic positioning
                (world_x, world_y)
            };
            
            // Transform line position to screen
            let line_pos = self.transform_point(world_x, world_y);
            
            // Get relationship type text
            let rel_text = match edge_route.relationship_type {
                super::graph::RelationType::OneToOne => "1:1",
                super::graph::RelationType::OneToMany => "1:M",
                super::graph::RelationType::ManyToOne => "M:1",
                super::graph::RelationType::ManyToMany => "M:M",
            };
            
            // Format label with table names
            let full_label = format!("{}.{}:{}.{}", 
                edge_route.from_table, edge_route.label.split(':').next().unwrap_or(""),
                edge_route.to_table, edge_route.label.split(':').nth(1).unwrap_or("")
            );
            
            // Measure text size (use constant font size for stability)
            let font_id = FontId::proportional(11.0 * self.zoom);
            let rel_font_id = FontId::proportional(10.0 * self.zoom);
            
            let rel_text_with_brackets = format!("[{}]", rel_text);
            let rel_galley = ui.painter().layout_no_wrap(
                rel_text_with_brackets.clone(),
                rel_font_id.clone(),
                Color32::from_rgb(180, 180, 180),
            );
            let label_galley = ui.painter().layout_no_wrap(
                full_label.clone(),
                font_id.clone(),
                Color32::WHITE,
            );
            
            // Calculate total size with spacing
            let spacing = 5.0 * self.zoom;
            let total_width = rel_galley.size().x + spacing + label_galley.size().x;
            let total_height = rel_galley.size().y.max(label_galley.size().y);
            let padding = egui::vec2(6.0 * self.zoom, 4.0 * self.zoom);
            
            // If custom offset exists, use it directly; otherwise find best position
            let label_screen_pos = if self.label_offsets.get(edge_idx).and_then(|o| *o).is_some() {
                // Use the custom dragged position
                self.transform_point(label_world_x, label_world_y)
            } else {
                // Try multiple angles around the line position (not just 4 cardinal directions)
                let base_offset_distance = 55.0;  // World space distance - increased for better visibility
                let min_safe_distance = 45.0; // Minimum distance to ensure pointer line is always visible
                let mut test_positions = Vec::new();
                
                // Try 8 directions around the line
                for angle_idx in 0..8 {
                    let angle = (angle_idx as f32) * std::f32::consts::PI / 4.0;
                    let offset_x = angle.cos() * base_offset_distance;
                    let offset_y = angle.sin() * base_offset_distance;
                    
                    // Calculate position in world space, then transform
                    let world_label_x_test = world_x + offset_x;
                    let world_label_y_test = world_y + offset_y;
                    let screen_pos = self.transform_point(world_label_x_test, world_label_y_test);
                    test_positions.push(screen_pos);
                }
                
                // Also try perpendicular positions (along normal) at various distances
                for multiplier in [1.0, -1.0, 1.5, -1.5, 2.0, -2.0] {
                    let offset_x = normal_world.x * base_offset_distance * multiplier;
                    let offset_y = normal_world.y * base_offset_distance * multiplier;
                    let world_label_x_test = world_x + offset_x;
                    let world_label_y_test = world_y + offset_y;
                    let screen_pos = self.transform_point(world_label_x_test, world_label_y_test);
                    test_positions.push(screen_pos);
                }
                
                let mut best_pos = test_positions[0];
                let mut best_score = f32::NEG_INFINITY;
            
            for test_pos in test_positions {
                let test_rect = Rect::from_min_size(
                    test_pos - egui::vec2(total_width / 2.0, total_height / 2.0) - padding,
                    egui::vec2(total_width, total_height) + padding * 2.0,
                );
                
                let mut score = 1000.0; // Start with high score
                
                // Calculate distance from label box to line (for pointer line visibility)
                let box_to_line_dist = (test_pos - line_pos).length();
                if box_to_line_dist < min_safe_distance * self.zoom {
                    score -= 300.0; // Heavy penalty if too close to line
                } else {
                    score += (box_to_line_dist / self.zoom).min(100.0); // Bonus for being far enough
                }
                
                // Check distance to all tables (nodes)
                for node_idx in self.erd_graph.graph().node_indices() {
                    if let Some(node_layout) = self.layout_engine.get_node_layout(node_idx) {
                        let node_pos = self.transform_point(node_layout.position.x, node_layout.position.y);
                        let node_size = egui::vec2(
                            node_layout.size.width * self.zoom,
                            node_layout.size.height * self.zoom,
                        );
                        let node_rect = Rect::from_min_size(node_pos, node_size);
                        
                        if test_rect.intersects(node_rect) {
                            score -= 500.0; // Heavy penalty for table intersection
                        } else {
                            let dx = if test_rect.max.x < node_rect.min.x {
                                node_rect.min.x - test_rect.max.x
                            } else if test_rect.min.x > node_rect.max.x {
                                test_rect.min.x - node_rect.max.x
                            } else {
                                0.0
                            };
                            let dy = if test_rect.max.y < node_rect.min.y {
                                node_rect.min.y - test_rect.max.y
                            } else if test_rect.min.y > node_rect.max.y {
                                test_rect.min.y - node_rect.max.y
                            } else {
                                0.0
                            };
                            let dist = (dx * dx + dy * dy).sqrt();
                            score += dist.min(50.0); // Bonus for being far from tables
                        }
                    }
                }
                
                // Check distance to OTHER labels
                for other_route in all_edge_routes {
                    if std::ptr::eq(other_route, edge_route) {
                        continue; // Skip self
                    }
                    
                    // Calculate where the other label would be positioned
                    // (simplified check using line middle point)
                    if let Some(other_middle) = Self::get_edge_middle_point(&other_route.points) {
                        let other_screen = self.transform_point(other_middle.0, other_middle.1);
                        
                        // Approximate other label rect (we use same offset for simplicity)
                        let approx_other_rect = Rect::from_center_size(
                            other_screen,
                            egui::vec2(total_width + padding.x * 2.0, total_height + padding.y * 2.0),
                        );
                        
                        if test_rect.intersects(approx_other_rect) {
                            score -= 400.0; // Heavy penalty for label overlap (increased)
                        } else {
                            let dist_x = (test_rect.center().x - approx_other_rect.center().x).abs();
                            let dist_y = (test_rect.center().y - approx_other_rect.center().y).abs();
                            let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();
                            if dist < 150.0 * self.zoom {
                                score -= (150.0 * self.zoom - dist) * 0.5; // Penalty for being close to other labels
                            }
                        }
                    }
                }
                
                if score > best_score {
                    best_score = score;
                    best_pos = test_pos;
                }
            }
            
                best_pos
            };
            
            // Use the determined label position
            let label_pos = label_screen_pos;
            
            // Cache the world position for click detection (only if not using custom offset)
            if self.label_offsets.get(edge_idx).and_then(|o| *o).is_none() {
                let label_world = self.screen_to_world(label_pos);
                // Ensure cache has enough capacity
                while self.cached_label_positions.len() <= edge_idx {
                    self.cached_label_positions.push(None);
                }
                self.cached_label_positions[edge_idx] = Some(label_world);
            }
            
            // Draw background box
            let text_rect = Rect::from_min_size(
                label_pos - egui::vec2(total_width / 2.0, total_height / 2.0) - padding,
                egui::vec2(total_width, total_height) + padding * 2.0,
            );
            
            ui.painter().rect_filled(
                text_rect,
                3.0 * self.zoom,
                Color32::from_rgba_premultiplied(50, 50, 50, 230),
            );
            
            ui.painter().rect_stroke(
                text_rect,
                3.0 * self.zoom,
                Stroke::new(1.0 * self.zoom, Color32::from_rgb(100, 100, 100)),
            );
            
            // Draw pointer line from box edge to line (at any angle)
            let box_center = text_rect.center();
            let to_line = line_pos - box_center;
            let box_edge_point = if to_line.length() > 0.0 {
                let dir = to_line.normalized();
                let half_width = text_rect.width() / 2.0;
                let half_height = text_rect.height() / 2.0;
                
                // Calculate intersection at any angle (not just 4 sides)
                let t_x = if dir.x.abs() > 0.001 { half_width / dir.x.abs() } else { f32::INFINITY };
                let t_y = if dir.y.abs() > 0.001 { half_height / dir.y.abs() } else { f32::INFINITY };
                let t = t_x.min(t_y);
                
                box_center + dir * t
            } else {
                box_center
            };
            
            ui.painter().line_segment(
                [box_edge_point, line_pos],
                Stroke::new(0.8 * self.zoom, Color32::from_rgb(120, 120, 120)),
            );
            
            // Draw text
            let rel_width = rel_galley.size().x;
            ui.painter().galley(
                Pos2::new(
                    text_rect.min.x + padding.x,
                    text_rect.center().y - rel_galley.size().y / 2.0,
                ),
                rel_galley,
                Color32::from_rgb(180, 180, 180),
            );
            
            ui.painter().galley(
                Pos2::new(
                    text_rect.min.x + padding.x + rel_width + spacing,
                    text_rect.center().y - label_galley.size().y / 2.0,
                ),
                label_galley,
                Color32::WHITE,
            );
        }
    }
    
    /// Helper function to get the middle point of an edge in world coordinates
    fn get_edge_middle_point(points: &[super::layout::Point]) -> Option<(f32, f32)> {
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
    
    /// Draw "one" marker (single perpendicular line)
    fn draw_one_marker(&self, ui: &mut egui::Ui, points: &[super::layout::Point], idx: usize) {
        if points.len() < 2 {
            return;
        }
        
        let (pos, direction) = if idx == 0 {
            // At start, direction is from first to second point
            let p0 = self.transform_point(points[0].x, points[0].y);
            let p1 = self.transform_point(points[1].x, points[1].y);
            let dx = p1.x - p0.x;
            let dy = p1.y - p0.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            (p0, egui::vec2(dx / len, dy / len))
        } else {
            // At end, direction is from second-to-last to last point
            let p_prev = self.transform_point(points[idx - 1].x, points[idx - 1].y);
            let p_last = self.transform_point(points[idx].x, points[idx].y);
            let dx = p_last.x - p_prev.x;
            let dy = p_last.y - p_prev.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            (p_last, egui::vec2(dx / len, dy / len))
        };
        
        // Perpendicular direction
        let perp = egui::vec2(-direction.y, direction.x);
        let size = 10.0 * self.zoom;
        
        // Draw perpendicular line with thicker stroke for visibility
        ui.painter().line_segment(
            [pos + perp * size, pos - perp * size],
            Stroke::new(2.5 * self.zoom, Color32::from_rgb(52, 73, 94)),
        );
    }
    
    /// Draw "many" marker (crow's foot - three lines)
    fn draw_many_marker(&self, ui: &mut egui::Ui, points: &[super::layout::Point], idx: usize) {
        if points.len() < 2 {
            return;
        }
        
        let (pos, direction) = if idx == 0 {
            // At start, direction is from first to second point
            let p0 = self.transform_point(points[0].x, points[0].y);
            let p1 = self.transform_point(points[1].x, points[1].y);
            let dx = p1.x - p0.x;
            let dy = p1.y - p0.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            (p0, egui::vec2(dx / len, dy / len))
        } else {
            // At end, direction is from second-to-last to last point
            let p_prev = self.transform_point(points[idx - 1].x, points[idx - 1].y);
            let p_last = self.transform_point(points[idx].x, points[idx].y);
            let dx = p_last.x - p_prev.x;
            let dy = p_last.y - p_prev.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            (p_last, egui::vec2(dx / len, dy / len))
        };
        
        // Perpendicular direction
        let perp = egui::vec2(-direction.y, direction.x);
        let size = 12.0 * self.zoom;
        let back_offset = 20.0 * self.zoom;
        
        // Base point (back from the endpoint along the edge direction)
        let base = if idx == 0 {
            pos + direction * back_offset
        } else {
            pos - direction * back_offset
        };
        
        // Draw three lines forming crow's foot with thicker strokes for visibility
        let crow_stroke = Stroke::new(2.5 * self.zoom, Color32::from_rgb(52, 73, 94));
        
        // Center line (pointing to the connection point)
        ui.painter().line_segment([base, pos], crow_stroke);
        
        // Left and right lines (spreading out from base)
        ui.painter().line_segment([base, pos + perp * size], crow_stroke);
        ui.painter().line_segment([base, pos - perp * size], crow_stroke);
    }
    
    fn transform_point(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(
            x * self.zoom + self.pan_offset.x,
            y * self.zoom + self.pan_offset.y,
        )
    }
    
    fn screen_to_world(&self, screen_pos: Pos2) -> (f32, f32) {
        (
            (screen_pos.x - self.pan_offset.x) / self.zoom,
            (screen_pos.y - self.pan_offset.y) / self.zoom,
        )
    }
    
    /// Get label position with custom offset applied (if any)
    fn get_label_position_with_offset(&self, edge_route: &super::layout::EdgeRoute, idx: usize) -> Option<(f32, f32)> {
        if let Some((mid_x, mid_y)) = Self::get_edge_middle_point(&edge_route.points) {
            // Check if there's a custom offset (from dragging)
            if let Some(Some((offset_x, offset_y))) = self.label_offsets.get(idx) {
                Some((mid_x + offset_x, mid_y + offset_y))
            } else {
                // Use cached position if available (from collision avoidance)
                self.cached_label_positions.get(idx)
                    .and_then(|p| *p)
                    .or(Some((mid_x, mid_y)))
            }
        } else {
            None
        }
    }
    
    /// Export the current view to SVG
    fn export_to_svg(&self) {
        let result = self.generate_svg();
        
        match result {
            Ok(svg_content) => {
                // Save to file
                let filename = format!("export_{}.svg", chrono::Local::now().format("%Y%m%d_%H%M%S"));
                match std::fs::write(&filename, svg_content) {
                    Ok(_) => println!("✅ Exported to {}", filename),
                    Err(e) => eprintln!("❌ Failed to write file: {}", e),
                }
            }
            Err(e) => eprintln!("❌ Failed to generate SVG: {}", e),
        }
    }
    
    /// Generate SVG content from current view
    fn generate_svg(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut svg = String::new();
        
        // Calculate bounding box for all elements
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        // Include tables
        for node_idx in self.erd_graph.graph().node_indices() {
            if let Some(layout) = self.layout_engine.get_node_layout(node_idx) {
                min_x = min_x.min(layout.position.x);
                min_y = min_y.min(layout.position.y);
                max_x = max_x.max(layout.position.x + layout.size.width);
                max_y = max_y.max(layout.position.y + layout.size.height);
            }
        }
        
        // Include all edge route points (relationship lines)
        for edge_route in self.layout_engine.get_edge_routes() {
            for point in &edge_route.points {
                min_x = min_x.min(point.x);
                min_y = min_y.min(point.y);
                max_x = max_x.max(point.x);
                max_y = max_y.max(point.y);
            }
        }
        
        // Include labels and pointer lines
        for (idx, edge_route) in self.layout_engine.get_edge_routes().iter().enumerate() {
            // Include the relationship line middle point (where pointer points to)
            if let Some((line_x, line_y)) = Self::get_edge_middle_point(&edge_route.points) {
                min_x = min_x.min(line_x);
                min_y = min_y.min(line_y);
                max_x = max_x.max(line_x);
                max_y = max_y.max(line_y);
            }
            
            if let Some((label_x, label_y)) = self.get_label_position_with_offset(edge_route, idx) {
                // Estimate label box size
                let rel_text = match edge_route.relationship_type {
                    super::graph::RelationType::OneToOne => "1:1",
                    super::graph::RelationType::OneToMany => "1:M",
                    super::graph::RelationType::ManyToOne => "M:1",
                    super::graph::RelationType::ManyToMany => "M:M",
                };
                let rel_text_with_brackets = format!("[{}]", rel_text);
                let field_label = format!("{}.{}:{}.{}", 
                    edge_route.from_table, 
                    edge_route.label.split(':').next().unwrap_or(""),
                    edge_route.to_table, 
                    edge_route.label.split(':').nth(1).unwrap_or("")
                );
                
                let rel_text_width = rel_text_with_brackets.len() as f32 * 6.0;
                let field_label_width = field_label.len() as f32 * 6.5;
                let spacing = 5.0;
                let total_text_width = rel_text_width + spacing + field_label_width;
                let padding = 6.0;
                let label_width = total_text_width + padding * 2.0;
                let label_height = 20.0 + padding * 2.0;
                
                min_x = min_x.min(label_x - label_width / 2.0);
                min_y = min_y.min(label_y - label_height / 2.0);
                max_x = max_x.max(label_x + label_width / 2.0);
                max_y = max_y.max(label_y + label_height / 2.0);
            }
        }
        
        // Include title
        min_y = min_y.min(self.title_position.1 - 50.0);
        max_y = max_y.max(self.title_position.1 + 50.0);
        
        // Add padding
        let padding = 50.0;
        min_x -= padding;
        min_y -= padding;
        max_x += padding;
        max_y += padding;
        
        let width = max_x - min_x;
        let height = max_y - min_y;
        
        // SVG header
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="{}" height="{}" viewBox="{} {} {} {}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <style>
    .table {{ fill: white; stroke: #3498db; stroke-width: 2; }}
    .table-header {{ fill: #3498db; }}
    .table-text {{ fill: white; font-family: Arial, sans-serif; font-size: 14px; }}
    .column-text {{ fill: #2c3e50; font-family: Arial, sans-serif; font-size: 11px; }}
    .type-text {{ fill: #7f8c8d; font-family: Arial, sans-serif; font-size: 9px; }}
    .relationship {{ fill: none; stroke: #34495e; stroke-width: 2; }}
    .relationship-selected {{ fill: none; stroke: #e74c3c; stroke-width: 3; }}
    .label-box {{ fill: rgba(50, 50, 50, 0.9); stroke: #646464; stroke-width: 1; }}
    .label-text {{ fill: white; font-family: Arial, sans-serif; font-size: 10px; }}
    .title-text {{ fill: #2c3e50; font-family: Arial, sans-serif; font-size: 32px; font-weight: bold; }}
  </style>
</defs>
"#,
            width, height, min_x, min_y, width, height
        ));
        
        // Draw edges first
        for edge_route in self.layout_engine.get_edge_routes() {
            if edge_route.points.len() < 2 {
                continue;
            }
            
            // Check if this edge is connected to selected table
            let is_selected = if let Some(selected) = self.selected_table {
                let selected_name = &self.erd_graph.graph()[selected].name;
                &edge_route.from_table == selected_name || &edge_route.to_table == selected_name
            } else {
                false
            };
            
            let class = if is_selected { "relationship-selected" } else { "relationship" };
            
            // Build path
            let mut path = String::from("M ");
            for (i, point) in edge_route.points.iter().enumerate() {
                if i == 0 {
                    path.push_str(&format!("{},{} ", point.x, point.y));
                } else {
                    path.push_str(&format!("L {},{} ", point.x, point.y));
                }
            }
            
            svg.push_str(&format!(r#"  <path class="{}" d="{}" />"#, class, path));
            svg.push('\n');
            
            // Draw relationship markers
            self.add_svg_markers(&mut svg, edge_route);
        }
        
        // Draw tables
        for node_idx in self.erd_graph.graph().node_indices() {
            let table = &self.erd_graph.graph()[node_idx];
            if let Some(layout) = self.layout_engine.get_node_layout(node_idx) {
                let x = layout.position.x;
                let y = layout.position.y;
                let w = layout.size.width;
                let h = layout.size.height;
                
                // Table background
                svg.push_str(&format!(
                    r#"  <rect class="table" x="{}" y="{}" width="{}" height="{}" rx="8" />"#,
                    x, y, w, h
                ));
                svg.push('\n');
                
                // Header
                let header_height = 40.0;
                svg.push_str(&format!(
                    r#"  <rect class="table-header" x="{}" y="{}" width="{}" height="{}" rx="8" />"#,
                    x, y, w, header_height
                ));
                svg.push('\n');
                
                // Table name
                svg.push_str(&format!(
                    r#"  <text class="table-text" x="{}" y="{}" text-anchor="middle">{}</text>"#,
                    x + w / 2.0, y + header_height / 2.0 + 5.0, table.name
                ));
                svg.push('\n');
                
                // Columns
                let mut y_offset = header_height + 20.0;
                for column in &table.columns {
                    let col_name = if column.attributes.contains(&"PK".to_string()) {
                        format!("🔑 {}", column.name)
                    } else {
                        column.name.clone()
                    };
                    
                    svg.push_str(&format!(
                        r#"  <text class="column-text" x="{}" y="{}">{}</text>"#,
                        x + 15.0, y + y_offset, col_name
                    ));
                    svg.push('\n');
                    
                    // Type
                    let type_text = if column.attributes.is_empty() {
                        column.data_type.clone()
                    } else {
                        format!("{} [{}]", column.data_type, column.attributes.join(","))
                    };
                    
                    svg.push_str(&format!(
                        r#"  <text class="type-text" x="{}" y="{}" text-anchor="end">{}</text>"#,
                        x + w - 15.0, y + y_offset, type_text
                    ));
                    svg.push('\n');
                    
                    y_offset += 25.0;
                }
            }
        }
        
        // Draw labels
        for (idx, edge_route) in self.layout_engine.get_edge_routes().iter().enumerate() {
            // Get the middle point of the edge line (where pointer should point to)
            let line_middle = Self::get_edge_middle_point(&edge_route.points);
            
            if let (Some((label_x, label_y)), Some((line_x, line_y))) = 
                (self.get_label_position_with_offset(edge_route, idx), line_middle) {
                
                let rel_text = match edge_route.relationship_type {
                    super::graph::RelationType::OneToOne => "1:1",
                    super::graph::RelationType::OneToMany => "1:M",
                    super::graph::RelationType::ManyToOne => "M:1",
                    super::graph::RelationType::ManyToMany => "M:M",
                };
                
                // Format: [1:M] table1.field:table2.field (split into two parts)
                let rel_text_with_brackets = format!("[{}]", rel_text);
                let field_label = format!("{}.{}:{}.{}", 
                    edge_route.from_table, 
                    edge_route.label.split(':').next().unwrap_or(""),
                    edge_route.to_table, 
                    edge_route.label.split(':').nth(1).unwrap_or("")
                );
                
                // Calculate text widths (approximate with character counting)
                // Font sizes: rel_text is 10px, field_label is 11px
                let rel_text_width = rel_text_with_brackets.len() as f32 * 6.0; // ~10px font
                let field_label_width = field_label.len() as f32 * 6.5; // ~11px font
                let spacing = 5.0;
                let total_text_width = rel_text_width + spacing + field_label_width;
                
                // Padding should match window rendering (6.0 * zoom, but we're in world coords)
                let padding = 6.0;
                let label_width = total_text_width;
                let label_height = 20.0; // Approximate text height
                
                // Calculate box edges for pointer line
                let box_left = label_x - label_width / 2.0 - padding;
                let box_right = label_x + label_width / 2.0 + padding;
                let box_top = label_y - label_height / 2.0 - padding;
                let box_bottom = label_y + label_height / 2.0 + padding;
                
                // Calculate direction from label center to line point
                let dx = line_x - label_x;
                let dy = line_y - label_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                // Find intersection point on box edge (where pointer line starts)
                let (edge_x, edge_y) = if dist > 0.0 {
                    let dir_x = dx / dist;
                    let dir_y = dy / dist;
                    
                    // Calculate t values for intersection with each side
                    let t_x = if dir_x.abs() > 0.001 {
                        if dir_x > 0.0 {
                            (box_right - label_x) / dir_x
                        } else {
                            (box_left - label_x) / dir_x
                        }
                    } else {
                        f32::INFINITY
                    };
                    
                    let t_y = if dir_y.abs() > 0.001 {
                        if dir_y > 0.0 {
                            (box_bottom - label_y) / dir_y
                        } else {
                            (box_top - label_y) / dir_y
                        }
                    } else {
                        f32::INFINITY
                    };
                    
                    let t = t_x.min(t_y);
                    (label_x + dir_x * t, label_y + dir_y * t)
                } else {
                    (label_x, label_y)
                };
                
                // Draw pointer line from box edge to line middle
                svg.push_str(&format!(
                    r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#787878" stroke-width="0.8" />"##,
                    edge_x, edge_y, line_x, line_y
                ));
                svg.push('\n');
                
                // Label box
                svg.push_str(&format!(
                    r#"  <rect class="label-box" x="{}" y="{}" width="{}" height="{}" rx="3" />"#,
                    box_left, box_top, label_width + padding * 2.0, label_height + padding * 2.0
                ));
                svg.push('\n');
                
                // Label text - relationship type in gray
                svg.push_str(&format!(
                    r##"  <text class="rel-type-text" x="{}" y="{}" fill="#B4B4B4" font-size="10">{}</text>"##,
                    label_x - total_text_width / 2.0, label_y + 4.0, rel_text_with_brackets
                ));
                svg.push('\n');
                
                // Label text - field names in white
                svg.push_str(&format!(
                    r##"  <text class="label-text" x="{}" y="{}" font-size="11">{}</text>"##,
                    label_x - total_text_width / 2.0 + rel_text_width + spacing, label_y + 4.0, field_label
                ));
                svg.push('\n');
            }
        }
        
        // Draw title
        svg.push_str(&format!(
            r#"  <text class="title-text" x="{}" y="{}" text-anchor="middle">{}</text>"#,
            self.title_position.0, self.title_position.1, self.title
        ));
        svg.push('\n');
        
        svg.push_str("</svg>");
        
        Ok(svg)
    }
    
    /// Add SVG markers for relationships
    fn add_svg_markers(&self, svg: &mut String, edge_route: &super::layout::EdgeRoute) {
        if edge_route.points.len() < 2 {
            return;
        }
        
        let start_idx = 0;
        let end_idx = edge_route.points.len() - 1;
        
        match edge_route.relationship_type {
            super::graph::RelationType::OneToOne => {
                self.add_svg_one_marker(svg, &edge_route.points, start_idx);
                self.add_svg_one_marker(svg, &edge_route.points, end_idx);
            }
            super::graph::RelationType::OneToMany => {
                self.add_svg_one_marker(svg, &edge_route.points, start_idx);
                self.add_svg_many_marker(svg, &edge_route.points, end_idx);
            }
            super::graph::RelationType::ManyToOne => {
                self.add_svg_many_marker(svg, &edge_route.points, start_idx);
                self.add_svg_one_marker(svg, &edge_route.points, end_idx);
            }
            super::graph::RelationType::ManyToMany => {
                self.add_svg_many_marker(svg, &edge_route.points, start_idx);
                self.add_svg_many_marker(svg, &edge_route.points, end_idx);
            }
        }
    }
    
    fn add_svg_one_marker(&self, svg: &mut String, points: &[super::layout::Point], idx: usize) {
        if points.len() < 2 {
            return;
        }
        
        let (pos, direction) = if idx == 0 {
            let p0 = &points[0];
            let p1 = &points[1];
            let dx = p1.x - p0.x;
            let dy = p1.y - p0.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            ((p0.x, p0.y), (dx / len, dy / len))
        } else {
            let p_prev = &points[idx - 1];
            let p_last = &points[idx];
            let dx = p_last.x - p_prev.x;
            let dy = p_last.y - p_prev.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            ((p_last.x, p_last.y), (dx / len, dy / len))
        };
        
        let perp = (-direction.1, direction.0);
        let size = 10.0;
        
        svg.push_str(&format!(
            r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#34495e" stroke-width="2.5" />"##,
            pos.0 + perp.0 * size, pos.1 + perp.1 * size,
            pos.0 - perp.0 * size, pos.1 - perp.1 * size
        ));
        svg.push('\n');
    }
    
    fn add_svg_many_marker(&self, svg: &mut String, points: &[super::layout::Point], idx: usize) {
        if points.len() < 2 {
            return;
        }
        
        let (pos, direction) = if idx == 0 {
            let p0 = &points[0];
            let p1 = &points[1];
            let dx = p1.x - p0.x;
            let dy = p1.y - p0.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            ((p0.x, p0.y), (dx / len, dy / len))
        } else {
            let p_prev = &points[idx - 1];
            let p_last = &points[idx];
            let dx = p_last.x - p_prev.x;
            let dy = p_last.y - p_prev.y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            ((p_last.x, p_last.y), (dx / len, dy / len))
        };
        
        let perp = (-direction.1, direction.0);
        let size = 12.0;
        let back_offset = 20.0;
        
        let base = if idx == 0 {
            (pos.0 + direction.0 * back_offset, pos.1 + direction.1 * back_offset)
        } else {
            (pos.0 - direction.0 * back_offset, pos.1 - direction.1 * back_offset)
        };
        
        // Center line
        svg.push_str(&format!(
            r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#34495e" stroke-width="2.5" />"##,
            base.0, base.1, pos.0, pos.1
        ));
        svg.push('\n');
        
        // Left line
        svg.push_str(&format!(
            r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#34495e" stroke-width="2.5" />"##,
            base.0, base.1, pos.0 + perp.0 * size, pos.1 + perp.1 * size
        ));
        svg.push('\n');
        
        // Right line
        svg.push_str(&format!(
            r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#34495e" stroke-width="2.5" />"##,
            base.0, base.1, pos.0 - perp.0 * size, pos.1 - perp.1 * size
        ));
        svg.push('\n');
    }
}

impl eframe::App for ErdCanvas {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Add menu bar at the top
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Export", |ui| {
                    if ui.button("SVG").clicked() {
                        self.export_to_svg();
                        ui.close_menu();
                    }
                });
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Set initial pan and zoom to center the graph
            if !self.initial_pan_set {
                let viewport_size = ui.available_size();
                self.center_graph(viewport_size);
                self.initial_pan_set = true;
            }
            
            // Get pointer information
            let pointer_pos = ui.input(|i| i.pointer.hover_pos());
            let pointer_down = ui.input(|i| i.pointer.primary_down());
            let pointer_pressed = ui.input(|i| i.pointer.primary_pressed());
            let pointer_released = ui.input(|i| i.pointer.primary_released());
            
            // Handle dragging
            if pointer_pressed {
                if let Some(pos) = pointer_pos {
                    let mut found_target = false;
                    
                    // First check if clicking on title (highest priority)
                    let title_screen = self.transform_point(self.title_position.0, self.title_position.1);
                    let title_width = (self.title.len() as f32 * 20.0 + 40.0) * self.zoom; // H1 is bigger
                    let title_height = 50.0 * self.zoom;
                    let title_rect = Rect::from_center_size(
                        title_screen, 
                        egui::vec2(title_width, title_height)
                    );
                    
                    if title_rect.contains(pos) {
                        self.drag_target = DragTarget::Title;
                        self.drag_offset = (pos - title_screen).to_pos2();
                        self.selected_table = None;
                        found_target = true;
                    }
                    
                    // Second pass: Check ALL labels with their actual rendered size
                    if !found_target {
                    for (idx, edge_route) in self.layout_engine.get_edge_routes().iter().enumerate() {
                        // Calculate where the label actually is (with custom offset if any)
                        let label_world_pos = if let Some(Some((offset_x, offset_y))) = self.label_offsets.get(idx) {
                            // Label has custom position
                            if let Some((mid_x, mid_y)) = Self::get_edge_middle_point(&edge_route.points) {
                                Some((mid_x + offset_x, mid_y + offset_y))
                            } else {
                                None
                            }
                        } else {
                            // Label uses automatic positioning
                            // Use cached position if available, otherwise use middle point
                            self.cached_label_positions.get(idx).and_then(|p| *p)
                                .or_else(|| Self::get_edge_middle_point(&edge_route.points))
                        };
                        
                        if let Some((label_world_x, label_world_y)) = label_world_pos {
                            let label_screen = self.transform_point(label_world_x, label_world_y);
                            
                            // Use more accurate label size estimation
                            let label_text = format!("{}.{}:{}.{}", 
                                edge_route.from_table, 
                                edge_route.label.split(':').next().unwrap_or(""),
                                edge_route.to_table, 
                                edge_route.label.split(':').nth(1).unwrap_or("")
                            );
                            
                            // Estimate based on text length
                            let label_width = (label_text.len() as f32 * 7.0 + 40.0) * self.zoom;
                            let label_height = 28.0 * self.zoom;
                            let padding = egui::vec2(12.0 * self.zoom, 8.0 * self.zoom);
                            
                            let label_rect = Rect::from_center_size(
                                label_screen, 
                                egui::vec2(label_width + padding.x * 2.0, label_height + padding.y * 2.0)
                            );
                            
                            if label_rect.contains(pos) {
                                self.drag_target = DragTarget::Label(idx);
                                self.drag_offset = (pos - label_screen).to_pos2();
                                self.selected_table = None; // Deselect table when clicking label
                                found_target = true;
                                break;
                            }
                        }
                    }
                    }
                    
                    // Third pass: Check tables if no label was clicked
                    if !found_target {
                        for node_idx in self.erd_graph.graph().node_indices() {
                            if let Some(layout) = self.layout_engine.get_node_layout(node_idx) {
                                let screen_pos = self.transform_point(layout.position.x, layout.position.y);
                                let screen_size = egui::vec2(layout.size.width * self.zoom, layout.size.height * self.zoom);
                                let table_rect = Rect::from_min_size(screen_pos, screen_size);
                                
                                if table_rect.contains(pos) {
                                    self.drag_target = DragTarget::Table(node_idx);
                                    self.drag_offset = (pos - screen_pos).to_pos2();
                                    self.selected_table = Some(node_idx); // Select the clicked table
                                    found_target = true;
                                    break;
                                }
                            }
                        }
                    }
                    
                    // If clicking on empty space, deselect
                    if !found_target {
                        self.selected_table = None;
                    }
                }
            }
            
            // Handle drag movement
            if pointer_down && self.drag_target != DragTarget::None {
                if let Some(pos) = pointer_pos {
                    let target_pos = pos - self.drag_offset.to_vec2();
                    let (world_x, world_y) = self.screen_to_world(target_pos);
                    
                    match &self.drag_target {
                        DragTarget::Title => {
                            // Update title position
                            self.title_position = (world_x, world_y);
                        }
                        DragTarget::Table(node_idx) => {
                            // Update table position
                            if let Some(layout) = self.layout_engine.get_node_layout_mut(*node_idx) {
                                layout.position.x = world_x;
                                layout.position.y = world_y;
                            }
                            // Recalculate edge routes
                            self.layout_engine.recompute_edge_routes(&self.erd_graph);
                        }
                        DragTarget::Label(idx) => {
                            // Get the automatic label position
                            if let Some(edge_route) = self.layout_engine.get_edge_routes().get(*idx) {
                                if let Some((auto_x, auto_y)) = Self::get_edge_middle_point(&edge_route.points) {
                                    // Store offset from automatic position
                                    let offset_x = world_x - auto_x;
                                    let offset_y = world_y - auto_y;
                                    
                                    // Ensure label_offsets has enough capacity
                                    while self.label_offsets.len() <= *idx {
                                        self.label_offsets.push(None);
                                    }
                                    self.label_offsets[*idx] = Some((offset_x, offset_y));
                                }
                            }
                        }
                        DragTarget::None => {}
                    }
                }
            }
            
            // Release drag
            if pointer_released {
                self.drag_target = DragTarget::None;
            }
            
            // Handle keyboard zoom (+/- keys)
            let zoom_delta = ui.input(|i| {
                let mut delta = 0.0;
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    delta += 0.1;
                }
                if i.key_pressed(egui::Key::Minus) {
                    delta -= 0.1;
                }
                delta
            });
            if zoom_delta != 0.0 {
                self.zoom = (self.zoom + zoom_delta).clamp(0.1, 5.0);
            }
            
            // Handle mouse wheel zoom
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = 1.0 + scroll / 500.0;
                self.zoom = (self.zoom * zoom_factor).clamp(0.1, 5.0);
            }
            
            // Handle arrow key navigation (only when not dragging)
            if self.drag_target == DragTarget::None {
                let arrow_pan = ui.input(|i| {
                    let mut pan = egui::Vec2::ZERO;
                    let step = 20.0;
                    if i.key_pressed(egui::Key::ArrowLeft) {
                        pan.x += step;
                    }
                    if i.key_pressed(egui::Key::ArrowRight) {
                        pan.x -= step;
                    }
                    if i.key_pressed(egui::Key::ArrowUp) {
                        pan.y += step;
                    }
                    if i.key_pressed(egui::Key::ArrowDown) {
                        pan.y -= step;
                    }
                    pan
                });
                self.pan_offset += arrow_pan;
            }
            
            // Draw edges first (so they appear behind tables)
            self.draw_edges(ui);
            
            // Draw tables
            for node_idx in self.erd_graph.graph().node_indices() {
                self.draw_table(ui, node_idx);
            }
            
            // Draw labels on top of everything
            self.draw_edge_labels(ui);
            
            // Draw title on top
            self.draw_title(ui);
            
            // Show controls
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(format!(
                    "Zoom: {:.1}x | Drag tables/labels to move | Scroll/+- to zoom | Arrow keys to pan", 
                    self.zoom
                ));
            });
        });
    }
}
