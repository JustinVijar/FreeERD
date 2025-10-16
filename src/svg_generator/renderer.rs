use crate::ast::{RelationshipType, Column};
use super::layout::{Point, TableLayout, Rectangle, RelationshipBoxLayout, ConnectionSide};
use std::sync::Mutex;

// Global tracking of relationship box positions to avoid overlaps
static RELATIONSHIP_BOX_POSITIONS: Mutex<Vec<(f64, f64, f64, f64)>> = Mutex::new(Vec::new());

#[derive(Debug, Clone, Copy)]
pub enum LineStyle {
    Solid,
    Dotted,
}

#[derive(Debug, Clone, Copy)]
pub enum ArrowType {
    None,
    Single,
    Double,
}

pub struct SvgRenderer {
    content: String,
    defs_content: String,
    pub table_layouts: Vec<Rectangle>,
    pub relationship_boxes: Vec<RelationshipBoxLayout>,
}

impl SvgRenderer {
    pub fn new() -> Self {
        SvgRenderer {
            content: String::new(),
            defs_content: String::new(),
            table_layouts: Vec::new(),
            relationship_boxes: Vec::new(),
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
        // Clear relationship box positions for new diagram
        Self::clear_relationship_box_positions();
        
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

    pub fn render_relationship_line(&mut self, path: &[Point], relationship_type: super::super::ast::RelationshipType) {
        if path.is_empty() {
            return;
        }

        let path_str = if path.len() == 1 {
            // Single point - shouldn't happen but handle gracefully
            format!("M {:.1} {:.1}", path[0].x, path[0].y)
        } else if path.len() == 2 {
            // Simple line
            format!("M {:.1} {:.1} L {:.1} {:.1}", 
                   path[0].x, path[0].y, path[1].x, path[1].y)
        } else {
            // Multi-segment path
            let mut path_parts = vec![format!("M {:.1} {:.1}", path[0].x, path[0].y)];
            for point in &path[1..] {
                path_parts.push(format!("L {:.1} {:.1}", point.x, point.y));
            }
            path_parts.join(" ")
        };

        // Determine line style and markers based on relationship type
        let (class, markers) = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => {
                // One-to-one: dashed line with no arrows
                ("relationship-line relationship-dashed", "")
            },
            _ => {
                // All other relationships: solid line with arrow at end
                ("relationship-line", " marker-end=\"url(#arrow-end)\"")
            }
        };

        self.content.push_str(&format!(
            "  <path d=\"{}\" class=\"{}\"{} />\n",
            path_str, class, markers
        ));
    }


    pub fn add_background(&mut self, width: f64, height: f64) {
        self.content.push_str(&format!(
            "  <rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#ffffff\" />\n",
            width, height
        ));
    }

    pub fn render_relationship_tape_label(&mut self, from_table: &str, from_field: &str, to_table: &str, to_field: &str, relationship_type: super::super::ast::RelationshipType, path: &[Point]) {
        if path.len() < 2 {
            return;
        }
        
        // Check if this is a self-referencing relationship
        let is_self_referencing = from_table == to_table;
        
        if is_self_referencing {
            // For self-referencing relationships, render the box with proper layering
            self.render_self_referencing_relationship(from_table, from_field, to_table, to_field, relationship_type, path);
        } else {
            // For regular relationships, render as separate segments with connection box
            self.render_relationship_with_connection_box(from_table, from_field, to_table, to_field, relationship_type, path);
        }
    }
    
    fn render_relationship_with_connection_box(&mut self, from_table: &str, from_field: &str, to_table: &str, to_field: &str, relationship_type: super::super::ast::RelationshipType, path: &[Point]) {
        if path.len() < 2 {
            return;
        }
        
        let start = path[0];
        let end = path[path.len() - 1];
        
        // Create relationship text with correct operator
        let operator = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "-",
            super::super::ast::RelationshipType::OneToMany => ">",
            super::super::ast::RelationshipType::ManyToOne => "<",
            super::super::ast::RelationshipType::ManyToMany => "<>",
        };
        let relationship_text = format!("{}.{} {} {}.{}", from_table, from_field, operator, to_table, to_field);
        
        // Calculate relationship box position and dimensions
        let box_position = self.find_optimal_box_position(path, &relationship_text);
        let text_width = relationship_text.len() as f64 * 6.5;
        let text_height = 16.0;
        let padding = 6.0;
        let box_width = text_width + padding * 2.0;
        let box_height = text_height + padding * 2.0;
        
        let box_x = box_position.x - text_width / 2.0 - padding;
        let box_y = box_position.y - text_height / 2.0 - padding;
        
        // Calculate connection points on the box
        let left_connection = Point::new(box_x, box_position.y);
        let right_connection = Point::new(box_x + box_width, box_position.y);
        
        // Determine connection sides based on table positions
        let (source_connection, target_connection, source_side, target_side) = 
            self.calculate_adaptive_connections(start, end, box_position, box_x, box_y, box_width, box_height);
        
        // Render line segments FIRST (behind the box)
        // First segment: from table to box (marker at box end)
        self.render_line_segment(start, source_connection, relationship_type, "none");
        // Second segment: from box to table (marker at box start)
        self.render_line_segment(target_connection, end, relationship_type, "none");
        
        // Render the relationship box with adaptive connection dots (on top of lines)
        self.render_relationship_box_with_adaptive_dots(box_position, &relationship_text, box_x, box_y, box_width, box_height, source_side, target_side);
    }
    
    fn render_self_referencing_relationship(&mut self, from_table: &str, from_field: &str, to_table: &str, to_field: &str, relationship_type: super::super::ast::RelationshipType, path: &[Point]) {
        // Create relationship text with correct operator
        let operator = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "-",
            super::super::ast::RelationshipType::OneToMany => ">",
            super::super::ast::RelationshipType::ManyToOne => "<",
            super::super::ast::RelationshipType::ManyToMany => "<>",
        };
        let relationship_text = format!("{}.{} {} {}.{}", from_table, from_field, operator, to_table, to_field);
        
        // Calculate box position for self-referencing relationship
        let mid_index = path.len() / 2;
        let box_position = if path.len() > 2 {
            path[mid_index]
        } else {
            Point::new(
                (path[0].x + path[path.len() - 1].x) / 2.0,
                (path[0].y + path[path.len() - 1].y) / 2.0
            )
        };
        
        // Calculate box dimensions
        let text_width = relationship_text.len() as f64 * 6.5;
        let text_height = 16.0;
        let padding = 6.0;
        let box_width = text_width + padding * 2.0;
        let box_height = text_height + padding * 2.0;
        
        let box_x = box_position.x - text_width / 2.0 - padding;
        let box_y = box_position.y - text_height / 2.0 - padding;
        
        // Register this box position to prevent overlaps with future boxes
        self.register_relationship_box_position(box_position, box_width, box_height);
        
        // Render the relationship box (on top of the line that was already rendered)
        self.content.push_str(&format!(
            "  <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
             fill=\"white\" fill-opacity=\"0.95\" stroke=\"#6c757d\" stroke-width=\"1\" rx=\"4\" />\n",
            box_x, box_y, box_width, box_height
        ));
        
        // Render the relationship text
        self.content.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" class=\"relationship-self-ref\">{}</text>\n",
            box_position.x, box_position.y + 4.0, Self::escape_xml(&relationship_text)
        ));
    }
    
    fn render_relationship_box(&mut self, from_table: &str, from_field: &str, to_table: &str, to_field: &str, relationship_type: super::super::ast::RelationshipType, path: &[Point]) {
        // Create simple relationship text with correct operator first
        let operator = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "-",
            super::super::ast::RelationshipType::OneToMany => ">",
            super::super::ast::RelationshipType::ManyToOne => "<",
            super::super::ast::RelationshipType::ManyToMany => "<>",
        };
        let relationship_text = format!("{}.{} {} {}.{}", from_table, from_field, operator, to_table, to_field);
        
        // Calculate optimal position for the box along the path
        let mut box_position = self.find_optimal_box_position(path, &relationship_text);
        
        // For self-referencing relationships, use a specific position
        if from_table == to_table {
            let mid_index = path.len() / 2;
            box_position = if path.len() > 2 {
                path[mid_index]
            } else {
                Point::new(
                    (path[0].x + path[path.len() - 1].x) / 2.0,
                    (path[0].y + path[path.len() - 1].y) / 2.0
                )
            };
        }
        
        // Calculate box dimensions
        let text_width = relationship_text.len() as f64 * 6.5;
        let text_height = 16.0;
        let padding = 6.0;
        
        let box_x = box_position.x - text_width / 2.0 - padding;
        let box_y = box_position.y - text_height / 2.0 - padding;
        let box_width = text_width + padding * 2.0;
        let box_height = text_height + padding * 2.0;
        
        // Render background box
        self.content.push_str(&format!(
            "  <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
             fill=\"white\" fill-opacity=\"0.95\" stroke=\"#6c757d\" stroke-width=\"1\" rx=\"4\" />\n",
            box_x, box_y, box_width, box_height
        ));
        
        // Render the relationship text
        self.content.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" class=\"relationship-self-ref\">{}</text>\n",
            box_position.x, box_position.y + 4.0, Self::escape_xml(&relationship_text)
        ));
        
        // Add connection dots (circles) on the left and right sides of the box
        let dot_radius = 3.0;
        let left_dot_x = box_x;
        let right_dot_x = box_x + box_width;
        let dot_y = box_position.y;
        
        // Left connection dot (from source table)
        self.content.push_str(&format!(
            "  <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
            left_dot_x, dot_y, dot_radius
        ));
        
        // Right connection dot (to target table)
        self.content.push_str(&format!(
            "  <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
            right_dot_x, dot_y, dot_radius
        ));
    }
    
    fn calculate_adaptive_connections(&self, start: Point, end: Point, box_position: Point, box_x: f64, box_y: f64, box_width: f64, box_height: f64) -> (Point, Point, ConnectionSide, ConnectionSide) {
        // Calculate the primary direction from start to end
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        
        // Determine if the connection is more horizontal or vertical
        let is_horizontal = dx.abs() > dy.abs();
        
        let (source_connection, source_side) = if is_horizontal {
            // Horizontal connection: use left/right sides
            if start.x < box_position.x {
                // Source is to the left of box
                (Point::new(box_x, box_position.y), ConnectionSide::Left)
            } else {
                // Source is to the right of box
                (Point::new(box_x + box_width, box_position.y), ConnectionSide::Right)
            }
        } else {
            // Vertical connection: use top/bottom sides
            if start.y < box_position.y {
                // Source is above the box
                (Point::new(box_position.x, box_y), ConnectionSide::Top)
            } else {
                // Source is below the box
                (Point::new(box_position.x, box_y + box_height), ConnectionSide::Bottom)
            }
        };
        
        let (target_connection, target_side) = if is_horizontal {
            // Horizontal connection: use left/right sides
            if end.x > box_position.x {
                // Target is to the right of box
                (Point::new(box_x + box_width, box_position.y), ConnectionSide::Right)
            } else {
                // Target is to the left of box
                (Point::new(box_x, box_position.y), ConnectionSide::Left)
            }
        } else {
            // Vertical connection: use top/bottom sides
            if end.y > box_position.y {
                // Target is below the box
                (Point::new(box_position.x, box_y + box_height), ConnectionSide::Bottom)
            } else {
                // Target is above the box
                (Point::new(box_position.x, box_y), ConnectionSide::Top)
            }
        };
        
        (source_connection, target_connection, source_side, target_side)
    }
    
    fn get_exact_dot_positions(&self, box_position: Point, box_x: f64, box_y: f64, box_width: f64, box_height: f64, source_side: ConnectionSide, target_side: ConnectionSide) -> (Point, Point) {
        // Calculate EXACT dot positions (must match the dots rendered in render_relationship_box_with_adaptive_dots)
        let source_dot = match source_side {
            ConnectionSide::Left => Point::new(box_x, box_position.y),
            ConnectionSide::Right => Point::new(box_x + box_width, box_position.y),
            ConnectionSide::Top => Point::new(box_position.x, box_y),
            ConnectionSide::Bottom => Point::new(box_position.x, box_y + box_height),
        };
        
        let target_dot = match target_side {
            ConnectionSide::Left => Point::new(box_x, box_position.y),
            ConnectionSide::Right => Point::new(box_x + box_width, box_position.y),
            ConnectionSide::Top => Point::new(box_position.x, box_y),
            ConnectionSide::Bottom => Point::new(box_position.x, box_y + box_height),
        };
        
        (source_dot, target_dot)
    }
    
    pub fn render_line_segment(&mut self, start: Point, end: Point, relationship_type: super::super::ast::RelationshipType, marker_position: &str) {
        // Determine line style based on relationship type
        let class = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "relationship-line relationship-dashed",
            _ => "relationship-line",
        };
        
        // Determine markers based on relationship type and position (crow's foot notation)
        let (marker_start, marker_end) = match marker_position {
            "start" => {
                // Marker at start of line (source table end)
                match relationship_type {
                    super::super::ast::RelationshipType::OneToOne => {
                        ("marker-start=\"url(#one-start)\"", "")
                    },
                    super::super::ast::RelationshipType::OneToMany => {
                        ("marker-start=\"url(#one-start)\"", "")
                    },
                    super::super::ast::RelationshipType::ManyToOne => {
                        ("marker-start=\"url(#many-start)\"", "")
                    },
                    super::super::ast::RelationshipType::ManyToMany => {
                        ("marker-start=\"url(#many-start)\"", "")
                    },
                }
            },
            "end" => {
                // Marker at end of line (target table end)
                match relationship_type {
                    super::super::ast::RelationshipType::OneToOne => {
                        ("", "marker-end=\"url(#one-end)\"")
                    },
                    super::super::ast::RelationshipType::OneToMany => {
                        ("", "marker-end=\"url(#many-end)\"")
                    },
                    super::super::ast::RelationshipType::ManyToOne => {
                        ("", "marker-end=\"url(#one-end)\"")
                    },
                    super::super::ast::RelationshipType::ManyToMany => {
                        ("", "marker-end=\"url(#many-end)\"")
                    },
                }
            },
            _ => ("", ""), // No markers
        };
        
        // Use high precision coordinates to ensure perfect connection to dots
        self.content.push_str(&format!(
            "  <path d=\"M {:.6} {:.6} L {:.6} {:.6}\" class=\"{}\" {} {} />\n",
            start.x, start.y, end.x, end.y, class, marker_start, marker_end
        ));
    }
    
    fn render_relationship_box_with_dots(&mut self, box_position: Point, relationship_text: &str, box_x: f64, box_y: f64, box_width: f64, box_height: f64) {
        // Render background box
        self.content.push_str(&format!(
            "  <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
             fill=\"white\" fill-opacity=\"0.95\" stroke=\"#6c757d\" stroke-width=\"1\" rx=\"4\" />\n",
            box_x, box_y, box_width, box_height
        ));
        
        // Render the relationship text
        self.content.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" class=\"relationship-self-ref\">{}</text>\n",
            box_position.x, box_position.y + 4.0, Self::escape_xml(&relationship_text)
        ));
        
        // Add connection dots (circles) on the left and right sides of the box
        let dot_radius = 3.0;
        let left_dot_x = box_x;
        let right_dot_x = box_x + box_width;
        let dot_y = box_position.y;
        
        // Left connection dot (from source table)
        self.content.push_str(&format!(
            "  <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
            left_dot_x, dot_y, dot_radius
        ));
        
        // Right connection dot (to target table)
        self.content.push_str(&format!(
            "  <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
            right_dot_x, dot_y, dot_radius
        ));
    }
    
    fn render_relationship_box_with_adaptive_dots(&mut self, box_position: Point, relationship_text: &str, box_x: f64, box_y: f64, box_width: f64, box_height: f64, source_side: ConnectionSide, target_side: ConnectionSide) {
        // Extract relationship type from the text to determine markers
        let relationship_type = if relationship_text.contains(" <> ") {
            super::super::ast::RelationshipType::ManyToMany
        } else if relationship_text.contains(" > ") {
            super::super::ast::RelationshipType::OneToMany
        } else if relationship_text.contains(" < ") {
            super::super::ast::RelationshipType::ManyToOne
        } else {
            super::super::ast::RelationshipType::OneToOne
        };
        
        // Render background box
        self.content.push_str(&format!(
            "  <rect x=\"{:.6}\" y=\"{:.6}\" width=\"{:.6}\" height=\"{:.6}\" \
             fill=\"white\" fill-opacity=\"0.95\" stroke=\"#6c757d\" stroke-width=\"1\" rx=\"4\" />\n",
            box_x, box_y, box_width, box_height
        ));
        
        // Render the relationship text
        self.content.push_str(&format!(
            "  <text x=\"{:.6}\" y=\"{:.6}\" class=\"relationship-self-ref\">{}</text>\n",
            box_position.x, box_position.y + 4.0, Self::escape_xml(&relationship_text)
        ));
        
        // Add adaptive connection dots based on connection sides
        let dot_radius = 3.0;
        
        // Calculate EXACT connection points (must match line endpoints exactly)
        let (source_dot_x, source_dot_y) = match source_side {
            ConnectionSide::Left => (box_x, box_position.y),
            ConnectionSide::Right => (box_x + box_width, box_position.y),
            ConnectionSide::Top => (box_position.x, box_y),
            ConnectionSide::Bottom => (box_position.x, box_y + box_height),
        };
        
        let (target_dot_x, target_dot_y) = match target_side {
            ConnectionSide::Left => (box_x, box_position.y),
            ConnectionSide::Right => (box_x + box_width, box_position.y),
            ConnectionSide::Top => (box_position.x, box_y),
            ConnectionSide::Bottom => (box_position.x, box_y + box_height),
        };
        
        // Render source connection dot with high precision
        self.content.push_str(&format!(
            "  <circle cx=\"{:.6}\" cy=\"{:.6}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
            source_dot_x, source_dot_y, dot_radius
        ));
        
        // Only render target dot if it's at a different position than source dot
        if (source_dot_x - target_dot_x).abs() > 0.1 || (source_dot_y - target_dot_y).abs() > 0.1 {
            self.content.push_str(&format!(
                "  <circle cx=\"{:.6}\" cy=\"{:.6}\" r=\"{:.1}\" fill=\"#2c3e50\" />\n",
                target_dot_x, target_dot_y, dot_radius
            ));
        }
        
        // ALWAYS render cardinality labels for both sides (even if dots are at same position)
        self.render_marker_at_dot(source_dot_x, source_dot_y, source_side, relationship_type, true);
        self.render_marker_at_dot(target_dot_x, target_dot_y, target_side, relationship_type, false);
    }
    
    fn render_marker_at_dot(&mut self, x: f64, y: f64, side: ConnectionSide, relationship_type: super::super::ast::RelationshipType, is_source: bool) {
        // Determine which label to use: "1" for one, "M" for many
        let label = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "1",
            super::super::ast::RelationshipType::OneToMany => if is_source { "1" } else { "M" },
            super::super::ast::RelationshipType::ManyToOne => if is_source { "M" } else { "1" },
            super::super::ast::RelationshipType::ManyToMany => "M",
        };
        
        // Background rectangle dimensions
        let bg_width = 16.0;
        let bg_height = 16.0;
        let offset = 12.0; // Distance from the dot to center of box
        
        // Calculate background rectangle center position based on connection side
        let (bg_center_x, bg_center_y) = match side {
            ConnectionSide::Left => (x - offset, y),      // Left of box
            ConnectionSide::Right => (x + offset, y),     // Right of box
            ConnectionSide::Top => (x, y - offset),       // Above box
            ConnectionSide::Bottom => (x, y + offset),    // Below box
        };
        
        // Calculate top-left corner of background rectangle
        let bg_x = bg_center_x - bg_width / 2.0;
        let bg_y = bg_center_y - bg_height / 2.0;
        
        // Render background rectangle (black with slight rounding)
        self.content.push_str(&format!(
            "  <rect x=\"{:.6}\" y=\"{:.6}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#2c3e50\" rx=\"3\" />\n",
            bg_x, bg_y, bg_width, bg_height
        ));
        
        // Text position: centered in the box
        // For vertical centering, we use dominant-baseline="middle" for true vertical centering
        let text_x = bg_center_x;
        let text_y = bg_center_y;
        
        // Render the text label (white text on black background, centered)
        self.content.push_str(&format!(
            "  <text x=\"{:.6}\" y=\"{:.6}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"12\" font-weight=\"bold\" fill=\"white\">{}</text>\n",
            text_x, text_y, label
        ));
    }
    
    pub fn render_relationship_line_segments_only(&mut self, from_table: &str, from_field: &str, to_table: &str, to_field: &str, relationship_type: super::super::ast::RelationshipType, path: &[Point]) {
        if path.len() < 2 {
            return;
        }
        
        let start = path[0];
        let end = path[path.len() - 1];
        
        // Create relationship text to calculate box position
        let operator = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "-",
            super::super::ast::RelationshipType::OneToMany => ">",
            super::super::ast::RelationshipType::ManyToOne => "<",
            super::super::ast::RelationshipType::ManyToMany => "<>",
        };
        let relationship_text = format!("{}.{} {} {}.{}", from_table, from_field, operator, to_table, to_field);
        
        // Calculate optimal box position (after collision avoidance)
        let box_position = self.find_optimal_box_position(path, &relationship_text);
        let text_width = relationship_text.len() as f64 * 6.5;
        let text_height = 16.0;
        let padding = 6.0;
        let box_width = text_width + padding * 2.0;
        let box_height = text_height + padding * 2.0;
        
        let box_x = box_position.x - text_width / 2.0 - padding;
        let box_y = box_position.y - text_height / 2.0 - padding;
        
        // Calculate connection sides and get exact dot positions
        let (_, _, source_side, target_side) = 
            self.calculate_adaptive_connections(start, end, box_position, box_x, box_y, box_width, box_height);
        
        // Get exact dot positions that will be rendered
        let (source_dot, target_dot) = self.get_exact_dot_positions(box_position, box_x, box_y, box_width, box_height, source_side, target_side);
        
        // Render line segments connecting EXACTLY to the dot positions WITHOUT markers
        // (markers are drawn at the box dots instead)
        self.render_line_segment(start, source_dot, relationship_type, "none");
        self.render_line_segment(target_dot, end, relationship_type, "none");
    }
    
    pub fn render_relationship_box_only(&mut self, from_table: &str, from_field: &str, to_table: &str, to_field: &str, relationship_type: super::super::ast::RelationshipType, path: &[Point]) {
        if path.len() < 2 {
            return;
        }
        
        // Check if this is a self-referencing relationship
        let is_self_referencing = from_table == to_table;
        
        if is_self_referencing {
            // For self-referencing relationships, render the box
            self.render_self_referencing_relationship(from_table, from_field, to_table, to_field, relationship_type, path);
        } else {
            // For regular relationships, render the box with adaptive dots
            let start = path[0];
            let end = path[path.len() - 1];
            
            // Create relationship text
            let operator = match relationship_type {
                super::super::ast::RelationshipType::OneToOne => "-",
                super::super::ast::RelationshipType::OneToMany => ">",
                super::super::ast::RelationshipType::ManyToOne => "<",
                super::super::ast::RelationshipType::ManyToMany => "<>",
            };
            let relationship_text = format!("{}.{} {} {}.{}", from_table, from_field, operator, to_table, to_field);
            
            // Calculate box position and dimensions
            let box_position = self.find_optimal_box_position(path, &relationship_text);
            let text_width = relationship_text.len() as f64 * 6.5;
            let text_height = 16.0;
            let padding = 6.0;
            let box_width = text_width + padding * 2.0;
            let box_height = text_height + padding * 2.0;
            
            let box_x = box_position.x - text_width / 2.0 - padding;
            let box_y = box_position.y - text_height / 2.0 - padding;
            
            // Calculate connection sides
            let (_, _, source_side, target_side) = 
                self.calculate_adaptive_connections(start, end, box_position, box_x, box_y, box_width, box_height);
            
            // Register this box position to prevent overlaps with future boxes
            self.register_relationship_box_position(box_position, box_width, box_height);
        
            // Render ONLY the relationship box with adaptive dots
            self.render_relationship_box_with_adaptive_dots(box_position, &relationship_text, box_x, box_y, box_width, box_height, source_side, target_side);
        }
    }
    
    fn find_optimal_box_position(&self, path: &[Point], relationship_text: &str) -> Point {
        if path.len() < 2 {
            return path[0];
        }
        
        let start = path[0];
        let end = path[path.len() - 1];
        
        // Calculate box dimensions for collision detection
        let text_width = relationship_text.len() as f64 * 6.5;
        let text_height = 16.0;
        let padding = 6.0;
        let box_width = text_width + padding * 2.0;
        let box_height = text_height + padding * 2.0;
        
        // First, try to find open space in the canvas by avoiding table-dense areas
        let safe_position = self.find_open_canvas_space(start, end, box_width, box_height);
        if let Some(pos) = safe_position {
            return pos;
        }
        
        // Try multiple positions along the line to find the best one
        let candidate_positions = vec![0.5, 0.4, 0.6, 0.3, 0.7, 0.25, 0.75, 0.2, 0.8, 0.15, 0.85];
        
        for &ratio in &candidate_positions {
            let candidate = Point::new(
                start.x + (end.x - start.x) * ratio,
                start.y + (end.y - start.y) * ratio
            );
            
            // Check if this position is safe (not overlapping with tables or other boxes)
            if self.is_position_safe_from_tables_and_boxes(candidate, box_width, box_height) {
                return candidate;
            }
        }
        
        // If no safe position found along the line, try perpendicular offsets
        let midpoint = Point::new(
            (start.x + end.x) / 2.0,
            (start.y + end.y) / 2.0
        );
        
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance > 0.0 {
            // Try perpendicular offsets at different distances (increased range)
            let offsets = vec![80.0, 100.0, 120.0, 150.0, 180.0, 200.0];
            
            for &offset_distance in &offsets {
                let perpendicular_x = -dy / distance * offset_distance;
                let perpendicular_y = dx / distance * offset_distance;
                
                // Try both sides of the line
                let candidates = vec![
                    Point::new(midpoint.x + perpendicular_x, midpoint.y + perpendicular_y),
                    Point::new(midpoint.x - perpendicular_x, midpoint.y - perpendicular_y),
                ];
                
                for candidate in candidates {
                    if self.is_position_safe_from_tables_and_boxes(candidate, box_width, box_height) {
                        return candidate;
                    }
                }
            }
        }
        
        // Last resort: find any open space in the canvas
        self.find_any_open_space(box_width, box_height).unwrap_or_else(|| {
            // Ultimate fallback
            Point::new(midpoint.x, midpoint.y - 200.0)
        })
    }
    
    fn is_position_safe_from_tables_and_boxes(&self, position: Point, box_width: f64, box_height: f64) -> bool {
        // Calculate the bounding box for the relationship box
        let box_left = position.x - box_width / 2.0;
        let box_right = position.x + box_width / 2.0;
        let box_top = position.y - box_height / 2.0;
        let box_bottom = position.y + box_height / 2.0;
        
        // Minimum safe distance from tables and other boxes
        let safe_margin = 50.0; // Increased margin for better clearance
        let box_spacing = 40.0; // Minimum spacing between relationship boxes
        
        // Check collision with actual table layouts
        for table_rect in &self.table_layouts {
            let table_left = table_rect.x;
            let table_top = table_rect.y;
            let table_right = table_rect.x + table_rect.width;
            let table_bottom = table_rect.y + table_rect.height;
            
            if !(box_right + safe_margin < table_left || 
                 box_left - safe_margin > table_right ||
                 box_bottom + safe_margin < table_top ||
                 box_top - safe_margin > table_bottom) {
                return false; // Collision detected with actual table
            }
        }
        
        // Check collision with existing relationship boxes
        if let Ok(existing_boxes) = RELATIONSHIP_BOX_POSITIONS.lock() {
            for &(existing_left, existing_top, existing_right, existing_bottom) in existing_boxes.iter() {
                if !(box_right + box_spacing < existing_left || 
                     box_left - box_spacing > existing_right ||
                     box_bottom + box_spacing < existing_top ||
                     box_top - box_spacing > existing_bottom) {
                    return false; // Collision with existing relationship box
                }
            }
        }
        
        // Check if position is within canvas bounds
        if position.x < 50.0 || position.x > 2550.0 || position.y < 50.0 || position.y > 1850.0 {
            return false;
        }
        
        true
    }
    
    fn register_relationship_box_position(&self, position: Point, box_width: f64, box_height: f64) {
        let box_left = position.x - box_width / 2.0;
        let box_right = position.x + box_width / 2.0;
        let box_top = position.y - box_height / 2.0;
        let box_bottom = position.y + box_height / 2.0;
        
        if let Ok(mut existing_boxes) = RELATIONSHIP_BOX_POSITIONS.lock() {
            existing_boxes.push((box_left, box_top, box_right, box_bottom));
        }
    }
    
    pub fn clear_relationship_box_positions() {
        if let Ok(mut existing_boxes) = RELATIONSHIP_BOX_POSITIONS.lock() {
            existing_boxes.clear();
        }
    }
    
    fn find_open_canvas_space(&self, start: Point, end: Point, box_width: f64, box_height: f64) -> Option<Point> {
        // Define regions to search for open space, prioritizing areas between tables
        let canvas_width = 2600.0;
        let canvas_height = 1900.0;
        
        // Calculate midpoint for reference
        let midpoint = Point::new(
            (start.x + end.x) / 2.0,
            (start.y + end.y) / 2.0
        );
        
        // Define search regions (areas likely to be free of tables)
        let search_regions = vec![
            // Between table rows
            (200.0, 200.0, canvas_width - 200.0, 280.0),   // Between top and middle rows
            (200.0, 520.0, canvas_width - 200.0, 600.0),   // Between middle and bottom rows
            
            // Side margins
            (50.0, 100.0, 150.0, canvas_height - 100.0),   // Left margin
            (canvas_width - 150.0, 100.0, canvas_width - 50.0, canvas_height - 100.0), // Right margin
            
            // Top and bottom margins
            (200.0, 50.0, canvas_width - 200.0, 100.0),    // Top margin
            (200.0, canvas_height - 100.0, canvas_width - 200.0, canvas_height - 50.0), // Bottom margin
        ];
        
        // For each region, try to find a position close to the relationship midpoint
        for &(region_left, region_top, region_right, region_bottom) in &search_regions {
            // Try positions within this region, starting from closest to midpoint
            let region_center_x = (region_left + region_right) / 2.0;
            let region_center_y = (region_top + region_bottom) / 2.0;
            
            // Calculate distance from midpoint to region center
            let dx = region_center_x - midpoint.x;
            let dy = region_center_y - midpoint.y;
            let distance_to_region = (dx * dx + dy * dy).sqrt();
            
            // Skip regions that are too far from the relationship
            if distance_to_region > 400.0 {
                continue;
            }
            
            // Try several positions within this region
            let positions_to_try = vec![
                Point::new(region_center_x, region_center_y),
                Point::new(region_left + 50.0, region_center_y),
                Point::new(region_right - 50.0, region_center_y),
                Point::new(region_center_x, region_top + 30.0),
                Point::new(region_center_x, region_bottom - 30.0),
            ];
            
            for candidate in positions_to_try {
                // Check if candidate is within region bounds
                if candidate.x >= region_left && candidate.x <= region_right &&
                   candidate.y >= region_top && candidate.y <= region_bottom {
                    
                    if self.is_position_safe_from_tables_and_boxes(candidate, box_width, box_height) {
                        return Some(candidate);
                    }
                }
            }
        }
        
        None
    }
    
    fn find_any_open_space(&self, box_width: f64, box_height: f64) -> Option<Point> {
        // Grid search across the entire canvas for any open space
        let canvas_width = 2600.0;
        let canvas_height = 1900.0;
        let step_size = 100.0;
        
        for y in (100..((canvas_height - 100.0) as i32)).step_by(step_size as usize) {
            for x in (100..((canvas_width - 100.0) as i32)).step_by(step_size as usize) {
                let candidate = Point::new(x as f64, y as f64);
                
                if self.is_position_safe_from_tables_and_boxes(candidate, box_width, box_height) {
                    return Some(candidate);
                }
            }
        }
        
        None
    }
    
    fn interpolate_path_position(&self, path: &[Point], ratio: f64) -> Point {
        if path.len() < 2 {
            return path[0];
        }
        
        let index = ((path.len() - 1) as f64 * ratio) as usize;
        if index < path.len() {
            path[index]
        } else {
            // Interpolate between start and end
            let start = path[0];
            let end = path[path.len() - 1];
            Point::new(
                start.x + (end.x - start.x) * ratio,
                start.y + (end.y - start.y) * ratio
            )
        }
    }
    
    fn is_position_safe(&self, position: Point, box_width: f64, box_height: f64, margin: f64) -> bool {
        // For now, implement a simple check
        // TODO: Add actual table bounds checking when table layouts are available
        // This is a placeholder that always returns true for center positions
        // In a full implementation, this would check against all table rectangles
        true
    }
    
    fn find_offset_position(&self, center: Point, box_width: f64, box_height: f64, margin: f64) -> Point {
        // Simple offset strategy: try moving the box slightly in different directions
        let offsets = vec![
            (0.0, -30.0),   // Up
            (0.0, 30.0),    // Down
            (-30.0, 0.0),   // Left
            (30.0, 0.0),    // Right
        ];
        
        for (dx, dy) in offsets {
            let offset_position = Point::new(center.x + dx, center.y + dy);
            if self.is_position_safe(offset_position, box_width, box_height, margin) {
                return offset_position;
            }
        }
        
        // Fallback to original center position
        center
    }
    
    fn render_tape_pattern(&mut self, from_table: &str, from_field: &str, to_table: &str, to_field: &str, relationship_type: super::super::ast::RelationshipType, path: &[Point]) {
        // Calculate the path length and position for the tape
        let start = path[0];
        let end = path[path.len() - 1];
        
        // Calculate angle of the line for text rotation
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let mut angle = dy.atan2(dx) * 180.0 / std::f64::consts::PI;
        
        // Prevent upside-down text by flipping angle if it's between 90 and 270 degrees
        let mut _flipped = false;
        if angle > 90.0 || angle < -90.0 {
            angle += 180.0;
            if angle > 180.0 {
                angle -= 360.0;
            }
            _flipped = true;
        }
        
        // Create dynamic tape pattern based on line length with correct operator
        let operator = match relationship_type {
            super::super::ast::RelationshipType::OneToOne => "-",
            super::super::ast::RelationshipType::OneToMany => ">",
            super::super::ast::RelationshipType::ManyToOne => "<",
            super::super::ast::RelationshipType::ManyToMany => "<>",
        };
        let relationship_text = format!("{}.{} {} {}.{}", from_table, from_field, operator, to_table, to_field);
        let line_length = ((dx * dx) + (dy * dy)).sqrt();
        
        // Calculate how many repetitions fit based on line length
        let text_width_estimate = relationship_text.len() as f64 * 6.0; // Approximate character width
        let separator_width = 20.0; // Space for " | "
        let ellipsis_width = 30.0; // Space for "..."
        let available_width = line_length - ellipsis_width;
        
        let repetitions = ((available_width / (text_width_estimate + separator_width)).floor() as usize).max(1).min(5);
        
        // Build dynamic tape pattern
        let mut tape_parts = Vec::new();
        for _ in 0..repetitions {
            tape_parts.push(relationship_text.clone());
        }
        let tape_pattern = format!("...{} | {}...", tape_parts.join(" | "), tape_parts[0]);
        
        // Calculate position closer to the line start/end for better ellipsis placement
        let _line_length = ((dx * dx) + (dy * dy)).sqrt();
        let position_ratio = 0.5; // Keep at center but adjust offset
        
        let text_x = start.x + dx * position_ratio;
        let text_y = start.y + dy * position_ratio;
        
        // Render tape labels both above and below the line for better visibility
        let offset = 20.0;
        let perpendicular_angle = angle + 90.0;
        let offset_x = offset * (perpendicular_angle * std::f64::consts::PI / 180.0).cos();
        let offset_y = offset * (perpendicular_angle * std::f64::consts::PI / 180.0).sin();
        
        // Top tape label
        let top_x = text_x + offset_x;
        let top_y = text_y + offset_y;
        
        // Bottom tape label (opposite side)
        let bottom_x = text_x - offset_x;
        let bottom_y = text_y - offset_y;
        
        // Render both tape labels with escaped XML
        self.content.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" class=\"relationship-tape\" transform=\"rotate({:.1} {:.1} {:.1})\">{}</text>\n",
            top_x, top_y, angle, top_x, top_y, Self::escape_xml(&tape_pattern)
        ));
        
        self.content.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" class=\"relationship-tape\" transform=\"rotate({:.1} {:.1} {:.1})\">{}</text>\n",
            bottom_x, bottom_y, angle, bottom_x, bottom_y, Self::escape_xml(&tape_pattern)
        ));
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