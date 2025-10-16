use crate::ast::{Table, Column, Relationship};
use super::force_layout::FruchtermanReingoldLayout;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rectangle {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rectangle { x, y, width, height }
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.x + self.width / 2.0,
            self.y + self.height / 2.0
        )
    }
}

#[derive(Debug, Clone)]
pub struct TableLayout {
    pub table_name: String,
    pub bounds: Rectangle,
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct RelationshipBoxLayout {
    pub id: String,
    pub text: String,
    pub bounds: Rectangle,
    pub source_connection: Point,
    pub target_connection: Point,
    pub source_side: ConnectionSide,
    pub target_side: ConnectionSide,
}

impl TableLayout {
    pub fn new(table: Table, x: f64, y: f64) -> Self {
        let field_height = 28.0; // Increased for better text readability
        let header_height = 40.0; // Increased for better table name display
        let padding = 10.0;
        
        // Calculate table dimensions
        let max_field_width = table.columns.iter()
            .map(|col| Self::calculate_field_width(col))
            .fold(120.0, f64::max);
        
        let table_name_width = table.name.len() as f64 * 10.0 + 40.0; // More generous table name sizing
        let width = max_field_width.max(table_name_width).max(220.0); // Increased minimum width significantly
        let height = header_height + (table.columns.len() as f64 * field_height) + padding;
        
        let bounds = Rectangle::new(x, y, width, height);
        
        TableLayout {
            table_name: table.name.clone(),
            bounds,
        }
    }

    fn calculate_field_width(column: &Column) -> f64 {
        // Calculate field name width (8px per character - increased for safety)
        let name_width = column.name.len() as f64 * 8.0;
        
        // Calculate field type width (7px per character - increased for safety)
        let type_text = format!("{}", column.datatype);
        let type_width = type_text.len() as f64 * 7.0;
        
        // Calculate attributes width (6px per character + brackets - increased for safety)
        let attrs_width = if !column.attributes.is_empty() {
            let attrs_text = column.attributes.iter()
                .map(|attr| format!("{}", attr))
                .collect::<Vec<_>>()
                .join(", ");
            (attrs_text.len() + 2) as f64 * 6.0 // +2 for brackets []
        } else {
            0.0
        };
        
        // Total width: name + spacing + type + spacing + attrs + left/right padding
        let spacing_between_elements = if !column.attributes.is_empty() { 25.0 } else { 15.0 }; // Increased spacing
        let left_right_padding = 30.0; // Increased padding on each side
        
        let total_width = name_width + spacing_between_elements + type_width + attrs_width + left_right_padding;
        
        // Width calculation complete
        
        // Ensure minimum width for readability
        total_width.max(200.0) // Increased minimum width
    }

}

impl RelationshipBoxLayout {
    pub fn new(
        id: String,
        text: String,
        position: Point,
        source_table_pos: Point,
        target_table_pos: Point,
        _existing_boxes: &[Rectangle],
    ) -> Self {
        // Calculate box dimensions
        let text_width = text.len() as f64 * 6.5;
        let text_height = 16.0;
        let padding = 6.0;
        let box_width = text_width + padding * 2.0;
        let box_height = text_height + padding * 2.0;
        
        // Create bounds
        let bounds = Rectangle::new(
            position.x - box_width / 2.0,
            position.y - box_height / 2.0,
            box_width,
            box_height,
        );
        
        // Determine connection sides based on where each table is relative to the box
        // Calculate vectors from box center to each table
        let source_dx = source_table_pos.x - position.x;
        let source_dy = source_table_pos.y - position.y;
        let target_dx = target_table_pos.x - position.x;
        let target_dy = target_table_pos.y - position.y;
        
        // Determine which side of the box each table is closest to
        let source_side = Self::determine_connection_side(source_dx, source_dy);
        let target_side = Self::determine_connection_side(target_dx, target_dy);
        
        // Calculate exact connection points on the box edges
        let source_connection = Self::get_connection_point(&bounds, position, source_side);
        let target_connection = Self::get_connection_point(&bounds, position, target_side);
        
        RelationshipBoxLayout {
            id,
            text,
            bounds,
            source_connection,
            target_connection,
            source_side,
            target_side,
        }
    }
    
    fn determine_connection_side(dx: f64, dy: f64) -> ConnectionSide {
        // Determine which side based on the angle to the table
        // Use 45-degree sectors to determine the side
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();
        
        if abs_dx > abs_dy {
            // More horizontal than vertical
            if dx > 0.0 {
                ConnectionSide::Right
            } else {
                ConnectionSide::Left
            }
        } else {
            // More vertical than horizontal
            if dy > 0.0 {
                ConnectionSide::Bottom
            } else {
                ConnectionSide::Top
            }
        }
    }
    
    fn get_connection_point(bounds: &Rectangle, center: Point, side: ConnectionSide) -> Point {
        match side {
            ConnectionSide::Left => Point::new(bounds.x, center.y),
            ConnectionSide::Right => Point::new(bounds.x + bounds.width, center.y),
            ConnectionSide::Top => Point::new(center.x, bounds.y),
            ConnectionSide::Bottom => Point::new(center.x, bounds.y + bounds.height),
        }
    }
    
    pub fn get_dot_positions(&self) -> (Point, Point) {
        (self.source_connection, self.target_connection)
    }
    
}

pub struct LayoutEngine {
    pub table_layouts: Vec<TableLayout>,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

impl LayoutEngine {
    pub fn new() -> Self {
        LayoutEngine {
            table_layouts: Vec::new(),
            canvas_width: 800.0,
            canvas_height: 600.0,
        }
    }

    pub fn layout_tables_with_relationships(&mut self, tables: &[Table], relationships: &[Relationship]) {
        self.table_layouts.clear();
        
        if tables.is_empty() {
            return;
        }

        // Use Fruchterman-Reingold force-directed layout with auto-sizing
        let mut fr_layout = FruchtermanReingoldLayout::new(tables, relationships, 0.0, 0.0);
        
        // Run the force-directed algorithm
        println!("🔄 Running Fruchterman-Reingold layout algorithm...");
        fr_layout.run_layout();
        
        // Get the optimized table layouts
        self.table_layouts = fr_layout.get_table_layouts();
        
        // Update canvas size based on final positions
        let (width, height) = fr_layout.get_canvas_size();
        self.canvas_width = width;
        self.canvas_height = height;
        
        println!("✅ Force-directed layout complete. Canvas: {}x{}", width, height);
    }
    

    pub fn find_table_layout(&self, table_name: &str) -> Option<&TableLayout> {
        self.table_layouts.iter().find(|layout| layout.table_name == table_name)
    }

    pub fn get_self_referencing_curve(&self, table_bounds: &Rectangle) -> Vec<Point> {
        // Create a curved path for self-referencing relationships
        let margin = 30.0;
        let curve_size = 80.0;
        
        // Start from right side of table
        let start = Point::new(table_bounds.x + table_bounds.width, table_bounds.y + table_bounds.height / 2.0);
        
        // Create control points for a curved path that goes around the table
        let control1 = Point::new(start.x + curve_size, start.y);
        let control2 = Point::new(start.x + curve_size, table_bounds.y - margin);
        let control3 = Point::new(table_bounds.x + table_bounds.width / 2.0, table_bounds.y - margin);
        let end = Point::new(table_bounds.x + table_bounds.width / 2.0, table_bounds.y);
        
        vec![start, control1, control2, control3, end]
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Schema, Table, Column, DataType, Relationship, RelationshipType};

    fn create_test_schema() -> Schema {
        let mut schema = Schema::new();
        schema.title = Some("Test Schema".to_string());

        // Create multiple tables with different sizes
        let mut customers = Table::new("Customers".to_string());
        customers.columns.push(Column::new("id".to_string(), DataType::Int));
        customers.columns.push(Column::new("name".to_string(), DataType::String));
        customers.columns.push(Column::new("email".to_string(), DataType::String));
        
        let mut orders = Table::new("Orders".to_string());
        orders.columns.push(Column::new("id".to_string(), DataType::Int));
        orders.columns.push(Column::new("customer_id".to_string(), DataType::Int));
        orders.columns.push(Column::new("total".to_string(), DataType::Float));
        
        let mut products = Table::new("Products".to_string());
        products.columns.push(Column::new("id".to_string(), DataType::Int));
        products.columns.push(Column::new("name".to_string(), DataType::String));
        products.columns.push(Column::new("price".to_string(), DataType::Float));

        schema.tables.push(customers);
        schema.tables.push(orders);
        schema.tables.push(products);

        // Add relationships
        schema.relationships.push(Relationship {
            from_table: "Customers".to_string(),
            from_field: "id".to_string(),
            to_table: "Orders".to_string(),
            to_field: "customer_id".to_string(),
            relationship_type: RelationshipType::OneToMany,
        });

        schema
    }

    #[test]
    fn test_no_table_overlaps() {
        let schema = create_test_schema();
        let mut layout_engine = LayoutEngine::new();
        layout_engine.layout_tables(&schema.tables);

        // Check that no tables overlap
        for (i, table1) in layout_engine.table_layouts.iter().enumerate() {
            for (j, table2) in layout_engine.table_layouts.iter().enumerate() {
                if i != j {
                    let min_distance = calculate_min_distance(&table1.bounds, &table2.bounds);
                    assert!(
                        min_distance >= 60.0,
                        "Tables '{}' and '{}' are too close! Distance: {}, Required: 60.0",
                        table1.table.name,
                        table2.table.name,
                        min_distance
                    );
                }
            }
        }
    }

    #[test]
    fn test_lines_avoid_tables() {
        let schema = create_test_schema();
        let mut layout_engine = LayoutEngine::new();
        layout_engine.layout_tables(&schema.tables);

        // Test that relationship lines don't intersect tables
        for relationship in &schema.relationships {
            let source_layout = layout_engine.find_table_layout(&relationship.from_table).unwrap();
            let target_layout = layout_engine.find_table_layout(&relationship.to_table).unwrap();

            let start_point = source_layout.bounds.right_center();
            let end_point = target_layout.bounds.left_center();

            let path = layout_engine.get_safe_path_avoiding_tables(start_point, end_point, false);

            // Check that the path doesn't intersect any table
            for table_layout in &layout_engine.table_layouts {
                // Skip the source and target tables as lines can connect to them
                if table_layout.table.name == relationship.from_table || 
                   table_layout.table.name == relationship.to_table {
                    continue;
                }

                for i in 0..path.len() - 1 {
                    assert!(
                        !layout_engine.line_intersects_rectangle(path[i], path[i + 1], &table_layout.bounds),
                        "Relationship line from {} to {} intersects table {}! Path segment: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                        relationship.from_table,
                        relationship.to_table,
                        table_layout.table.name,
                        path[i].x,
                        path[i].y,
                        path[i + 1].x,
                        path[i + 1].y
                    );
                }
            }
        }
    }

    fn calculate_min_distance(rect1: &Rectangle, rect2: &Rectangle) -> f64 {
        let dx = f64::max(0.0, f64::max(rect1.x - (rect2.x + rect2.width), rect2.x - (rect1.x + rect1.width)));
        let dy = f64::max(0.0, f64::max(rect1.y - (rect2.y + rect2.height), rect2.y - (rect1.y + rect1.height)));
        (dx * dx + dy * dy).sqrt()
    }
}