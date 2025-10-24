use crate::renderer::graph::ErdGraph;
use crate::renderer::layout::LayoutEngine;
use petgraph::graph::NodeIndex;

pub struct SvgContext<'a> {
    pub erd_graph: &'a ErdGraph,
    pub layout_engine: &'a LayoutEngine,
    pub selected_table: Option<NodeIndex>,
    pub label_offsets: &'a [Option<(f32, f32)>],
}

pub fn generate_svg(ctx: &SvgContext, title: &str, title_position: (f32, f32)) -> Result<String, Box<dyn std::error::Error>> {
    let mut svg = String::new();
    
    // Calculate bounding box for all elements
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    
    // Include tables
    for node_idx in ctx.erd_graph.graph().node_indices() {
        if let Some(layout) = ctx.layout_engine.get_node_layout(node_idx) {
            min_x = min_x.min(layout.position.x);
            min_y = min_y.min(layout.position.y);
            max_x = max_x.max(layout.position.x + layout.size.width);
            max_y = max_y.max(layout.position.y + layout.size.height);
        }
    }
    
    // Include all edge route points (relationship lines)
    for edge_route in ctx.layout_engine.get_edge_routes() {
        for point in &edge_route.points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
    }
    
    // Include labels and pointer lines
    for (idx, edge_route) in ctx.layout_engine.get_edge_routes().iter().enumerate() {
        // Include the relationship line middle point (where pointer points to)
        if let Some((line_x, line_y)) = get_edge_middle_point(&edge_route.points) {
            min_x = min_x.min(line_x);
            min_y = min_y.min(line_y);
            max_x = max_x.max(line_x);
            max_y = max_y.max(line_y);
        }
        
        if let Some((label_x, label_y)) = get_label_position_with_offset(ctx, edge_route, idx) {
            // Estimate label box size
            let rel_text = match edge_route.relationship_type {
                crate::renderer::graph::RelationType::OneToOne => "1:1",
                crate::renderer::graph::RelationType::OneToMany => "1:M",
                crate::renderer::graph::RelationType::ManyToOne => "M:1",
                crate::renderer::graph::RelationType::ManyToMany => "M:M",
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
    min_y = min_y.min(title_position.1 - 50.0);
    max_y = max_y.max(title_position.1 + 50.0);
    
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
    for edge_route in ctx.layout_engine.get_edge_routes() {
        if edge_route.points.len() < 2 {
            continue;
        }
        
        // Check if this edge is connected to selected table
        let is_selected = if let Some(selected) = ctx.selected_table {
            let selected_name = &ctx.erd_graph.graph()[selected].name;
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
        add_svg_markers(&mut svg, edge_route);
    }
    
    // Draw tables
    for node_idx in ctx.erd_graph.graph().node_indices() {
        let table = &ctx.erd_graph.graph()[node_idx];
        if let Some(layout) = ctx.layout_engine.get_node_layout(node_idx) {
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
    for (idx, edge_route) in ctx.layout_engine.get_edge_routes().iter().enumerate() {
        // Get the middle point of the edge line (where pointer should point to)
        let line_middle = get_edge_middle_point(&edge_route.points);
        
        if let (Some((label_x, label_y)), Some((line_x, line_y))) = 
            (get_label_position_with_offset(ctx, edge_route, idx), line_middle) {
            
            let rel_text = match edge_route.relationship_type {
                crate::renderer::graph::RelationType::OneToOne => "1:1",
                crate::renderer::graph::RelationType::OneToMany => "1:M",
                crate::renderer::graph::RelationType::ManyToOne => "M:1",
                crate::renderer::graph::RelationType::ManyToMany => "M:M",
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
        title_position.0, title_position.1, title
    ));
    svg.push('\n');
    
    svg.push_str("</svg>");
    
    Ok(svg)
}

fn get_edge_middle_point(points: &[crate::renderer::layout::Point]) -> Option<(f32, f32)> {
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

fn get_label_position_with_offset(ctx: &SvgContext, edge_route: &crate::renderer::layout::EdgeRoute, idx: usize) -> Option<(f32, f32)> {
    if let Some((mid_x, mid_y)) = get_edge_middle_point(&edge_route.points) {
        // Check if there's a custom offset (from dragging)
        if let Some(Some((offset_x, offset_y))) = ctx.label_offsets.get(idx) {
            Some((mid_x + offset_x, mid_y + offset_y))
        } else {
            Some((mid_x, mid_y))
        }
    } else {
        None
    }
}

fn add_svg_markers(svg: &mut String, edge_route: &crate::renderer::layout::EdgeRoute) {
    if edge_route.points.len() < 2 {
        return;
    }
    
    let start_idx = 0;
    let end_idx = edge_route.points.len() - 1;
    
    use crate::renderer::graph::RelationType;
    match edge_route.relationship_type {
        RelationType::OneToOne => {
            add_svg_one_marker(svg, &edge_route.points, start_idx);
            add_svg_one_marker(svg, &edge_route.points, end_idx);
        }
        RelationType::OneToMany => {
            add_svg_one_marker(svg, &edge_route.points, start_idx);
            add_svg_many_marker(svg, &edge_route.points, end_idx);
        }
        RelationType::ManyToOne => {
            add_svg_many_marker(svg, &edge_route.points, start_idx);
            add_svg_one_marker(svg, &edge_route.points, end_idx);
        }
        RelationType::ManyToMany => {
            add_svg_many_marker(svg, &edge_route.points, start_idx);
            add_svg_many_marker(svg, &edge_route.points, end_idx);
        }
    }
}

fn add_svg_one_marker(svg: &mut String, points: &[crate::renderer::layout::Point], idx: usize) {
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

fn add_svg_many_marker(svg: &mut String, points: &[crate::renderer::layout::Point], idx: usize) {
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
