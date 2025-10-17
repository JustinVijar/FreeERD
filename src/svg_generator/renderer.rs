use crate::ast::Column;
use super::layout::{Point, TableLayout, Rectangle, RelationshipBoxLayout, ConnectionSide};

pub struct SvgRenderer {
    content: String,
    pub table_layouts: Vec<Rectangle>,
    pub relationship_boxes: Vec<RelationshipBoxLayout>,
    title: Option<String>,
    canvas_width: f64,
}

impl SvgRenderer {
    pub fn new() -> Self {
        SvgRenderer {
            content: String::new(),
            table_layouts: Vec::new(),
            relationship_boxes: Vec::new(),
            title: None,
            canvas_width: 0.0,
        }
    }
    
    pub fn set_table_layouts(&mut self, layouts: Vec<Rectangle>) {
        self.table_layouts = layouts;
    }
    
    pub fn add_relationship_box(&mut self, box_layout: RelationshipBoxLayout) {
        self.relationship_boxes.push(box_layout);
    }
    
    pub fn render_relationship_boxes(&mut self) {
        for box_layout in &self.relationship_boxes.clone() {
            self.render_relationship_box_from_layout(box_layout);
        }
    }
    
    fn render_relationship_box_from_layout(&mut self, box_layout: &RelationshipBoxLayout) {
        // Render background box with high precision
        self.content.push_str(&format!(
            "  <rect x=\"{:.6}\" y=\"{:.6}\" width=\"{:.6}\" height=\"{:.6}\" \
             fill=\"white\" fill-opacity=\"0.95\" stroke=\"#6c757d\" stroke-width=\"1\" rx=\"4\" />\n",
            box_layout.bounds.x, box_layout.bounds.y, box_layout.bounds.width, box_layout.bounds.height
        ));
        
        // Render the relationship text
        let text_x = box_layout.bounds.x + box_layout.bounds.width / 2.0;
        let text_y = box_layout.bounds.y + box_layout.bounds.height / 2.0 + 4.0;
        self.content.push_str(&format!(
            "  <text x=\"{:.6}\" y=\"{:.6}\" class=\"relationship-self-ref\">{}</text>\n",
            text_x, text_y, Self::escape_xml(&box_layout.text)
        ));
        
        // Render connection dots with exact positions
        let dot_radius = 3.0;
        let (source_dot, target_dot) = box_layout.get_dot_positions();
        
        // Source connection dot
        self.content.push_str(&format!(
            "  <circle cx=\"{:.6}\" cy=\"{:.6}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
            source_dot.x, source_dot.y, dot_radius
        ));
        
        // Target connection dot (only if different position)
        if (source_dot.x - target_dot.x).abs() > 0.1 || (source_dot.y - target_dot.y).abs() > 0.1 {
            self.content.push_str(&format!(
                "  <circle cx=\"{:.6}\" cy=\"{:.6}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
                target_dot.x, target_dot.y, dot_radius
            ));
        }
        
        // Extract relationship type from the text
        let relationship_type = if box_layout.text.contains(" <> ") {
            super::super::ast::RelationshipType::ManyToMany
        } else if box_layout.text.contains(" > ") {
            super::super::ast::RelationshipType::OneToMany
        } else if box_layout.text.contains(" < ") {
            super::super::ast::RelationshipType::ManyToOne
        } else {
            super::super::ast::RelationshipType::OneToOne
        };
        
        // Add cardinality labels at both dots
        self.render_marker_at_dot(source_dot.x, source_dot.y, box_layout.source_side, relationship_type, true);
        self.render_marker_at_dot(target_dot.x, target_dot.y, box_layout.target_side, relationship_type, false);
    }

    pub fn start_svg(&mut self, width: f64, height: f64, title: Option<&str>) {
        self.canvas_width = width;
        self.title = title.map(|t| t.to_string());
        
        self.content.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
"#,
            width, height
        ));

        if let Some(title) = title {
            self.content.push_str(&format!(
                r#"  <title>{}</title>
"#,
                Self::escape_xml(title)
            ));
        }

        // Add defs section with crow's foot notation markers
        self.content.push_str("  <defs>\n");
        self.content.push_str("    <!-- Crow's Foot Notation Markers -->\n");
        
        // One side (single line perpendicular)
        self.content.push_str("    <marker id=\"one-end\" markerWidth=\"12\" markerHeight=\"12\"\n");
        self.content.push_str("            refX=\"11\" refY=\"6\" orient=\"auto\">\n");
        self.content.push_str("      <line x1=\"11\" y1=\"2\" x2=\"11\" y2=\"10\" stroke=\"#6c757d\" stroke-width=\"2\" />\n");
        self.content.push_str("    </marker>\n");
        
        self.content.push_str("    <marker id=\"one-start\" markerWidth=\"12\" markerHeight=\"12\"\n");
        self.content.push_str("            refX=\"1\" refY=\"6\" orient=\"auto\">\n");
        self.content.push_str("      <line x1=\"1\" y1=\"2\" x2=\"1\" y2=\"10\" stroke=\"#6c757d\" stroke-width=\"2\" />\n");
        self.content.push_str("    </marker>\n");
        
        // Many side (crow's foot - three lines forming a fork)
        self.content.push_str("    <marker id=\"many-end\" markerWidth=\"14\" markerHeight=\"14\"\n");
        self.content.push_str("            refX=\"13\" refY=\"7\" orient=\"auto\">\n");
        self.content.push_str("      <path d=\"M 13,7 L 3,2 M 13,7 L 3,12 M 13,7 L 3,7\" stroke=\"#6c757d\" stroke-width=\"2\" fill=\"none\" />\n");
        self.content.push_str("    </marker>\n");
        
        self.content.push_str("    <marker id=\"many-start\" markerWidth=\"14\" markerHeight=\"14\"\n");
        self.content.push_str("            refX=\"1\" refY=\"7\" orient=\"auto\">\n");
        self.content.push_str("      <path d=\"M 1,7 L 11,2 M 1,7 L 11,12 M 1,7 L 11,7\" stroke=\"#6c757d\" stroke-width=\"2\" fill=\"none\" />\n");
        self.content.push_str("    </marker>\n");
        
        self.content.push_str("  </defs>\n");
        
        // Add CSS styles
        self.content.push_str("  <style>\n");
        self.content.push_str("    .table-rect {\n");
        self.content.push_str("      fill: #f8f9fa;\n");
        self.content.push_str("      stroke: #343a40;\n");
        self.content.push_str("      stroke-width: 2;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .table-header {\n");
        self.content.push_str("      fill: #495057;\n");
        self.content.push_str("      stroke: #343a40;\n");
        self.content.push_str("      stroke-width: 2;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .table-title {\n");
        self.content.push_str("      font-family: Arial, sans-serif;\n");
        self.content.push_str("      font-size: 14px;\n");
        self.content.push_str("      font-weight: bold;\n");
        self.content.push_str("      fill: white;\n");
        self.content.push_str("      text-anchor: middle;\n");
        self.content.push_str("      dominant-baseline: central;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .field-text {\n");
        self.content.push_str("      font-family: Arial, sans-serif;\n");
        self.content.push_str("      font-size: 12px;\n");
        self.content.push_str("      fill: #212529;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .field-name {\n");
        self.content.push_str("      font-weight: bold;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .field-type {\n");
        self.content.push_str("      fill: #6c757d;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .field-attrs {\n");
        self.content.push_str("      fill: #28a745;\n");
        self.content.push_str("      font-style: italic;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .divider-line {\n");
        self.content.push_str("      stroke: #dee2e6;\n");
        self.content.push_str("      stroke-width: 1;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .relationship-line {\n");
        self.content.push_str("      fill: none;\n");
        self.content.push_str("      stroke: #6c757d;\n");
        self.content.push_str("      stroke-width: 2;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .relationship-label {\n");
        self.content.push_str("      font-family: Arial, sans-serif;\n");
        self.content.push_str("      font-size: 10px;\n");
        self.content.push_str("      fill: #495057;\n");
        self.content.push_str("      text-anchor: middle;\n");
        self.content.push_str("      dominant-baseline: central;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .relationship-tape {\n");
        self.content.push_str("      font-family: 'Courier New', monospace;\n");
        self.content.push_str("      font-size: 9px;\n");
        self.content.push_str("      fill: #2c3e50;\n");
        self.content.push_str("      text-anchor: middle;\n");
        self.content.push_str("      dominant-baseline: central;\n");
        self.content.push_str("      opacity: 0.9;\n");
        self.content.push_str("      font-weight: bold;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .relationship-self-ref {\n");
        self.content.push_str("      font-family: Arial, sans-serif;\n");
        self.content.push_str("      font-size: 10px;\n");
        self.content.push_str("      fill: #495057;\n");
        self.content.push_str("      text-anchor: middle;\n");
        self.content.push_str("      dominant-baseline: central;\n");
        self.content.push_str("      font-weight: bold;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .relationship-dotted {\n");
        self.content.push_str("      stroke-dasharray: 5,5;\n");
        self.content.push_str("    }\n");
        self.content.push_str("    .relationship-dashed {\n");
        self.content.push_str("      stroke-dasharray: 8,4;\n");
        self.content.push_str("    }\n");
        self.content.push_str("  </style>\n");
    }

    pub fn end_svg(&mut self) {
        self.content.push_str("</svg>");
    }

    pub fn render_table(&mut self, layout: &TableLayout, table: &crate::ast::Table) {
        let bounds = &layout.bounds;
        let table_name = &layout.table_name;
        
        let header_height = 40.0; // Match layout.rs values
        let field_height = 28.0; // Match layout.rs values
        let padding = 10.0; // Increased padding for better text spacing

        // Draw table background
        self.content.push_str(&format!(
            r#"  <rect x="{}" y="{}" width="{}" height="{}" class="table-rect" />
"#,
            bounds.x, bounds.y, bounds.width, bounds.height
        ));

        // Draw table header
        self.content.push_str(&format!(
            r#"  <rect x="{}" y="{}" width="{}" height="{}" class="table-header" />
"#,
            bounds.x, bounds.y, bounds.width, header_height
        ));

        // Draw table title
        let title_x = bounds.x + bounds.width / 2.0;
        let title_y = bounds.y + header_height / 2.0;
        self.content.push_str(&format!(
            r#"  <text x="{}" y="{}" class="table-title">{}</text>
"#,
            title_x, title_y, Self::escape_xml(table_name)
        ));

        // Draw fields
        for (i, column) in table.columns.iter().enumerate() {
            let field_y = bounds.y + header_height + (i as f64 * field_height);
            
            // Draw field divider line (except for first field)
            if i > 0 {
                self.content.push_str(&format!(
                    r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="divider-line" />
"#,
                    bounds.x, field_y, bounds.x + bounds.width, field_y
                ));
            }
            
            // Better vertical centering for text within field
            let text_y = field_y + field_height / 2.0 + 4.0; // +4 for better baseline alignment
            self.render_field(column, bounds.x + padding, text_y);
        }
    }

    fn render_field(&mut self, column: &Column, x: f64, y: f64) {
        let mut field_class = "field-text field-name";
        
        // Determine field styling based on attributes
        if column.is_primary_key() {
            field_class = "field-text pk-field";
        } else if column.is_foreign_key() {
            field_class = "field-text fk-field";
        }

        // Render field name
        self.content.push_str(&format!(
            r#"  <text x="{}" y="{}" class="{}">{}</text>
"#,
            x, y, field_class, Self::escape_xml(&column.name)
        ));

        // Render field type with proper spacing (match layout.rs calculations)
        let type_x = x + (column.name.len() as f64 * 8.0) + 15.0; // Match layout.rs font size
        self.content.push_str(&format!(
            r#"  <text x="{}" y="{}" class="field-text field-type">{}</text>
"#,
            type_x, y, Self::escape_xml(&format!("{}", column.datatype))
        ));

        // Render attributes
        if !column.attributes.is_empty() {
            let attrs_text = column.attributes.iter()
                .map(|attr| format!("{}", attr))
                .collect::<Vec<_>>()
                .join(", ");
            
            let attrs_x = type_x + (format!("{}", column.datatype).len() as f64 * 7.0) + 15.0; // Match layout.rs font size
            self.content.push_str(&format!(
                r#"  <text x="{}" y="{}" class="field-text field-attrs">[{}]</text>
"#,
                attrs_x, y, Self::escape_xml(&attrs_text)
            ));
        }
    }

    pub fn add_background(&mut self, width: f64, height: f64) {
        self.content.push_str(&format!(
            "  <rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#ffffff\" />\n",
            width, height
        ));
    }
    
    pub fn render_title(&mut self) {
        if let Some(ref title) = self.title.clone() {
            let title_x = self.canvas_width / 2.0;
            let escaped_title = Self::escape_xml(title);
            self.content.push_str(&format!(
                "  <text x=\"{}\" y=\"30\" text-anchor=\"middle\" font-size=\"24\" font-weight=\"bold\" fill=\"#2c3e50\">{}</text>\n",
                title_x,
                escaped_title
            ));
        }
    }

    fn render_marker_at_dot(&mut self, x: f64, y: f64, side: ConnectionSide, relationship_type: super::super::ast::RelationshipType, is_source: bool) {
        let label = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "1",
            super::super::ast::RelationshipType::OneToMany => if is_source { "1" } else { "M" },
            super::super::ast::RelationshipType::ManyToOne => if is_source { "M" } else { "1" },
            super::super::ast::RelationshipType::ManyToMany => "M",
        };
        
        let bg_width = 16.0;
        let bg_height = 16.0;
        let offset = 12.0;
        
        let (bg_center_x, bg_center_y) = match side {
            ConnectionSide::Left => (x - offset, y),
            ConnectionSide::Right => (x + offset, y),
            ConnectionSide::Top => (x, y - offset),
            ConnectionSide::Bottom => (x, y + offset),
        };
        
        let bg_x = bg_center_x - bg_width / 2.0;
        let bg_y = bg_center_y - bg_height / 2.0;
        
        self.content.push_str(&format!(
            "  <rect x=\"{:.6}\" y=\"{:.6}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#2c3e50\" rx=\"3\" />\n",
            bg_x, bg_y, bg_width, bg_height
        ));
        
        self.content.push_str(&format!(
            "  <text x=\"{:.6}\" y=\"{:.6}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"12\" font-weight=\"bold\" fill=\"white\">{}</text>\n",
            bg_center_x, bg_center_y, label
        ));
    }

    fn line_intersects_obstacles(&self, start: Point, end: Point) -> bool {
        // Check if line intersects any table
        for table_rect in &self.table_layouts {
            if self.line_intersects_rect(start, end, table_rect) {
                return true;
            }
        }
        
        // Check if line intersects any relationship box
        for box_layout in &self.relationship_boxes {
            if self.line_intersects_rect(start, end, &box_layout.bounds) {
                return true;
            }
        }
        
        false
    }
    
    fn line_intersects_rect(&self, start: Point, end: Point, rect: &Rectangle) -> bool {
        // Add padding around rectangle for better clearance
        let padding = 10.0;
        let rect_left = rect.x - padding;
        let rect_right = rect.x + rect.width + padding;
        let rect_top = rect.y - padding;
        let rect_bottom = rect.y + rect.height + padding;
        
        // Check if either endpoint is inside the rectangle
        if start.x >= rect_left && start.x <= rect_right && start.y >= rect_top && start.y <= rect_bottom {
            return false; // Start point is on the rectangle edge (connection point)
        }
        if end.x >= rect_left && end.x <= rect_right && end.y >= rect_top && end.y <= rect_bottom {
            return false; // End point is on the rectangle edge (connection point)
        }
        
        // Simple check: if line passes through rectangle area
        let line_min_x = start.x.min(end.x);
        let line_max_x = start.x.max(end.x);
        let line_min_y = start.y.min(end.y);
        let line_max_y = start.y.max(end.y);
        
        // Check if rectangle overlaps with line bounding box
        if rect_right < line_min_x || rect_left > line_max_x || 
           rect_bottom < line_min_y || rect_top > line_max_y {
            return false;
        }
        
        // More precise intersection check using line-rectangle intersection
        // Check intersection with all four edges
        self.line_segments_intersect(start, end, 
            Point::new(rect_left, rect_top), Point::new(rect_right, rect_top)) ||
        self.line_segments_intersect(start, end,
            Point::new(rect_right, rect_top), Point::new(rect_right, rect_bottom)) ||
        self.line_segments_intersect(start, end,
            Point::new(rect_right, rect_bottom), Point::new(rect_left, rect_bottom)) ||
        self.line_segments_intersect(start, end,
            Point::new(rect_left, rect_bottom), Point::new(rect_left, rect_top))
    }
    
    fn line_segments_intersect(&self, p1: Point, p2: Point, p3: Point, p4: Point) -> bool {
        let d = (p2.x - p1.x) * (p4.y - p3.y) - (p2.y - p1.y) * (p4.x - p3.x);
        if d.abs() < 0.001 {
            return false; // Parallel lines
        }
        
        let t = ((p3.x - p1.x) * (p4.y - p3.y) - (p3.y - p1.y) * (p4.x - p3.x)) / d;
        let u = ((p3.x - p1.x) * (p2.y - p1.y) - (p3.y - p1.y) * (p2.x - p1.x)) / d;
        
        t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0
    }
    
    fn create_curved_path(&self, start: Point, end: Point) -> String {
        // Create a quadratic Bezier curve that goes around obstacles
        let mid_x = (start.x + end.x) / 2.0;
        let mid_y = (start.y + end.y) / 2.0;
        
        // Calculate perpendicular offset for control point
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = (dx * dx + dy * dy).sqrt();
        
        if length < 0.001 {
            return format!("M {:.6} {:.6} L {:.6} {:.6}", start.x, start.y, end.x, end.y);
        }
        
        // Offset the control point perpendicular to the line
        let offset_distance = length * 0.2; // 20% of line length
        let perp_x = -dy / length * offset_distance;
        let perp_y = dx / length * offset_distance;
        
        let control_x = mid_x + perp_x;
        let control_y = mid_y + perp_y;
        
        // Check if this curve would still intersect obstacles
        // If so, try the opposite direction
        let test_mid = Point::new((start.x + control_x) / 2.0, (start.y + control_y) / 2.0);
        if self.line_intersects_obstacles(start, test_mid) || 
           self.line_intersects_obstacles(test_mid, end) {
            // Try opposite direction
            let control_x = mid_x - perp_x;
            let control_y = mid_y - perp_y;
            format!("M {:.6} {:.6} Q {:.6} {:.6} {:.6} {:.6}", 
                    start.x, start.y, control_x, control_y, end.x, end.y)
        } else {
            format!("M {:.6} {:.6} Q {:.6} {:.6} {:.6} {:.6}", 
                    start.x, start.y, control_x, control_y, end.x, end.y)
        }
    }

    pub fn render_line_segment(&mut self, start: Point, end: Point, relationship_type: super::super::ast::RelationshipType, marker_position: &str) {
        let class = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "relationship-line relationship-dashed",
            _ => "relationship-line",
        };
        
        let (marker_start, marker_end) = match marker_position {
            "start" => {
                match relationship_type {
                    super::super::ast::RelationshipType::OneToOne => ("marker-start=\"url(#one-start)\"", ""),
                    super::super::ast::RelationshipType::OneToMany => ("marker-start=\"url(#one-start)\"", ""),
                    super::super::ast::RelationshipType::ManyToOne => ("marker-start=\"url(#many-start)\"", ""),
                    super::super::ast::RelationshipType::ManyToMany => ("marker-start=\"url(#many-start)\"", ""),
                }
            },
            "end" => {
                match relationship_type {
                    super::super::ast::RelationshipType::OneToOne => ("", "marker-end=\"url(#one-end)\""),
                    super::super::ast::RelationshipType::OneToMany => ("", "marker-end=\"url(#many-end)\""),
                    super::super::ast::RelationshipType::ManyToOne => ("", "marker-end=\"url(#one-end)\""),
                    super::super::ast::RelationshipType::ManyToMany => ("", "marker-end=\"url(#many-end)\""),
                }
            },
            _ => ("", ""),
        };
        
        // Check if line intersects obstacles and use curved path if needed
        let use_curved_path = self.line_intersects_obstacles(start, end);
        
        // For many-to-many, render two parallel lines (double line effect)
        if matches!(relationship_type, super::super::ast::RelationshipType::ManyToMany) {
            // Calculate perpendicular offset for parallel lines
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let length = (dx * dx + dy * dy).sqrt();
            
            if length > 0.0 {
                let offset = 2.0; // Distance between the two parallel lines
                let perp_x = -dy / length * offset;
                let perp_y = dx / length * offset;
                
                if use_curved_path {
                    // Use curved paths for both lines
                    let path1 = self.create_curved_path(
                        Point::new(start.x + perp_x, start.y + perp_y),
                        Point::new(end.x + perp_x, end.y + perp_y)
                    );
                    let path2 = self.create_curved_path(
                        Point::new(start.x - perp_x, start.y - perp_y),
                        Point::new(end.x - perp_x, end.y - perp_y)
                    );
                    
                    self.content.push_str(&format!(
                        "  <path d=\"{}\" class=\"{}\" {} {} />\n",
                        path1, class, marker_start, marker_end
                    ));
                    self.content.push_str(&format!(
                        "  <path d=\"{}\" class=\"{}\" {} {} />\n",
                        path2, class, marker_start, marker_end
                    ));
                } else {
                    // Straight parallel lines
                    self.content.push_str(&format!(
                        "  <path d=\"M {:.6} {:.6} L {:.6} {:.6}\" class=\"{}\" {} {} />\n",
                        start.x + perp_x, start.y + perp_y, end.x + perp_x, end.y + perp_y, 
                        class, marker_start, marker_end
                    ));
                    self.content.push_str(&format!(
                        "  <path d=\"M {:.6} {:.6} L {:.6} {:.6}\" class=\"{}\" {} {} />\n",
                        start.x - perp_x, start.y - perp_y, end.x - perp_x, end.y - perp_y, 
                        class, marker_start, marker_end
                    ));
                }
            }
        } else {
            // Single line for all other relationship types
            if use_curved_path {
                let curved_path = self.create_curved_path(start, end);
                self.content.push_str(&format!(
                    "  <path d=\"{}\" class=\"{}\" {} {} />\n",
                    curved_path, class, marker_start, marker_end
                ));
            } else {
                self.content.push_str(&format!(
                    "  <path d=\"M {:.6} {:.6} L {:.6} {:.6}\" class=\"{}\" {} {} />\n",
                    start.x, start.y, end.x, end.y, class, marker_start, marker_end
                ));
            }
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    fn escape_xml(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}