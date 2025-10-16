use crate::ast::{Table, Relationship};
use super::layout::{Point, Rectangle, TableLayout};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ForceNode {
    pub table: Table,
    pub position: Point,
    pub velocity: Point,
    pub bounds: Rectangle,
    pub mass: f64,
}

impl ForceNode {
    pub fn new(table: Table, x: f64, y: f64) -> Self {
        let temp_layout = TableLayout::new(table.clone(), x, y);
        let bounds = temp_layout.bounds;
        
        ForceNode {
            table,
            position: Point::new(x, y),
            velocity: Point::new(0.0, 0.0),
            bounds,
            mass: bounds.width * bounds.height / 10000.0, // Mass based on table size
        }
    }
    
    pub fn update_bounds(&mut self) {
        let temp_layout = TableLayout::new(self.table.clone(), self.position.x, self.position.y);
        self.bounds = temp_layout.bounds;
    }
}

#[derive(Debug, Clone)]
pub struct ForceEdge {
    pub from_index: usize,
    pub to_index: usize,
    pub ideal_length: f64,
    pub strength: f64,
}

pub struct FruchtermanReingoldLayout {
    pub nodes: Vec<ForceNode>,
    pub edges: Vec<ForceEdge>,
    pub width: f64,
    pub height: f64,
    pub k: f64, // Optimal distance between nodes
    pub temperature: f64,
    pub cooling_factor: f64,
    pub iterations: usize,
}

impl FruchtermanReingoldLayout {
    pub fn new(tables: &[Table], relationships: &[Relationship], _width: f64, _height: f64) -> Self {
        let mut nodes = Vec::new();
        let mut table_indices = HashMap::new();
        
        // Calculate optimal canvas size based on table count and sizes
        let table_count = tables.len() as f64;
        
        // Estimate average table size
        let avg_table_width = 200.0; // Estimated average table width
        let avg_table_height = 150.0; // Estimated average table height
        
        // Calculate optimal canvas dimensions
        // Use a formula that scales with table count and ensures adequate spacing
        let min_spacing = 350.0; // Increased minimum spacing between tables
        let margin = 200.0; // Increased canvas margins
        
        // For optimal layout, aim for roughly square arrangement
        let cols = (table_count.sqrt().ceil()).max(2.0);
        let rows = (table_count / cols).ceil().max(2.0);
        
        let canvas_width = (cols * (avg_table_width + min_spacing) + margin * 2.0).max(1600.0);
        let canvas_height = (rows * (avg_table_height + min_spacing) + margin * 2.0).max(1200.0);
        
        println!("📐 Auto-sizing canvas: {}x{} for {} tables (organic layout)", 
                canvas_width as i32, canvas_height as i32, table_count as i32);
        
        // Create nodes with organic initial positioning - no rigid grid
        use std::f64::consts::PI;
        
        for (i, table) in tables.iter().enumerate() {
            // Use multiple positioning strategies for more natural distribution
            let angle = (i as f64 * 2.4) % (2.0 * PI); // Golden angle-like distribution
            let radius_factor = 0.3 + (i as f64 * 0.17) % 0.4; // Varying distances from center
            
            let center_x = canvas_width * 0.5;
            let center_y = canvas_height * 0.5;
            
            // Create organic spiral-like distribution with randomization
            let spiral_radius = (canvas_width.min(canvas_height) * 0.3) * radius_factor;
            let spiral_angle = angle + (i as f64 * 0.618) % (2.0 * PI); // Golden ratio for better distribution
            
            let base_x = center_x + spiral_radius * spiral_angle.cos();
            let base_y = center_y + spiral_radius * spiral_angle.sin();
            
            // Add some controlled randomization to break perfect patterns
            let random_offset_x = ((i as f64 * 17.0) % 200.0) - 100.0; // -100 to +100
            let random_offset_y = ((i as f64 * 23.0) % 200.0) - 100.0; // -100 to +100
            
            let x = (base_x + random_offset_x).max(margin).min(canvas_width - margin);
            let y = (base_y + random_offset_y).max(margin).min(canvas_height - margin);
            
            nodes.push(ForceNode::new(table.clone(), x, y));
            table_indices.insert(table.name.clone(), i);
        }
        
        // Create edges from relationships
        let mut edges = Vec::new();
        for relationship in relationships {
            if let (Some(&from_idx), Some(&to_idx)) = (
                table_indices.get(&relationship.from_table),
                table_indices.get(&relationship.to_table)
            ) {
                edges.push(ForceEdge {
                    from_index: from_idx,
                    to_index: to_idx,
                    ideal_length: 450.0, // Further increased ideal distance for more spread
                    strength: 0.6, // Weaker attraction to allow more spread
                });
            }
        }
        
        let area = canvas_width * canvas_height;
        let k = (area / tables.len() as f64).sqrt() * 1.2; // Reduced for more natural clustering
        
        FruchtermanReingoldLayout {
            nodes,
            edges,
            width: canvas_width,
            height: canvas_height,
            k,
            temperature: canvas_width.min(canvas_height) / 6.0, // Higher initial temperature for more movement
            cooling_factor: 0.96, // Faster cooling to allow settling into irregular patterns
            iterations: 250, // More iterations for better organic distribution
        }
    }
    
    pub fn run_layout(&mut self) {
        for iteration in 0..self.iterations {
            self.calculate_forces();
            self.update_positions();
            self.cool_temperature();
            
            // Early termination if system is stable
            if self.temperature < 1.0 {
                break;
            }
            
            // Progress indication for large layouts
            if iteration % 20 == 0 {
                println!("FR Layout iteration {}/{}", iteration, self.iterations);
            }
        }
        
        // Final bounds update
        for node in &mut self.nodes {
            node.update_bounds();
        }
    }
    
    fn calculate_forces(&mut self) {
        // Reset velocities
        for node in &mut self.nodes {
            node.velocity = Point::new(0.0, 0.0);
        }
        
        // Calculate repulsive forces between all pairs of nodes
        for i in 0..self.nodes.len() {
            for j in (i + 1)..self.nodes.len() {
                let dx = self.nodes[j].position.x - self.nodes[i].position.x;
                let dy = self.nodes[j].position.y - self.nodes[i].position.y;
                let distance = (dx * dx + dy * dy).sqrt().max(10.0); // Minimum distance to prevent explosion
                
                // Adaptive repulsion based on table sizes and relationship context
                let table_i_size = (self.nodes[i].bounds.width + self.nodes[i].bounds.height) / 2.0;
                let table_j_size = (self.nodes[j].bounds.width + self.nodes[j].bounds.height) / 2.0;
                let min_distance = table_i_size + table_j_size + 150.0; // Reduced for more natural clustering
                
                // Check if these tables are connected by relationships
                let are_connected = self.edges.iter().any(|edge| 
                    (edge.from_index == i && edge.to_index == j) || 
                    (edge.from_index == j && edge.to_index == i)
                );
                
                // Variable repulsion strength based on connection and distance
                let repulsion_strength = if distance < min_distance {
                    // Strong repulsion when overlapping, but less for connected tables
                    let base_strength = (self.k * self.k * 3.0) / (distance * 0.2);
                    if are_connected { base_strength * 0.7 } else { base_strength }
                } else {
                    // Weaker normal repulsion to allow organic clustering
                    let base_strength = (self.k * self.k * 1.2) / (distance * distance);
                    if are_connected { base_strength * 0.5 } else { base_strength }
                };
                
                let fx = (dx / distance) * repulsion_strength;
                let fy = (dy / distance) * repulsion_strength;
                
                // Apply forces (Newton's third law) with mass consideration
                let mass_i = self.nodes[i].mass.max(0.1);
                let mass_j = self.nodes[j].mass.max(0.1);
                
                self.nodes[i].velocity.x -= fx / mass_i;
                self.nodes[i].velocity.y -= fy / mass_i;
                self.nodes[j].velocity.x += fx / mass_j;
                self.nodes[j].velocity.y += fy / mass_j;
            }
        }
        
        // Calculate attractive forces for connected nodes
        for edge in &self.edges {
            let i = edge.from_index;
            let j = edge.to_index;
            
            let dx = self.nodes[j].position.x - self.nodes[i].position.x;
            let dy = self.nodes[j].position.y - self.nodes[i].position.y;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);
            
            let attraction_strength = (distance * distance) / (self.k * edge.ideal_length) * edge.strength;
            
            let fx = (dx / distance) * attraction_strength;
            let fy = (dy / distance) * attraction_strength;
            
            // Apply attractive forces
            self.nodes[i].velocity.x += fx / self.nodes[i].mass;
            self.nodes[i].velocity.y += fy / self.nodes[i].mass;
            self.nodes[j].velocity.x -= fx / self.nodes[j].mass;
            self.nodes[j].velocity.y -= fy / self.nodes[j].mass;
        }
    }
    
    fn update_positions(&mut self) {
        for node in &mut self.nodes {
            // Limit velocity by temperature with damping
            let velocity_magnitude = (node.velocity.x * node.velocity.x + 
                                    node.velocity.y * node.velocity.y).sqrt();
            
            let max_velocity = self.temperature * 2.0; // Allow higher velocities for better separation
            if velocity_magnitude > max_velocity {
                node.velocity.x = (node.velocity.x / velocity_magnitude) * max_velocity;
                node.velocity.y = (node.velocity.y / velocity_magnitude) * max_velocity;
            }
            
            // Apply damping to reduce oscillations
            node.velocity.x *= 0.9;
            node.velocity.y *= 0.9;
            
            // Update position
            node.position.x += node.velocity.x;
            node.position.y += node.velocity.y;
            
            // Keep nodes within bounds with generous margin
            let margin = 100.0;
            let max_x = self.width - node.bounds.width - margin;
            let max_y = self.height - node.bounds.height - margin;
            
            node.position.x = node.position.x.max(margin).min(max_x);
            node.position.y = node.position.y.max(margin).min(max_y);
            
            // If hitting boundaries, add repulsion from edges
            if node.position.x <= margin || node.position.x >= max_x {
                node.velocity.x *= -0.5; // Bounce back from horizontal edges
            }
            if node.position.y <= margin || node.position.y >= max_y {
                node.velocity.y *= -0.5; // Bounce back from vertical edges
            }
            
            // Update bounds after position change
            node.update_bounds();
        }
    }
    
    fn cool_temperature(&mut self) {
        self.temperature *= self.cooling_factor;
    }
    
    pub fn get_table_layouts(&self) -> Vec<TableLayout> {
        self.nodes.iter()
            .map(|node| TableLayout::new(node.table.clone(), node.position.x, node.position.y))
            .collect()
    }
    
    pub fn get_canvas_size(&self) -> (f64, f64) {
        let margin = 100.0;
        
        let max_x = self.nodes.iter()
            .map(|node| node.position.x + node.bounds.width)
            .fold(0.0, f64::max);
        
        let max_y = self.nodes.iter()
            .map(|node| node.position.y + node.bounds.height)
            .fold(0.0, f64::max);
        
        ((max_x + margin).max(800.0), (max_y + margin).max(600.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Column, DataType, RelationshipType};

    #[test]
    fn test_fruchterman_reingold_layout() {
        let mut tables = Vec::new();
        
        // Create test tables
        let mut table1 = Table::new("Table1".to_string());
        table1.columns.push(Column::new("id".to_string(), DataType::Int));
        tables.push(table1);
        
        let mut table2 = Table::new("Table2".to_string());
        table2.columns.push(Column::new("id".to_string(), DataType::Int));
        tables.push(table2);
        
        let relationships = vec![
            Relationship {
                from_table: "Table1".to_string(),
                from_field: "id".to_string(),
                to_table: "Table2".to_string(),
                to_field: "id".to_string(),
                relationship_type: RelationshipType::OneToMany,
            }
        ];
        
        let mut layout = FruchtermanReingoldLayout::new(&tables, &relationships, 800.0, 600.0);
        layout.run_layout();
        
        let table_layouts = layout.get_table_layouts();
        assert_eq!(table_layouts.len(), 2);
        
        // Check that tables are not overlapping
        let distance = ((table_layouts[1].bounds.x - table_layouts[0].bounds.x).powi(2) +
                       (table_layouts[1].bounds.y - table_layouts[0].bounds.y).powi(2)).sqrt();
        assert!(distance > 100.0, "Tables should be properly spaced apart");
    }
}
