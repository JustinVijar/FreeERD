#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::ast::{Schema, Table, Column, DataType, Attribute, Relationship, RelationshipType};

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
        products.columns.push(Column::new("description".to_string(), DataType::String));
        products.columns.push(Column::new("category".to_string(), DataType::String));

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

        schema.relationships.push(Relationship {
            from_table: "Products".to_string(),
            from_field: "id".to_string(),
            to_table: "Orders".to_string(),
            to_field: "product_id".to_string(),
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
                    assert!(
                        !tables_overlap_with_margin(&table1.bounds, &table2.bounds, 20.0),
                        "Tables '{}' and '{}' are overlapping! Table1: ({}, {}, {}, {}), Table2: ({}, {}, {}, {})",
                        table1.table.name,
                        table2.table.name,
                        table1.bounds.x,
                        table1.bounds.y,
                        table1.bounds.width,
                        table1.bounds.height,
                        table2.bounds.x,
                        table2.bounds.y,
                        table2.bounds.width,
                        table2.bounds.height
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

        // Test each relationship line
        for relationship in &schema.relationships {
            let source_layout = layout_engine.find_table_layout(&relationship.from_table).unwrap();
            let target_layout = layout_engine.find_table_layout(&relationship.to_table).unwrap();

            let start_point = source_layout.bounds.right_center();
            let end_point = target_layout.bounds.left_center();

            let path = layout_engine.get_path_around_tables(start_point, end_point);

            // Check that the path doesn't intersect any table
            for table_layout in &layout_engine.table_layouts {
                for i in 0..path.len() - 1 {
                    assert!(
                        !line_intersects_rectangle_with_margin(
                            path[i], 
                            path[i + 1], 
                            &table_layout.bounds, 
                            10.0
                        ),
                        "Relationship line from {} to {} intersects table {}! Path segment: ({}, {}) -> ({}, {}), Table: ({}, {}, {}, {})",
                        relationship.from_table,
                        relationship.to_table,
                        table_layout.table.name,
                        path[i].x,
                        path[i].y,
                        path[i + 1].x,
                        path[i + 1].y,
                        table_layout.bounds.x,
                        table_layout.bounds.y,
                        table_layout.bounds.width,
                        table_layout.bounds.height
                    );
                }
            }
        }
    }

    #[test]
    fn test_minimum_table_spacing() {
        let schema = create_test_schema();
        let mut layout_engine = LayoutEngine::new();
        layout_engine.layout_tables(&schema.tables);

        let min_spacing = 60.0;

        for (i, table1) in layout_engine.table_layouts.iter().enumerate() {
            for (j, table2) in layout_engine.table_layouts.iter().enumerate() {
                if i != j {
                    let distance = calculate_min_distance(&table1.bounds, &table2.bounds);
                    assert!(
                        distance >= min_spacing,
                        "Tables '{}' and '{}' are too close! Distance: {}, Required: {}",
                        table1.table.name,
                        table2.table.name,
                        distance,
                        min_spacing
                    );
                }
            }
        }
    }

    // Helper functions for testing
    fn tables_overlap_with_margin(rect1: &Rectangle, rect2: &Rectangle, margin: f64) -> bool {
        !(rect1.x + rect1.width + margin < rect2.x || 
          rect2.x + rect2.width + margin < rect1.x ||
          rect1.y + rect1.height + margin < rect2.y ||
          rect2.y + rect2.height + margin < rect1.y)
    }

    fn line_intersects_rectangle_with_margin(start: Point, end: Point, rect: &Rectangle, margin: f64) -> bool {
        let expanded_rect = Rectangle::new(
            rect.x - margin,
            rect.y - margin,
            rect.width + (margin * 2.0),
            rect.height + (margin * 2.0)
        );

        // Check if line segment intersects the expanded rectangle
        line_segment_intersects_rectangle(start, end, &expanded_rect)
    }

    fn line_segment_intersects_rectangle(start: Point, end: Point, rect: &Rectangle) -> bool {
        // Check if either endpoint is inside the rectangle
        if point_in_rectangle(start, rect) || point_in_rectangle(end, rect) {
            return true;
        }

        // Check if line intersects any of the rectangle's edges
        let rect_corners = [
            Point::new(rect.x, rect.y),
            Point::new(rect.x + rect.width, rect.y),
            Point::new(rect.x + rect.width, rect.y + rect.height),
            Point::new(rect.x, rect.y + rect.height),
        ];

        for i in 0..4 {
            let edge_start = rect_corners[i];
            let edge_end = rect_corners[(i + 1) % 4];
            
            if line_segments_intersect(start, end, edge_start, edge_end) {
                return true;
            }
        }

        false
    }

    fn point_in_rectangle(point: Point, rect: &Rectangle) -> bool {
        point.x >= rect.x && point.x <= rect.x + rect.width &&
        point.y >= rect.y && point.y <= rect.y + rect.height
    }

    fn line_segments_intersect(p1: Point, q1: Point, p2: Point, q2: Point) -> bool {
        fn orientation(p: Point, q: Point, r: Point) -> i32 {
            let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
            if val.abs() < f64::EPSILON { 0 } // collinear
            else if val > 0.0 { 1 } // clockwise
            else { 2 } // counterclockwise
        }

        fn on_segment(p: Point, q: Point, r: Point) -> bool {
            q.x <= f64::max(p.x, r.x) && q.x >= f64::min(p.x, r.x) &&
            q.y <= f64::max(p.y, r.y) && q.y >= f64::min(p.y, r.y)
        }

        let o1 = orientation(p1, q1, p2);
        let o2 = orientation(p1, q1, q2);
        let o3 = orientation(p2, q2, p1);
        let o4 = orientation(p2, q2, q1);

        // General case
        if o1 != o3 && o2 != o4 {
            return true;
        }

        // Special cases for collinear points
        (o1 == 0 && on_segment(p1, p2, q1)) ||
        (o2 == 0 && on_segment(p1, q2, q1)) ||
        (o3 == 0 && on_segment(p2, p1, q2)) ||
        (o4 == 0 && on_segment(p2, q1, q2))
    }

    fn calculate_min_distance(rect1: &Rectangle, rect2: &Rectangle) -> f64 {
        let dx = f64::max(0.0, f64::max(rect1.x - (rect2.x + rect2.width), rect2.x - (rect1.x + rect1.width)));
        let dy = f64::max(0.0, f64::max(rect1.y - (rect2.y + rect2.height), rect2.y - (rect1.y + rect1.height)));
        (dx * dx + dy * dy).sqrt()
    }
}
