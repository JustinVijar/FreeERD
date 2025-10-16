use crate::ast::{Table, Column, Relationship};
use super::force_layout::FruchtermanReingoldLayout;
use std::collections::HashMap;

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

    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rectangle) -> bool {
        !(self.x + self.width < other.x || 
          other.x + other.width < self.x ||
          self.y + self.height < other.y ||
          other.y + other.height < self.y)
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.x + self.width / 2.0,
            self.y + self.height / 2.0
        )
    }

    pub fn left_center(&self) -> Point {
        Point::new(self.x, self.y + self.height / 2.0)
    }

    pub fn right_center(&self) -> Point {
        Point::new(self.x + self.width, self.y + self.height / 2.0)
    }

    pub fn top_center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y)
    }

    pub fn bottom_center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height)
    }
}

#[derive(Debug, Clone)]
pub struct TableLayout {
    pub table_name: String,
    pub bounds: Rectangle,
    pub field_positions: HashMap<String, Point>,
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
        
        // Calculate field positions for connection points
        let mut field_positions = HashMap::new();
        for (i, column) in table.columns.iter().enumerate() {
            let field_y = y + header_height + (i as f64 * field_height) + (field_height / 2.0);
            field_positions.insert(
                column.name.clone(),
                Point::new(x + width, field_y) // Right side connection point
            );
        }
        
        TableLayout {
            table_name: table.name.clone(),
            bounds,
            field_positions,
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

    pub fn get_connection_point(&self, field_name: &str, side: ConnectionSide) -> Option<Point> {
        self.field_positions.get(field_name).map(|_| {
            match side {
                ConnectionSide::Left => self.bounds.left_center(),
                ConnectionSide::Right => self.bounds.right_center(),
                ConnectionSide::Top => self.bounds.top_center(),
                ConnectionSide::Bottom => self.bounds.bottom_center(),
            }
        })
    }
}

impl RelationshipBoxLayout {
    pub fn new(
        id: String,
        text: String,
        position: Point,
        source_table_pos: Point,
        target_table_pos: Point,
        existing_boxes: &[Rectangle],
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
    
    pub fn intersects_with_tables(&self, table_layouts: &[Rectangle]) -> bool {
        let safe_margin = 50.0;
        let expanded_bounds = Rectangle::new(
            self.bounds.x - safe_margin,
            self.bounds.y - safe_margin,
            self.bounds.width + safe_margin * 2.0,
            self.bounds.height + safe_margin * 2.0,
        );
        
        table_layouts.iter().any(|table| expanded_bounds.intersects(table))
    }
}

pub struct LayoutEngine {
    pub table_layouts: Vec<TableLayout>,
    pub relationship_box_layouts: Vec<RelationshipBoxLayout>,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

impl LayoutEngine {
    pub fn new() -> Self {
        LayoutEngine {
            table_layouts: Vec::new(),
            relationship_box_layouts: Vec::new(),
            canvas_width: 800.0,
            canvas_height: 600.0,
        }
    }

    pub fn layout_tables(&mut self, tables: &[Table]) {
        self.layout_tables_with_relationships(tables, &[]);
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
    
    fn tables_overlap(&self, rect1: &Rectangle, rect2: &Rectangle) -> bool {
        let margin = 80.0; // Increased minimum space between tables
        !(rect1.x + rect1.width + margin < rect2.x || 
          rect2.x + rect2.width + margin < rect1.x ||
          rect1.y + rect1.height + margin < rect2.y ||
          rect2.y + rect2.height + margin < rect1.y)
    }

    fn calculate_canvas_size(&mut self) {
        let margin = 100.0;
        
        if self.table_layouts.is_empty() {
            self.canvas_width = 800.0;
            self.canvas_height = 600.0;
            return;
        }

        let max_x = self.table_layouts.iter()
            .map(|layout| layout.bounds.x + layout.bounds.width)
            .fold(0.0, f64::max);
        
        let max_y = self.table_layouts.iter()
            .map(|layout| layout.bounds.y + layout.bounds.height)
            .fold(0.0, f64::max);

        self.canvas_width = (max_x + margin).max(800.0);
        self.canvas_height = (max_y + margin).max(600.0);
    }

    pub fn find_table_layout(&self, table_name: &str) -> Option<&TableLayout> {
        self.table_layouts.iter().find(|layout| layout.table_name == table_name)
    }

    pub fn get_path_around_tables(&self, start: Point, end: Point) -> Vec<Point> {
        self.get_safe_path_avoiding_tables(start, end, false)
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
    
    pub fn get_safe_path_avoiding_tables(&self, start: Point, end: Point, is_self_ref: bool) -> Vec<Point> {
        let safe_margin = 50.0; // Increased margin for better avoidance
        
        // If it's a self-referencing relationship, handle it specially
        if is_self_ref {
            // Find the table that contains both start and end points
            for layout in &self.table_layouts {
                let expanded = Rectangle::new(
                    layout.bounds.x - 10.0,
                    layout.bounds.y - 10.0,
                    layout.bounds.width + 20.0,
                    layout.bounds.height + 20.0
                );
                if self.point_in_rectangle(start, &expanded) || self.point_in_rectangle(end, &expanded) {
                    return self.get_self_referencing_curve(&layout.bounds);
                }
            }
        }
        
        // Check if direct line intersects any table
        let intersecting_tables: Vec<_> = self.table_layouts.iter()
            .filter(|layout| {
                let expanded_bounds = Rectangle::new(
                    layout.bounds.x - safe_margin,
                    layout.bounds.y - safe_margin,
                    layout.bounds.width + (safe_margin * 2.0),
                    layout.bounds.height + (safe_margin * 2.0)
                );
                self.line_intersects_rectangle(start, end, &expanded_bounds)
            })
            .collect();
        
        if intersecting_tables.is_empty() {
            // Direct line is clear, use it
            return vec![start, end];
        }
        
        // Try multiple avoidance strategies
        let strategies = [
            self.try_above_below_routing(start, end, &intersecting_tables, safe_margin),
            self.try_left_right_routing(start, end, &intersecting_tables, safe_margin),
            self.try_wide_detour_routing(start, end, &intersecting_tables, safe_margin),
        ];
        
        // Find the first strategy that doesn't intersect any tables
        for strategy in &strategies {
            if self.path_is_clear(strategy, safe_margin) {
                return strategy.clone();
            }
        }
        
        // If all strategies fail, use the shortest one
        strategies.into_iter()
            .min_by(|a, b| {
                let len_a = self.calculate_path_length(a);
                let len_b = self.calculate_path_length(b);
                len_a.partial_cmp(&len_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_else(|| vec![start, end])
    }
    
    fn try_above_below_routing(&self, start: Point, end: Point, obstacles: &[&TableLayout], margin: f64) -> Vec<Point> {
        if obstacles.is_empty() {
            return vec![start, end];
        }
        
        // Find the bounding box of all obstacles
        let min_y = obstacles.iter().map(|t| t.bounds.y).fold(f64::INFINITY, f64::min) - margin;
        let max_y = obstacles.iter().map(|t| t.bounds.y + t.bounds.height).fold(f64::NEG_INFINITY, f64::max) + margin;
        
        // Try routing above obstacles
        let above_y = min_y - 50.0;
        let above_path = vec![
            start,
            Point::new(start.x, above_y),
            Point::new(end.x, above_y),
            end
        ];
        
        // Try routing below obstacles
        let below_y = max_y + 50.0;
        let below_path = vec![
            start,
            Point::new(start.x, below_y),
            Point::new(end.x, below_y),
            end
        ];
        
        // Return the path that's closer to the original line
        let mid_y = (start.y + end.y) / 2.0;
        if (above_y - mid_y).abs() < (below_y - mid_y).abs() {
            above_path
        } else {
            below_path
        }
    }
    
    fn try_left_right_routing(&self, start: Point, end: Point, obstacles: &[&TableLayout], margin: f64) -> Vec<Point> {
        if obstacles.is_empty() {
            return vec![start, end];
        }
        
        // Find the bounding box of all obstacles
        let min_x = obstacles.iter().map(|t| t.bounds.x).fold(f64::INFINITY, f64::min) - margin;
        let max_x = obstacles.iter().map(|t| t.bounds.x + t.bounds.width).fold(f64::NEG_INFINITY, f64::max) + margin;
        
        // Try routing left of obstacles
        let left_x = min_x - 50.0;
        let left_path = vec![
            start,
            Point::new(left_x, start.y),
            Point::new(left_x, end.y),
            end
        ];
        
        // Try routing right of obstacles
        let right_x = max_x + 50.0;
        let right_path = vec![
            start,
            Point::new(right_x, start.y),
            Point::new(right_x, end.y),
            end
        ];
        
        // Return the path that's closer to the original line
        let mid_x = (start.x + end.x) / 2.0;
        if (left_x - mid_x).abs() < (right_x - mid_x).abs() {
            left_path
        } else {
            right_path
        }
    }
    
    fn try_wide_detour_routing(&self, start: Point, end: Point, obstacles: &[&TableLayout], margin: f64) -> Vec<Point> {
        if obstacles.is_empty() {
            return vec![start, end];
        }
        
        // Create a wide detour around all obstacles
        let min_x = obstacles.iter().map(|t| t.bounds.x).fold(f64::INFINITY, f64::min) - margin - 100.0;
        let max_x = obstacles.iter().map(|t| t.bounds.x + t.bounds.width).fold(f64::NEG_INFINITY, f64::max) + margin + 100.0;
        let min_y = obstacles.iter().map(|t| t.bounds.y).fold(f64::INFINITY, f64::min) - margin - 100.0;
        let max_y = obstacles.iter().map(|t| t.bounds.y + t.bounds.height).fold(f64::NEG_INFINITY, f64::max) + margin + 100.0;
        
        // Choose detour direction based on which side has more space
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        
        if dx.abs() > dy.abs() {
            // Horizontal movement - detour vertically
            if start.y < (min_y + max_y) / 2.0 {
                vec![start, Point::new(start.x, min_y), Point::new(end.x, min_y), end]
            } else {
                vec![start, Point::new(start.x, max_y), Point::new(end.x, max_y), end]
            }
        } else {
            // Vertical movement - detour horizontally
            if start.x < (min_x + max_x) / 2.0 {
                vec![start, Point::new(min_x, start.y), Point::new(min_x, end.y), end]
            } else {
                vec![start, Point::new(max_x, start.y), Point::new(max_x, end.y), end]
            }
        }
    }
    
    fn path_is_clear(&self, path: &[Point], margin: f64) -> bool {
        for i in 0..path.len() - 1 {
            for layout in &self.table_layouts {
                let expanded_bounds = Rectangle::new(
                    layout.bounds.x - margin,
                    layout.bounds.y - margin,
                    layout.bounds.width + (margin * 2.0),
                    layout.bounds.height + (margin * 2.0)
                );
                if self.line_intersects_rectangle(path[i], path[i + 1], &expanded_bounds) {
                    return false;
                }
            }
        }
        true
    }
    
    fn force_obstacle_avoidance_routing(&self, start: Point, end: Point, obstacles: &[Rectangle]) -> Vec<Point> {
        if obstacles.is_empty() {
            return vec![start, end];
        }
        
        let margin = 50.0;
        let min_x = obstacles.iter().map(|r| r.x - margin).fold(f64::INFINITY, f64::min);
        let max_x = obstacles.iter().map(|r| r.x + r.width + margin).fold(f64::NEG_INFINITY, f64::max);
        let min_y = obstacles.iter().map(|r| r.y - margin).fold(f64::INFINITY, f64::min);
        let max_y = obstacles.iter().map(|r| r.y + r.height + margin).fold(f64::NEG_INFINITY, f64::max);
        
        // Force routing around the obstacle bounding box
        let mut path = vec![start];
        
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        
        if dx.abs() > dy.abs() {
            // Horizontal routing - go above or below obstacles
            let route_y = if start.y < (min_y + max_y) / 2.0 { min_y } else { max_y };
            path.push(Point::new(start.x, route_y));
            path.push(Point::new(end.x, route_y));
        } else {
            // Vertical routing - go left or right of obstacles  
            let route_x = if start.x < (min_x + max_x) / 2.0 { min_x } else { max_x };
            path.push(Point::new(route_x, start.y));
            path.push(Point::new(route_x, end.y));
        }
        
        path.push(end);
        path
    }
    
    fn try_horizontal_first_routing(&self, start: Point, end: Point, _obstacles: &[Rectangle]) -> Vec<Point> {
        let mid_point = Point::new(end.x, start.y);
        vec![start, mid_point, end]
    }
    
    fn try_vertical_first_routing(&self, start: Point, end: Point, _obstacles: &[Rectangle]) -> Vec<Point> {
        let mid_point = Point::new(start.x, end.y);
        vec![start, mid_point, end]
    }
    
    fn try_obstacle_avoidance_routing(&self, start: Point, end: Point, obstacles: &[Rectangle]) -> Vec<Point> {
        let mut path = vec![start];
        
        // More sophisticated obstacle avoidance
        if !obstacles.is_empty() {
            let min_x = obstacles.iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
            let max_x = obstacles.iter().map(|r| r.x + r.width).fold(f64::NEG_INFINITY, f64::max);
            let min_y = obstacles.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
            let max_y = obstacles.iter().map(|r| r.y + r.height).fold(f64::NEG_INFINITY, f64::max);
            
            let margin = 40.0;
            
            // Determine routing strategy based on relative positions
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            
            // Check if we need to route around obstacles
            let direct_line_blocked = self.path_intersects_obstacles(&[start, end], obstacles);
            
            if direct_line_blocked {
                // Try routing around the obstacle cluster
                if dx.abs() > dy.abs() {
                    // Primarily horizontal movement
                    if start.y < min_y - margin {
                        // Route above obstacles
                        let route_y = min_y - margin;
                        path.push(Point::new(start.x, route_y));
                        path.push(Point::new(end.x, route_y));
                    } else if start.y > max_y + margin {
                        // Route below obstacles
                        let route_y = max_y + margin;
                        path.push(Point::new(start.x, route_y));
                        path.push(Point::new(end.x, route_y));
                    } else {
                        // Route around side
                        if dx > 0.0 {
                            // Going right, try routing below then above
                            let route_y = if start.y > (min_y + max_y) / 2.0 { max_y + margin } else { min_y - margin };
                            path.push(Point::new(start.x, route_y));
                            path.push(Point::new(end.x, route_y));
                        } else {
                            // Going left
                            let route_y = if start.y > (min_y + max_y) / 2.0 { max_y + margin } else { min_y - margin };
                            path.push(Point::new(start.x, route_y));
                            path.push(Point::new(end.x, route_y));
                        }
                    }
                } else {
                    // Primarily vertical movement
                    if start.x < min_x - margin {
                        // Route left of obstacles
                        let route_x = min_x - margin;
                        path.push(Point::new(route_x, start.y));
                        path.push(Point::new(route_x, end.y));
                    } else if start.x > max_x + margin {
                        // Route right of obstacles
                        let route_x = max_x + margin;
                        path.push(Point::new(route_x, start.y));
                        path.push(Point::new(route_x, end.y));
                    } else {
                        // Route around side
                        if dy > 0.0 {
                            // Going down
                            let route_x = if start.x > (min_x + max_x) / 2.0 { max_x + margin } else { min_x - margin };
                            path.push(Point::new(route_x, start.y));
                            path.push(Point::new(route_x, end.y));
                        } else {
                            // Going up
                            let route_x = if start.x > (min_x + max_x) / 2.0 { max_x + margin } else { min_x - margin };
                            path.push(Point::new(route_x, start.y));
                            path.push(Point::new(route_x, end.y));
                        }
                    }
                }
            }
        }
        
        path.push(end);
        path
    }
    
    fn calculate_path_length(&self, path: &[Point]) -> f64 {
        let mut length = 0.0;
        for i in 1..path.len() {
            let dx = path[i].x - path[i-1].x;
            let dy = path[i].y - path[i-1].y;
            length += (dx * dx + dy * dy).sqrt();
        }
        length
    }
    
    fn path_intersects_obstacles(&self, path: &[Point], obstacles: &[Rectangle]) -> bool {
        for i in 0..path.len() - 1 {
            let start = path[i];
            let end = path[i + 1];
            
            for obstacle in obstacles {
                if self.line_intersects_rectangle(start, end, obstacle) {
                    return true;
                }
            }
        }
        false
    }

    fn path_intersects_tables(&self, path_points: &[Point]) -> bool {
        for i in 0..path_points.len() - 1 {
            let start = path_points[i];
            let end = path_points[i + 1];
            
            for layout in &self.table_layouts {
                if self.line_intersects_rectangle(start, end, &layout.bounds) {
                    return true;
                }
            }
        }
        false
    }

    fn line_intersects_rectangle(&self, start: Point, end: Point, rect: &Rectangle) -> bool {
        // Add margin around rectangle for line avoidance
        let margin = 20.0;
        let expanded_rect = Rectangle::new(
            rect.x - margin,
            rect.y - margin,
            rect.width + (margin * 2.0),
            rect.height + (margin * 2.0)
        );
        
        // Check if either endpoint is inside the expanded rectangle
        if self.point_in_rectangle(start, &expanded_rect) || self.point_in_rectangle(end, &expanded_rect) {
            return true;
        }

        // Check if line intersects any of the rectangle's edges
        let rect_corners = [
            Point::new(expanded_rect.x, expanded_rect.y),
            Point::new(expanded_rect.x + expanded_rect.width, expanded_rect.y),
            Point::new(expanded_rect.x + expanded_rect.width, expanded_rect.y + expanded_rect.height),
            Point::new(expanded_rect.x, expanded_rect.y + expanded_rect.height),
        ];

        for i in 0..4 {
            let edge_start = rect_corners[i];
            let edge_end = rect_corners[(i + 1) % 4];
            
            if self.line_segments_intersect(start, end, edge_start, edge_end) {
                return true;
            }
        }

        false
    }
    
    fn point_in_rectangle(&self, point: Point, rect: &Rectangle) -> bool {
        point.x >= rect.x && point.x <= rect.x + rect.width &&
        point.y >= rect.y && point.y <= rect.y + rect.height
    }

    fn line_segments_intersect(&self, p1: Point, q1: Point, p2: Point, q2: Point) -> bool {
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