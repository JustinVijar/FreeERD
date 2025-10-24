pub mod types;
pub mod drawing;
pub mod svg;
pub mod interaction;
pub mod utils;

use egui::{Color32, Stroke, Pos2, Rect, FontId};
use super::graph::ErdGraph;
use super::layout::LayoutEngine;
use petgraph::graph::NodeIndex;
use types::DragTarget;

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
    
    /// Get label position with custom offset applied (if any)
    fn get_label_position_with_offset(&self, edge_route: &super::layout::EdgeRoute, idx: usize) -> Option<(f32, f32)> {
        interaction::get_label_position_world(
            edge_route,
            &self.cached_label_positions,
            &self.label_offsets,
            idx,
        )
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
        let svg_ctx = svg::SvgContext {
            erd_graph: &self.erd_graph,
            layout_engine: &self.layout_engine,
            selected_table: self.selected_table,
            label_offsets: &self.label_offsets,
        };
        
        svg::generate_svg(&svg_ctx, &self.title, self.title_position)
    }
    
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
            let line_pos = utils::transform_point(world_x, world_y, self.zoom, self.pan_offset);
            
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
                utils::transform_point(label_world_x, label_world_y, self.zoom, self.pan_offset)
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
                    let screen_pos = utils::transform_point(world_label_x_test, world_label_y_test, self.zoom, self.pan_offset);
                    test_positions.push(screen_pos);
                }
                
                // Also try perpendicular positions (along normal) at various distances
                for multiplier in [1.0, -1.0, 1.5, -1.5, 2.0, -2.0] {
                    let offset_x = normal_world.x * base_offset_distance * multiplier;
                    let offset_y = normal_world.y * base_offset_distance * multiplier;
                    let world_label_x_test = world_x + offset_x;
                    let world_label_y_test = world_y + offset_y;
                    let screen_pos = utils::transform_point(world_label_x_test, world_label_y_test, self.zoom, self.pan_offset);
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
                        let node_pos = utils::transform_point(node_layout.position.x, node_layout.position.y, self.zoom, self.pan_offset);
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
                    if let Some(other_middle) = interaction::get_edge_middle_point_simple(&other_route.points) {
                        let other_screen = utils::transform_point(other_middle.0, other_middle.1, self.zoom, self.pan_offset);
                        
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
                let label_world = utils::screen_to_world(label_pos, self.zoom, self.pan_offset);
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
                if let Some(bounds) = utils::calculate_bounds(&self.erd_graph, &self.layout_engine) {
                    utils::center_graph(&mut self.zoom, &mut self.pan_offset, viewport_size, bounds);
                }
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
                    let title_screen = utils::transform_point(self.title_position.0, self.title_position.1, self.zoom, self.pan_offset);
                    
                    if interaction::check_title_click(pos, title_screen, &self.title, self.zoom) {
                        self.drag_target = DragTarget::Title;
                        self.drag_offset = (pos - title_screen).to_pos2();
                        self.selected_table = None;
                        found_target = true;
                    }
                    
                    // Second pass: Check ALL labels with their actual rendered size
                    if !found_target {
                    for (idx, edge_route) in self.layout_engine.get_edge_routes().iter().enumerate() {
                        let label_world_pos = self.get_label_position_with_offset(edge_route, idx);
                        
                        if let Some((label_world_x, label_world_y)) = label_world_pos {
                            let label_screen = utils::transform_point(label_world_x, label_world_y, self.zoom, self.pan_offset);
                            
                            // Use more accurate label size estimation
                            let label_text = format!("{}.{}:{}.{}", 
                                edge_route.from_table, 
                                edge_route.label.split(':').next().unwrap_or(""),
                                edge_route.to_table, 
                                edge_route.label.split(':').nth(1).unwrap_or("")
                            );
                            
                            if interaction::check_label_click(pos, label_screen, &label_text, self.zoom) {
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
                                if interaction::check_table_click(pos, layout, self.zoom, self.pan_offset) {
                                    self.drag_target = DragTarget::Table(node_idx);
                                    let screen_pos = utils::transform_point(layout.position.x, layout.position.y, self.zoom, self.pan_offset);
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
                    let (world_x, world_y) = utils::screen_to_world(target_pos, self.zoom, self.pan_offset);
                    
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
                                if let Some((auto_x, auto_y)) = interaction::get_edge_middle_point_simple(&edge_route.points) {
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
            {
                let ctx = drawing::DrawingContext {
                    erd_graph: &self.erd_graph,
                    layout_engine: &self.layout_engine,
                    zoom: self.zoom,
                    pan_offset: self.pan_offset,
                    selected_table: self.selected_table,
                    label_offsets: &self.label_offsets,
                    cached_label_positions: &self.cached_label_positions,
                };
                drawing::draw_edges(&ctx, ui);
            }
            
            // Draw tables
            {
                let ctx = drawing::DrawingContext {
                    erd_graph: &self.erd_graph,
                    layout_engine: &self.layout_engine,
                    zoom: self.zoom,
                    pan_offset: self.pan_offset,
                    selected_table: self.selected_table,
                    label_offsets: &self.label_offsets,
                    cached_label_positions: &self.cached_label_positions,
                };
                for node_idx in self.erd_graph.graph().node_indices() {
                    drawing::draw_table(&ctx, ui, node_idx);
                }
            }
            
            // Draw labels on top of everything
            self.draw_edge_labels(ui);
            
            // Draw title on top
            {
                let ctx = drawing::DrawingContext {
                    erd_graph: &self.erd_graph,
                    layout_engine: &self.layout_engine,
                    zoom: self.zoom,
                    pan_offset: self.pan_offset,
                    selected_table: self.selected_table,
                    label_offsets: &self.label_offsets,
                    cached_label_positions: &self.cached_label_positions,
                };
                drawing::draw_title(&ctx, ui, &self.title, self.title_position, &self.drag_target);
            }
            
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
