use crate::ast::*;
use std::collections::HashMap;

pub struct SvgGenerator {
    schema: Schema,
    config: SvgConfig,
}

#[derive(Debug, Clone, Copy)]
enum ConnectionSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct SvgConfig {
    pub width: usize,
    pub height: usize,
    pub table_width: usize,
    pub table_padding: usize,
    pub row_height: usize,
    pub margin: usize,
    pub font_size: usize,
    pub title_font_size: usize,
    pub background_color: String,
    pub table_header_color: String,
    pub table_body_color: String,
    pub border_color: String,
    pub text_color: String,
    pub relationship_color: String,
}

impl Default for SvgConfig {
    fn default() -> Self {
        SvgConfig {
            width: 1200,
            height: 800,
            table_width: 280,
            table_padding: 15,
            row_height: 25,
            margin: 50,
            font_size: 14,
            title_font_size: 24,
            background_color: "#ffffff".to_string(),
            table_header_color: "#4a90e2".to_string(),
            table_body_color: "#f5f5f5".to_string(),
            border_color: "#333333".to_string(),
            text_color: "#333333".to_string(),
            relationship_color: "#666666".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct TableLayout {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl SvgGenerator {
    pub fn new(schema: Schema) -> Self {
        SvgGenerator {
            schema,
            config: SvgConfig::default(),
        }
    }
    
    pub fn with_config(schema: Schema, config: SvgConfig) -> Self {
        SvgGenerator { schema, config }
    }
    
    pub fn generate(&self) -> String {
        let mut svg = String::new();
        
        // Calculate layout
        let table_layouts = self.calculate_layout();
        
        // Adjust canvas size based on actual content
        let (canvas_width, canvas_height) = self.calculate_canvas_size(&table_layouts);
        
        // SVG header
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            canvas_width, canvas_height, canvas_width, canvas_height
        ));
        svg.push('\n');
        
        // Background
        svg.push_str(&format!(
            r#"  <rect width="100%" height="100%" fill="{}"/>"#,
            self.config.background_color
        ));
        svg.push('\n');
        
        // Title
        if let Some(title) = &self.schema.title {
            svg.push_str(&self.generate_title(title));
        }
        
        // Draw relationships first (so they appear behind tables and don't obscure content)
        svg.push_str(&self.generate_relationships(&table_layouts));
        
        // Draw tables on top
        for (table, layout) in self.schema.tables.iter().zip(table_layouts.iter()) {
            svg.push_str(&self.generate_table(table, layout));
        }
        
        // SVG footer
        svg.push_str("</svg>");
        
        svg
    }
    
    fn calculate_layout(&self) -> Vec<TableLayout> {
        let mut layouts = Vec::new();
        let tables_per_row = 3;
        let horizontal_spacing = 180;  // Increased from 120 to 180 for more clearance
        let vertical_spacing = 140;    // Increased from 100 to 140 for more clearance
        
        // First pass: calculate all table heights
        let table_heights: Vec<usize> = self.schema.tables.iter()
            .map(|table| self.calculate_table_height(table))
            .collect();
        
        // Calculate row heights (max height in each row)
        let num_rows = (self.schema.tables.len() + tables_per_row - 1) / tables_per_row;
        let mut row_heights = vec![0; num_rows];
        
        for (i, height) in table_heights.iter().enumerate() {
            let row = i / tables_per_row;
            if *height > row_heights[row] {
                row_heights[row] = *height;
            }
        }
        
        // Calculate cumulative Y positions for each row
        let mut row_y_positions = vec![0; num_rows];
        let title_offset = if self.schema.title.is_some() { 80 } else { 0 };
        
        row_y_positions[0] = self.config.margin + title_offset;
        for i in 1..num_rows {
            row_y_positions[i] = row_y_positions[i - 1] + row_heights[i - 1] + vertical_spacing;
        }
        
        // Second pass: position tables
        for (i, _table) in self.schema.tables.iter().enumerate() {
            let row = i / tables_per_row;
            let col = i % tables_per_row;
            
            let table_height = table_heights[i];
            
            let x = self.config.margin + col * (self.config.table_width + horizontal_spacing);
            let y = row_y_positions[row];
            
            layouts.push(TableLayout {
                x,
                y,
                width: self.config.table_width,
                height: table_height,
            });
        }
        
        layouts
    }
    
    fn calculate_canvas_size(&self, layouts: &[TableLayout]) -> (usize, usize) {
        if layouts.is_empty() {
            return (self.config.width, self.config.height);
        }
        
        let mut max_x = 0;
        let mut max_y = 0;
        
        for layout in layouts {
            let right = layout.x + layout.width;
            let bottom = layout.y + layout.height;
            
            if right > max_x {
                max_x = right;
            }
            if bottom > max_y {
                max_y = bottom;
            }
        }
        
        // Add extra margin for relationship lines (they extend beyond tables)
        let line_clearance = 100; // Extra space for routing lines
        let canvas_width = max_x + self.config.margin + line_clearance;
        let canvas_height = max_y + self.config.margin + line_clearance;
        
        // No minimum size restriction - let it grow as needed
        (canvas_width, canvas_height)
    }
    
    fn calculate_table_height(&self, table: &Table) -> usize {
        let header_height = self.config.row_height + 10;
        // Increased row height to accommodate attributes
        let row_height = self.config.row_height + 10; // Extra space for attributes
        let rows_height = table.columns.len() * row_height;
        header_height + rows_height + self.config.table_padding * 2
    }
    
    fn generate_title(&self, title: &str) -> String {
        format!(
            r#"  <text x="50%" y="{}" text-anchor="middle" font-size="{}" font-weight="bold" fill="{}">{}</text>"#,
            self.config.margin - 10,
            self.config.title_font_size,
            self.config.text_color,
            escape_xml(title)
        ) + "\n"
    }
    
    fn generate_table(&self, table: &Table, layout: &TableLayout) -> String {
        let mut svg = String::new();
        
        // Table container group
        svg.push_str(&format!(r#"  <g class="table" id="table-{}">"#, table.name));
        svg.push('\n');
        
        // Table background
        svg.push_str(&format!(
            r#"    <rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="2" rx="5"/>"#,
            layout.x, layout.y, layout.width, layout.height,
            self.config.table_body_color, self.config.border_color
        ));
        svg.push('\n');
        
        // Table header
        let header_height = self.config.row_height + 10;
        svg.push_str(&format!(
            r#"    <rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="2" rx="5"/>"#,
            layout.x, layout.y, layout.width, header_height,
            self.config.table_header_color, self.config.border_color
        ));
        svg.push('\n');
        
        // Table name
        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" font-size="{}" font-weight="bold" fill="white">{}</text>"#,
            layout.x + 10,
            layout.y + header_height - 10,
            self.config.font_size + 2,
            escape_xml(&table.name)
        ));
        svg.push('\n');
        
        // Columns with dividers
        let row_height = self.config.row_height + 10; // Match increased row height
        let mut y_offset = layout.y + header_height + self.config.table_padding;
        for (i, column) in table.columns.iter().enumerate() {
            // Draw divider line before each column (except first)
            if i > 0 {
                svg.push_str(&format!(
                    r#"    <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1" opacity="0.3"/>"#,
                    layout.x,
                    y_offset - 5,
                    layout.x + layout.width,
                    y_offset - 5,
                    self.config.border_color
                ));
                svg.push('\n');
            }
            
            svg.push_str(&self.generate_column(column, layout.x, layout.width, y_offset, &table.name));
            y_offset += row_height;
        }
        
        svg.push_str("  </g>\n");
        svg
    }
    
    fn generate_column(&self, column: &Column, table_x: usize, _table_width: usize, y: usize, table_name: &str) -> String {
        let mut svg = String::new();
        let padding = 10;
        
        // Column name with key indicator
        let name_text = column.name.clone();
        let icon = if column.is_primary_key() {
            "🔑 "
        } else if column.is_foreign_key() {
            "🔗 "
        } else {
            ""
        };
        
        // Main column name and type
        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" font-size="{}" font-weight="bold" fill="{}" class="field" id="field-{}-{}">{}{}: {}</text>"#,
            table_x + padding,
            y + self.config.font_size,
            self.config.font_size,
            self.config.text_color,
            table_name,
            column.name,
            icon,
            escape_xml(&name_text),
            column.datatype
        ));
        svg.push('\n');
        
        // Attributes in a compact format
        if !column.attributes.is_empty() {
            let attrs: Vec<String> = column.attributes.iter()
                .map(|a| match a {
                    Attribute::PrimaryKey => "PK".to_string(),
                    Attribute::ForeignKey => "FK".to_string(),
                    Attribute::Unique => "UQ".to_string(),
                    Attribute::Nullable => "NULL".to_string(),
                    Attribute::AutoIncrement => "AI".to_string(),
                    Attribute::Default(v) => format!("={}", match v {
                        DefaultValue::Now => "NOW".to_string(),
                        DefaultValue::True => "TRUE".to_string(),
                        DefaultValue::False => "FALSE".to_string(),
                        DefaultValue::Null => "NULL".to_string(),
                        DefaultValue::String(s) => {
                            if s.len() > 8 {
                                format!("{}...", &s[..8])
                            } else {
                                s.clone()
                            }
                        },
                        DefaultValue::Number(n) => n.to_string(),
                    }),
                })
                .collect();
            
            if !attrs.is_empty() {
                let attr_text = attrs.join(", ");
                svg.push_str(&format!(
                    r##"    <text x="{}" y="{}" font-size="{}" fill="#666" font-style="italic">[{}]</text>"##,
                    table_x + padding,
                    y + self.config.font_size + 12,
                    self.config.font_size - 2,
                    escape_xml(&attr_text)
                ));
                svg.push('\n');
            }
        }
        
        svg
    }
    
    fn generate_relationships(&self, layouts: &[TableLayout]) -> String {
        let mut svg = String::new();
        
        // Create a map of table names to layouts
        let layout_map: HashMap<_, _> = self.schema.tables.iter()
            .zip(layouts.iter())
            .map(|(table, layout)| (table.name.as_str(), (layout, table)))
            .collect();
        
        // Group relationships by source table and field to calculate offsets
        let mut field_relationship_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut field_relationship_indices: HashMap<usize, usize> = HashMap::new();
        
        // First pass: count relationships per source field
        for rel in &self.schema.relationships {
            let key = (rel.from_table.clone(), rel.from_field.clone());
            *field_relationship_counts.entry(key).or_insert(0) += 1;
        }
        
        // Second pass: assign indices
        let mut current_indices: HashMap<(String, String), usize> = HashMap::new();
        for (idx, rel) in self.schema.relationships.iter().enumerate() {
            let key = (rel.from_table.clone(), rel.from_field.clone());
            let index = *current_indices.get(&key).unwrap_or(&0);
            field_relationship_indices.insert(idx, index);
            current_indices.insert(key.clone(), index + 1);
        }
        
        svg.push_str("  <g class=\"relationships\">\n");
        
        for (idx, rel) in self.schema.relationships.iter().enumerate() {
            if let (Some((from_layout, from_table)), Some((to_layout, to_table))) = 
                (layout_map.get(rel.from_table.as_str()), layout_map.get(rel.to_table.as_str())) {
                
                let key = (rel.from_table.clone(), rel.from_field.clone());
                let total_count = *field_relationship_counts.get(&key).unwrap_or(&1);
                let line_index = *field_relationship_indices.get(&idx).unwrap_or(&0);
                
                svg.push_str(&self.generate_field_relationship_with_offset(
                    rel, from_layout, from_table, to_layout, to_table, line_index, total_count
                ));
            }
        }
        
        svg.push_str("  </g>\n");
        svg
    }
    
    fn generate_field_relationship_with_offset(&self, rel: &Relationship, from_layout: &TableLayout, from_table: &Table,
                                    to_layout: &TableLayout, to_table: &Table, line_index: usize, total_lines: usize) -> String {
        // Find the Y position of the specific fields
        let base_from_y = self.get_field_y_position(from_table, &rel.from_field, from_layout);
        let base_to_y = self.get_field_y_position(to_table, &rel.to_field, to_layout);
        
        // Calculate offset for multiple lines from the same field
        let line_spacing = 8; // Pixels between parallel lines
        let total_offset = (total_lines - 1) as i32 * line_spacing;
        let start_offset = -total_offset / 2;
        let offset_value = start_offset + (line_index as i32 * line_spacing);
        
        // Apply Y offset for horizontal line segments
        let from_field_y = (base_from_y as i32 + offset_value) as usize;
        let to_field_y = base_to_y;
        
        // Determine connection sides based on table positions
        let from_center_x = from_layout.x + from_layout.width / 2;
        let to_center_x = to_layout.x + to_layout.width / 2;
        
        let (start_x, start_y, end_x, end_y, from_side) = if from_center_x < to_center_x {
            // Connect right to left
            (from_layout.x + from_layout.width, from_field_y, to_layout.x, to_field_y, ConnectionSide::Right)
        } else {
            // Connect left to right
            (from_layout.x, from_field_y, to_layout.x + to_layout.width, to_field_y, ConnectionSide::Left)
        };
        
        // Generate path with horizontal offset for vertical segments
        let path = self.create_field_routed_path_with_offset(start_x, start_y, end_x, end_y, from_side, offset_value);
        
        // Determine arrow type and style based on relationship type
        match rel.relationship_type {
            RelationshipType::OneToOne => {
                // Dotted line with NO arrow for one-to-one
                format!(
                    r#"    <path d="{}" stroke="{}" stroke-width="2" fill="none" style="stroke-dasharray: 2,2"/>"#,
                    path, self.config.relationship_color
                ) + "\n"
            }
            RelationshipType::ManyToMany => {
                // Dashed line with arrow for many-to-many
                format!(
                    r#"    <path d="{}" stroke="{}" stroke-width="2" fill="none" style="stroke-dasharray: 5,5" marker-end="url(#arrowhead)"/>"#,
                    path, self.config.relationship_color
                ) + "\n"
            }
            _ => {
                // Solid line with arrow for one-to-many and many-to-one
                format!(
                    r#"    <path d="{}" stroke="{}" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>"#,
                    path, self.config.relationship_color
                ) + "\n"
            }
        }
    }
    
    fn generate_field_relationship(&self, rel: &Relationship, from_layout: &TableLayout, from_table: &Table,
                                    to_layout: &TableLayout, to_table: &Table) -> String {
        // Find the Y position of the specific fields
        let from_field_y = self.get_field_y_position(from_table, &rel.from_field, from_layout);
        let to_field_y = self.get_field_y_position(to_table, &rel.to_field, to_layout);
        
        // Determine connection sides based on table positions
        let from_center_x = from_layout.x + from_layout.width / 2;
        let to_center_x = to_layout.x + to_layout.width / 2;
        
        let (start_x, start_y, end_x, end_y, from_side) = if from_center_x < to_center_x {
            // Connect right to left
            (from_layout.x + from_layout.width, from_field_y, to_layout.x, to_field_y, ConnectionSide::Right)
        } else {
            // Connect left to right
            (from_layout.x, from_field_y, to_layout.x + to_layout.width, to_field_y, ConnectionSide::Left)
        };
        
        // Generate path
        let path = self.create_field_routed_path(start_x, start_y, end_x, end_y, from_side);
        
        // Determine arrow type and style based on relationship type
        let (stroke_style, marker_end) = match rel.relationship_type {
            RelationshipType::OneToMany => ("stroke-dasharray: none", "url(#arrowhead)"),
            RelationshipType::ManyToOne => ("stroke-dasharray: none", "url(#arrowhead)"),
            RelationshipType::ManyToMany => ("stroke-dasharray: 5,5", "url(#arrowhead)"),
            RelationshipType::OneToOne => ("stroke-dasharray: 2,2", "url(#diamond)"), // Dotted for one-to-one
        };
        
        format!(
            r#"    <path d="{}" stroke="{}" stroke-width="2" fill="none" style="{}" marker-end="{}"/>"#,
            path, self.config.relationship_color, stroke_style, marker_end
        ) + "\n"
    }
    
    fn get_field_y_position(&self, table: &Table, field_name: &str, layout: &TableLayout) -> usize {
        let header_height = self.config.row_height + 10;
        let start_y = layout.y + header_height + self.config.table_padding;
        let row_height = self.config.row_height + 10; // Match the increased row height
        
        // Find the field index
        for (i, column) in table.columns.iter().enumerate() {
            if column.name == field_name {
                // Return the center of the field row (accounting for name + attributes)
                return start_y + i * row_height + 10;
            }
        }
        
        // Fallback to middle of table if field not found
        layout.y + layout.height / 2
    }
    
    fn create_field_routed_path_with_offset(&self, start_x: usize, start_y: usize, end_x: usize, end_y: usize,
                                from_side: ConnectionSide, x_offset: i32) -> String {
        let base_offset = 60; // Larger offset to stay well clear of tables
        
        // Calculate actual offsets, handling negative values properly
        let actual_offset = base_offset as i32 + x_offset;
        
        // Route paths to stay in the gaps between tables
        match from_side {
            ConnectionSide::Right => {
                if (start_y as i32 - end_y as i32).abs() < 10 {
                    // Nearly same level - direct line (tables are in same row)
                    format!("M {} {} L {} {}", start_x, start_y, end_x, end_y)
                } else {
                    // Different levels - use midpoint routing to avoid parallel lines
                    // Each line gets its own horizontal position in the gap
                    let exit_x = (start_x as i32 + actual_offset).max(start_x as i32 + 10) as usize;
                    let enter_x = if end_x as i32 - actual_offset > 10 {
                        (end_x as i32 - actual_offset) as usize
                    } else {
                        end_x.saturating_sub(10)
                    };
                    
                    // Calculate midpoint with offset applied
                    let mid_x = (exit_x + enter_x) / 2;
                    
                    // Create path: horizontal -> vertical -> horizontal -> vertical -> horizontal
                    // This ensures lines cross at perpendicular angles
                    format!(
                        "M {} {} L {} {} L {} {} L {} {} L {} {} L {} {}",
                        start_x, start_y,
                        exit_x, start_y,
                        exit_x, (start_y + end_y) / 2,
                        mid_x, (start_y + end_y) / 2,
                        mid_x, end_y,
                        end_x, end_y
                    )
                }
            }
            ConnectionSide::Left => {
                if (start_y as i32 - end_y as i32).abs() < 10 {
                    format!("M {} {} L {} {}", start_x, start_y, end_x, end_y)
                } else {
                    // Different levels - use midpoint routing
                    let exit_x = if start_x as i32 - actual_offset > 10 {
                        (start_x as i32 - actual_offset) as usize
                    } else {
                        start_x.saturating_sub(10)
                    };
                    let enter_x = (end_x as i32 + actual_offset).max(end_x as i32 + 10) as usize;
                    
                    let mid_x = (exit_x + enter_x) / 2;
                    
                    format!(
                        "M {} {} L {} {} L {} {} L {} {} L {} {} L {} {}",
                        start_x, start_y,
                        exit_x, start_y,
                        exit_x, (start_y + end_y) / 2,
                        mid_x, (start_y + end_y) / 2,
                        mid_x, end_y,
                        end_x, end_y
                    )
                }
            }
            _ => format!("M {} {} L {} {}", start_x, start_y, end_x, end_y)
        }
    }
    
    fn create_field_routed_path(&self, start_x: usize, start_y: usize, end_x: usize, end_y: usize,
                                from_side: ConnectionSide) -> String {
        let offset = 60; // Larger offset to stay well clear of tables
        
        // Route paths to stay in the gaps between tables
        match from_side {
            ConnectionSide::Right => {
                if (start_y as i32 - end_y as i32).abs() < 10 {
                    // Nearly same level - direct line (tables are in same row)
                    format!("M {} {} L {} {}", start_x, start_y, end_x, end_y)
                } else {
                    // Different levels - route around with generous clearance
                    // Go out, down/up in the gap, then in
                    format!(
                        "M {} {} L {} {} L {} {} L {} {} L {} {}",
                        start_x, start_y,
                        start_x + offset, start_y,
                        start_x + offset, end_y,
                        end_x - offset, end_y,
                        end_x, end_y
                    )
                }
            }
            ConnectionSide::Left => {
                if (start_y as i32 - end_y as i32).abs() < 10 {
                    format!("M {} {} L {} {}", start_x, start_y, end_x, end_y)
                } else {
                    // Route with clearance on the left side
                    format!(
                        "M {} {} L {} {} L {} {} L {} {} L {} {}",
                        start_x, start_y,
                        start_x - offset, start_y,
                        start_x - offset, end_y,
                        end_x + offset, end_y,
                        end_x, end_y
                    )
                }
            }
            _ => format!("M {} {} L {} {}", start_x, start_y, end_x, end_y)
        }
    }
    
    
    pub fn generate_with_defs(&self) -> String {
        let mut svg = self.generate();
        
        // Insert arrow marker definitions after the opening svg tag
        let defs = r##"
  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#666666"/>
    </marker>
    <marker id="diamond" markerWidth="10" markerHeight="10" refX="5" refY="5" orient="auto">
      <polygon points="0 5, 5 0, 10 5, 5 10" fill="#666666"/>
    </marker>
  </defs>
"##;
        
        // Find position to insert (after first line)
        if let Some(pos) = svg.find('\n') {
            svg.insert_str(pos + 1, defs);
        }
        
        svg
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_svg_generation() {
        let mut schema = Schema::new();
        schema.title = Some("Test Schema".to_string());
        
        let mut table = Table::new("Users".to_string());
        table.columns.push(Column::new("id".to_string(), DataType::Int));
        schema.tables.push(table);
        
        let generator = SvgGenerator::new(schema);
        let svg = generator.generate_with_defs();
        
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Users"));
        assert!(svg.contains("</svg>"));
    }
}
