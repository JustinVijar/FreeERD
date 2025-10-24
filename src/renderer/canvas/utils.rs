use egui::Pos2;

pub fn transform_point(x: f32, y: f32, zoom: f32, pan_offset: Pos2) -> Pos2 {
    Pos2::new(
        x * zoom + pan_offset.x,
        y * zoom + pan_offset.y,
    )
}

pub fn screen_to_world(screen_pos: Pos2, zoom: f32, pan_offset: Pos2) -> (f32, f32) {
    (
        (screen_pos.x - pan_offset.x) / zoom,
        (screen_pos.y - pan_offset.y) / zoom,
    )
}

/// Check if a box at the proposed position collides with any existing tables
pub fn check_collision_with_tables(
    erd_graph: &crate::renderer::graph::ErdGraph,
    layout_engine: &crate::renderer::layout::LayoutEngine,
    proposed_x: f32,
    proposed_y: f32,
    table_width: f32,
    table_height: f32,
    exclude_node: petgraph::graph::NodeIndex,
) -> bool {
    let proposed_rect = egui::Rect::from_min_size(
        Pos2::new(proposed_x, proposed_y),
        egui::vec2(table_width, table_height),
    );
    
    for node_idx in erd_graph.graph().node_indices() {
        if node_idx == exclude_node {
            continue; // Don't check collision with self
        }
        
        if let Some(layout) = layout_engine.get_node_layout(node_idx) {
            let other_rect = egui::Rect::from_min_size(
                Pos2::new(layout.position.x, layout.position.y),
                egui::vec2(layout.size.width, layout.size.height),
            );
            
            if proposed_rect.intersects(other_rect) {
                return true; // Collision detected
            }
        }
    }
    
    false
}

/// Resolve collision by sliding the box to the nearest non-colliding position
pub fn resolve_collision(
    erd_graph: &crate::renderer::graph::ErdGraph,
    layout_engine: &crate::renderer::layout::LayoutEngine,
    proposed_x: f32,
    proposed_y: f32,
    table_width: f32,
    table_height: f32,
    exclude_node: petgraph::graph::NodeIndex,
) -> (f32, f32) {
    // If no collision, return the proposed position
    if !check_collision_with_tables(
        erd_graph,
        layout_engine,
        proposed_x,
        proposed_y,
        table_width,
        table_height,
        exclude_node,
    ) {
        return (proposed_x, proposed_y);
    }
    
    // Try to push the box away from colliding objects
    // Test multiple directions: up, down, left, right
    
    let max_offset = 150.0; // Maximum offset before giving up
    let step = 5.0; // Increment step
    
    let original_x = proposed_x;
    let original_y = proposed_y;
    
    // Try pushing vertically first
    for offset_steps in 1..=(max_offset / step) as i32 {
        let offset = offset_steps as f32 * step;
        
        // Try moving up
        if !check_collision_with_tables(
            erd_graph,
            layout_engine,
            proposed_x,
            proposed_y - offset,
            table_width,
            table_height,
            exclude_node,
        ) {
            return (proposed_x, proposed_y - offset);
        }
        
        // Try moving down
        if !check_collision_with_tables(
            erd_graph,
            layout_engine,
            proposed_x,
            proposed_y + offset,
            table_width,
            table_height,
            exclude_node,
        ) {
            return (proposed_x, proposed_y + offset);
        }
    }
    
    // Try pushing horizontally
    for offset_steps in 1..=(max_offset / step) as i32 {
        let offset = offset_steps as f32 * step;
        
        // Try moving left
        if !check_collision_with_tables(
            erd_graph,
            layout_engine,
            proposed_x - offset,
            proposed_y,
            table_width,
            table_height,
            exclude_node,
        ) {
            return (proposed_x - offset, proposed_y);
        }
        
        // Try moving right
        if !check_collision_with_tables(
            erd_graph,
            layout_engine,
            proposed_x + offset,
            proposed_y,
            table_width,
            table_height,
            exclude_node,
        ) {
            return (proposed_x + offset, proposed_y);
        }
    }
    
    // If all else fails, keep the original position
    (original_x, original_y)
}

pub fn calculate_bounds(
    erd_graph: &crate::renderer::graph::ErdGraph,
    layout_engine: &crate::renderer::layout::LayoutEngine,
) -> Option<egui::Rect> {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    
    for node_idx in erd_graph.graph().node_indices() {
        if let Some(layout) = layout_engine.get_node_layout(node_idx) {
            min_x = min_x.min(layout.position.x);
            min_y = min_y.min(layout.position.y);
            max_x = max_x.max(layout.position.x + layout.size.width);
            max_y = max_y.max(layout.position.y + layout.size.height);
        }
    }
    
    if min_x < f32::MAX {
        Some(egui::Rect::from_min_max(
            Pos2::new(min_x, min_y),
            Pos2::new(max_x, max_y),
        ))
    } else {
        None
    }
}

pub fn center_graph(
    zoom: &mut f32,
    pan_offset: &mut Pos2,
    viewport_size: egui::Vec2,
    bounds: egui::Rect,
) {
    let graph_center = bounds.center();
    let viewport_center = Pos2::new(viewport_size.x / 2.0, viewport_size.y / 2.0);
    
    // Calculate offset to center the graph
    *pan_offset = viewport_center - graph_center.to_vec2();
    
    // Add some padding
    let padding = 50.0;
    
    // Calculate zoom to fit graph in viewport with padding
    let graph_width = bounds.width();
    let graph_height = bounds.height();
    
    if graph_width > 0.0 && graph_height > 0.0 {
        let zoom_x = (viewport_size.x - padding * 2.0) / graph_width;
        let zoom_y = (viewport_size.y - padding * 2.0) / graph_height;
        *zoom = zoom_x.min(zoom_y).min(1.0); // Don't zoom in beyond 1.0
        
        // Recalculate pan offset with zoom
        let scaled_graph_center = Pos2::new(
            graph_center.x * *zoom,
            graph_center.y * *zoom,
        );
        *pan_offset = viewport_center - scaled_graph_center.to_vec2();
    }
}
