use crate::ast::{Schema, Relationship, RelationshipType};
use super::layout::{LayoutEngine, Point, ConnectionSide};
use super::renderer::SvgRenderer;

pub struct SvgGenerator {
    schema: Schema,
    layout_engine: LayoutEngine,
}

impl SvgGenerator {
    pub fn new(schema: Schema) -> Self {
        SvgGenerator {
            schema,
            layout_engine: LayoutEngine::new(),
        }
    }

    pub fn generate_with_defs(&self) -> String {
        let mut generator = self.clone();
        generator.generate_internal()
    }

    fn generate_internal(&mut self) -> String {
        // Layout tables using Fruchterman-Reingold with relationships
        self.layout_engine.layout_tables_with_relationships(&self.schema.tables, &self.schema.relationships);

        let mut renderer = SvgRenderer::new();
        
        // Start SVG with calculated dimensions
        let width = self.layout_engine.canvas_width;
        let height = self.layout_engine.canvas_height;
        
        renderer.start_svg(width, height, self.schema.title.as_deref());
        
        // Pass table layouts to renderer for accurate collision detection
        let table_rectangles: Vec<super::layout::Rectangle> = self.layout_engine.table_layouts
            .iter()
            .map(|layout| layout.bounds.clone())
            .collect();
        renderer.set_table_layouts(table_rectangles);
        
        // Add background
        renderer.add_background(width, height);

        // Render relationships in two phases to ensure proper layering
        // Phase 1: Create relationship boxes first (to get exact connection points)
        self.create_relationship_boxes(&mut renderer);
        
        // Phase 2: Render ALL line segments using exact box connection points
        self.render_relationship_lines_with_boxes(&mut renderer);
        
        // Phase 3: Render ALL relationship boxes (on top of lines, but behind tables)
        renderer.render_relationship_boxes();

        // Render tables SECOND (so they appear on top of lines)
        for layout in &self.layout_engine.table_layouts {
            if let Some(table) = self.schema.tables.iter().find(|t| t.name == layout.table_name) {
                renderer.render_table(layout, table);
            }
        }

        // End SVG
        renderer.end_svg();

        renderer.get_content().to_string()
    }

    fn create_relationship_boxes(&self, renderer: &mut SvgRenderer) {
        let mut existing_boxes: Vec<super::layout::Rectangle> = Vec::new();
        
        for relationship in &self.schema.relationships {
            self.create_single_relationship_box(renderer, relationship, &existing_boxes);
            
            // Add the newly created box to existing boxes for collision detection
            if let Some(last_box) = renderer.relationship_boxes.last() {
                existing_boxes.push(last_box.bounds.clone());
            }
        }
    }
    
    fn create_single_relationship_box(&self, renderer: &mut SvgRenderer, relationship: &Relationship, existing_boxes: &[super::layout::Rectangle]) {
        // Find source and target table layouts
        let source_layout = match self.layout_engine.find_table_layout(&relationship.from_table) {
            Some(layout) => layout,
            None => return,
        };

        let target_layout = match self.layout_engine.find_table_layout(&relationship.to_table) {
            Some(layout) => layout,
            None => return,
        };

        // Calculate connection points
        let (start_point, end_point) = self.calculate_connection_points(
            source_layout, target_layout, &relationship.from_field, &relationship.to_field
        );

        let is_self_referencing = relationship.from_table == relationship.to_table;
        
        let path = if is_self_referencing {
            // Use curved path for self-referencing relationships
            self.layout_engine.get_self_referencing_curve(&source_layout.bounds)
        } else {
            // Use simple straight line
            vec![start_point, end_point]
        };

        // Create relationship text
        let operator = match relationship.relationship_type {
            RelationshipType::OneToOne => "-",
            RelationshipType::OneToMany => ">",
            RelationshipType::ManyToOne => "<",
            RelationshipType::ManyToMany => "<>",
        };
        let relationship_text = format!("{}.{} {} {}.{}", 
            relationship.from_table, relationship.from_field, operator, 
            relationship.to_table, relationship.to_field);

        // Find optimal position with proper collision detection
        let box_position = self.find_optimal_position_with_collision_detection(&path, &relationship_text, &renderer.table_layouts, existing_boxes);
        
        // Create relationship box layout
        let box_id = format!("{}_{}_{}_{}", 
            relationship.from_table, relationship.from_field,
            relationship.to_table, relationship.to_field);
        
        let box_layout = super::layout::RelationshipBoxLayout::new(
            box_id,
            relationship_text,
            box_position,
            start_point,
            end_point,
            existing_boxes,
        );
        
        // Add to renderer
        renderer.add_relationship_box(box_layout);
    }
    
    fn find_optimal_position_with_collision_detection(&self, path: &[Point], relationship_text: &str, table_layouts: &[super::layout::Rectangle], existing_boxes: &[super::layout::Rectangle]) -> Point {
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
        
        // Try multiple positions along the line
        let candidate_ratios = vec![0.5, 0.4, 0.6, 0.3, 0.7, 0.25, 0.75, 0.2, 0.8];
        
        for &ratio in &candidate_ratios {
            let candidate = Point::new(
                start.x + (end.x - start.x) * ratio,
                start.y + (end.y - start.y) * ratio
            );
            
            if self.is_position_safe_from_all_obstacles(candidate, box_width, box_height, table_layouts, existing_boxes) {
                return candidate;
            }
        }
        
        // Try perpendicular offsets if no position along line works
        let midpoint = Point::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance > 0.0 {
            let offsets = vec![60.0, 80.0, 100.0, 120.0, 150.0];
            
            for &offset_distance in &offsets {
                let perpendicular_x = -dy / distance * offset_distance;
                let perpendicular_y = dx / distance * offset_distance;
                
                let candidates = vec![
                    Point::new(midpoint.x + perpendicular_x, midpoint.y + perpendicular_y),
                    Point::new(midpoint.x - perpendicular_x, midpoint.y - perpendicular_y),
                ];
                
                for candidate in candidates {
                    if self.is_position_safe_from_all_obstacles(candidate, box_width, box_height, table_layouts, existing_boxes) {
                        return candidate;
                    }
                }
            }
        }
        
        // Fallback to midpoint with vertical offset
        Point::new(midpoint.x, midpoint.y - 100.0)
    }
    
    fn is_position_safe_from_all_obstacles(&self, position: Point, box_width: f64, box_height: f64, table_layouts: &[super::layout::Rectangle], existing_boxes: &[super::layout::Rectangle]) -> bool {
        let box_left = position.x - box_width / 2.0;
        let box_right = position.x + box_width / 2.0;
        let box_top = position.y - box_height / 2.0;
        let box_bottom = position.y + box_height / 2.0;
        
        let safe_margin = 50.0;
        let box_spacing = 40.0;
        
        // Check collision with tables
        for table_rect in table_layouts {
            if !(box_right + safe_margin < table_rect.x || 
                 box_left - safe_margin > table_rect.x + table_rect.width ||
                 box_bottom + safe_margin < table_rect.y ||
                 box_top - safe_margin > table_rect.y + table_rect.height) {
                return false; // Collision with table
            }
        }
        
        // Check collision with existing relationship boxes
        for existing_box in existing_boxes {
            if !(box_right + box_spacing < existing_box.x || 
                 box_left - box_spacing > existing_box.x + existing_box.width ||
                 box_bottom + box_spacing < existing_box.y ||
                 box_top - box_spacing > existing_box.y + existing_box.height) {
                return false; // Collision with existing box
            }
        }
        
        // Check canvas bounds
        if position.x < 50.0 || position.x > 2550.0 || position.y < 50.0 || position.y > 1850.0 {
            return false;
        }
        
        true
    }
    
    fn render_relationship_lines_with_boxes(&self, renderer: &mut SvgRenderer) {
        for relationship in &self.schema.relationships {
            self.render_single_relationship_line_with_box(renderer, relationship);
        }
    }
    
    fn render_single_relationship_line_with_box(&self, renderer: &mut SvgRenderer, relationship: &Relationship) {
        // Find source and target table layouts
        let source_layout = match self.layout_engine.find_table_layout(&relationship.from_table) {
            Some(layout) => layout,
            None => return,
        };

        let target_layout = match self.layout_engine.find_table_layout(&relationship.to_table) {
            Some(layout) => layout,
            None => return,
        };

        // Calculate connection points
        let (start_point, end_point) = self.calculate_connection_points(
            source_layout, target_layout, &relationship.from_field, &relationship.to_field
        );

        // Find the corresponding relationship box
        let box_id = format!("{}_{}_{}_{}", 
            relationship.from_table, relationship.from_field,
            relationship.to_table, relationship.to_field);
        
        if let Some(box_layout) = renderer.relationship_boxes.iter().find(|b| b.id == box_id) {
            let (source_dot, target_dot) = box_layout.get_dot_positions();
            
            // Render line segments with EXACT connection to dots WITHOUT markers
            // (markers are drawn at the box dots instead)
            renderer.render_line_segment(start_point, source_dot, relationship.relationship_type, "none");
            renderer.render_line_segment(target_dot, end_point, relationship.relationship_type, "none");
        }
    }

    fn calculate_connection_points(
        &self,
        source_layout: &super::layout::TableLayout,
        target_layout: &super::layout::TableLayout,
        from_field: &str,
        to_field: &str,
    ) -> (Point, Point) {
        let source_bounds = &source_layout.bounds;
        let target_bounds = &target_layout.bounds;

        // Determine best connection sides based on relative positions
        let source_center = source_bounds.center();
        let target_center = target_bounds.center();

        let dx = target_center.x - source_center.x;
        let dy = target_center.y - source_center.y;

        let (source_side, target_side) = if dx.abs() > dy.abs() {
            // Horizontal connection preferred
            if dx > 0.0 {
                (ConnectionSide::Right, ConnectionSide::Left)
            } else {
                (ConnectionSide::Left, ConnectionSide::Right)
            }
        } else {
            // Vertical connection preferred
            if dy > 0.0 {
                (ConnectionSide::Bottom, ConnectionSide::Top)
            } else {
                (ConnectionSide::Top, ConnectionSide::Bottom)
            }
        };

        // Calculate distributed connection points to avoid overlap
        // Include both source and target info to ensure unique positioning for each relationship
        let source_key = format!("{}_to_{}", 
            source_layout.table_name, target_layout.table_name);
        let target_key = format!("{}_{}_from_{}_{}", 
            to_field, target_layout.table_name, source_layout.table_name, from_field);
        
        let start_point = self.get_distributed_connection_point(source_bounds, source_side, &source_key);
        let end_point = self.get_distributed_connection_point(target_bounds, target_side, &target_key);

        (start_point, end_point)
    }
    
    fn get_distributed_connection_point(&self, bounds: &super::layout::Rectangle, side: ConnectionSide, field_name: &str) -> Point {
        // Use field name hash to create consistent but distributed connection points
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        field_name.hash(&mut hasher);
        let hash_value = hasher.finish();
        
        // Create offset based on hash (0.1 to 0.9 of the side length for better distribution)
        let offset_ratio = 0.1 + (hash_value % 80) as f64 / 100.0; // 0.1 to 0.9
        
        match side {
            ConnectionSide::Left => Point::new(
                bounds.x,
                bounds.y + bounds.height * offset_ratio
            ),
            ConnectionSide::Right => Point::new(
                bounds.x + bounds.width,
                bounds.y + bounds.height * offset_ratio
            ),
            ConnectionSide::Top => Point::new(
                bounds.x + bounds.width * offset_ratio,
                bounds.y
            ),
            ConnectionSide::Bottom => Point::new(
                bounds.x + bounds.width * offset_ratio,
                bounds.y + bounds.height
            ),
        }
    }


}

impl Clone for SvgGenerator {
    fn clone(&self) -> Self {
        SvgGenerator {
            schema: self.schema.clone(),
            layout_engine: LayoutEngine::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Table, Column, DataType, Attribute, RelationshipType};

    #[test]
    fn test_svg_generation() {
        let mut schema = Schema::new();
        schema.title = Some("Test Schema".to_string());

        // Create a simple table
        let mut table = Table::new("Users".to_string());
        let mut id_column = Column::new("id".to_string(), DataType::Int);
        id_column.attributes.push(Attribute::PrimaryKey);
        table.columns.push(id_column);

        let name_column = Column::new("name".to_string(), DataType::String);
        table.columns.push(name_column);

        schema.tables.push(table);

        let generator = SvgGenerator::new(schema);
        let svg_content = generator.generate();

        // Basic checks
        assert!(svg_content.contains("<svg"));
        assert!(svg_content.contains("</svg>"));
        assert!(svg_content.contains("Users"));
        assert!(svg_content.contains("Test Schema"));
    }

    #[test]
    fn test_relationship_rendering() {
        let mut schema = Schema::new();

        // Create two tables
        let mut users_table = Table::new("Users".to_string());
        let mut id_column = Column::new("id".to_string(), DataType::Int);
        id_column.attributes.push(Attribute::PrimaryKey);
        users_table.columns.push(id_column);
        schema.tables.push(users_table);

        let mut orders_table = Table::new("Orders".to_string());
        let mut order_id_column = Column::new("id".to_string(), DataType::Int);
        order_id_column.attributes.push(Attribute::PrimaryKey);
        orders_table.columns.push(order_id_column);

        let mut user_id_column = Column::new("user_id".to_string(), DataType::Int);
        user_id_column.attributes.push(Attribute::ForeignKey);
        orders_table.columns.push(user_id_column);
        schema.tables.push(orders_table);

        // Add relationship
        let relationship = Relationship {
            from_table: "Users".to_string(),
            from_field: "id".to_string(),
            to_table: "Orders".to_string(),
            to_field: "user_id".to_string(),
            relationship_type: RelationshipType::OneToMany,
        };
        schema.relationships.push(relationship);

        let generator = SvgGenerator::new(schema);
        let svg_content = generator.generate();

        // Check that relationship line is rendered
        assert!(svg_content.contains("relationship-line"));
        assert!(svg_content.contains("marker-end"));
    }
}