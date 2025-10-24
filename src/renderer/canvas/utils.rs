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
