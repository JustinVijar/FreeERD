use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use euclid::{Point2D, Size2D};
use rand::Rng;

use super::graph::{ErdGraph, RelationType};

pub struct UnknownUnit;
pub type Point = Point2D<f32, UnknownUnit>;
pub type Size = Size2D<f32, UnknownUnit>;

/// Layout information for a node
#[derive(Debug, Clone)]
pub struct NodeLayout {
    pub position: Point,
    pub size: Size,
    #[allow(dead_code)]
    pub layer: usize,
}

/// Routed edge path (orthogonal) with relationship type
#[derive(Debug, Clone)]
pub struct EdgeRoute {
    pub points: Vec<Point>,
    pub relationship_type: RelationType,
    #[allow(dead_code)]
    pub is_self_referencing: bool,
    pub label: String,
    pub from_table: String,
    pub to_table: String,
}

pub struct LayoutEngine {
    node_layouts: HashMap<NodeIndex, NodeLayout>,
    edge_routes: Vec<EdgeRoute>,
    min_spacing: f32,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            node_layouts: HashMap::new(),
            edge_routes: Vec::new(),
            min_spacing: 80.0,  // Minimum spacing between nodes
        }
    }

    /// Compute layout with floating-point coordinates
    pub fn compute_layout(&mut self, graph: &ErdGraph) {
        let node_count = graph.graph().node_count();
        if node_count == 0 {
            return;
        }
        
        println!("Computing layout for {} nodes", node_count);
        
        // Initialize node layouts with proper sizes and initial positions
        self.initialize_node_layouts(graph);
        
        // Use force-directed placement
        self.force_directed_layout(graph);
        
        // Route edges orthogonally
        self.route_edges_orthogonal(graph);
        
        println!("Layout complete");
    }
    
    /// Initialize node layouts with calculated sizes
    fn initialize_node_layouts(&mut self, graph: &ErdGraph) {
        let g = graph.graph();
        let mut rng = rand::thread_rng();
        
        let node_count = g.node_count();
        // Arrange in a grid pattern initially
        let cols = (node_count as f32).sqrt().ceil() as usize;
        let spacing_x = 400.0;  // Horizontal spacing (increased for wider tables)
        let spacing_y = 250.0;  // Vertical spacing
        
        for (idx, node) in g.node_indices().enumerate() {
            let table = &g[node];
            
            // Calculate width based on content (use character-based estimation)
            let min_width = 200.0;
            let max_width = 500.0;
            
            // Estimate width based on table name
            let name_width = table.name.len() as f32 * 9.0 + 40.0; // ~9px per char + padding
            
            // Estimate width based on longest column content
            let mut max_column_width: f32 = 0.0;
            for column in &table.columns {
                // Column name
                let col_name_width = column.name.len() as f32 * 7.0;
                
                // Data type and attributes
                let type_text = if column.attributes.is_empty() {
                    column.data_type.clone()
                } else {
                    format!("{} [{}]", column.data_type, column.attributes.join(","))
                };
                let type_width = type_text.len() as f32 * 6.0;
                
                let total_column_width = col_name_width + type_width + 40.0; // Add padding
                max_column_width = max_column_width.max(total_column_width);
            }
            
            // Choose the maximum of name width and column width, clamped to min/max
            let width = name_width.max(max_column_width).max(min_width).min(max_width);
            let height = 60.0 + table.columns.len() as f32 * 25.0;
            
            // Place in grid with some randomness
            let col = idx % cols;
            let row = idx / cols;
            let x = col as f32 * spacing_x + rng.gen_range(-30.0..30.0);
            let y = row as f32 * spacing_y + rng.gen_range(-30.0..30.0);
            
            println!("Node {} '{}' at ({:.1}, {:.1}) size ({:.1}, {:.1})", 
                node.index(), table.name, x, y, width, height);
            
            self.node_layouts.insert(
                node,
                NodeLayout {
                    position: Point::new(x, y),
                    size: Size::new(width, height),
                    layer: 0,
                },
            );
        }
    }
    
    /// Force-directed layout with collision avoidance
    fn force_directed_layout(&mut self, graph: &ErdGraph) {
        let g = graph.graph();
        let node_count = g.node_count();
        let iterations = 150.min(node_count * 30);
        
        for iter in 0..iterations {
            let mut forces: HashMap<NodeIndex, (f32, f32)> = HashMap::new();
            
            // Initialize forces
            for node in g.node_indices() {
                forces.insert(node, (0.0, 0.0));
            }
            
            // Attractive forces along edges (pull connected nodes together)
            for edge in g.edge_references() {
                let source = edge.source();
                let target = edge.target();
                
                if let (Some(src_layout), Some(tgt_layout)) = 
                    (self.node_layouts.get(&source), self.node_layouts.get(&target)) 
                {
                    let dx = tgt_layout.position.x - src_layout.position.x;
                    let dy = tgt_layout.position.y - src_layout.position.y;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    
                    // Spring force (stronger when far apart)
                    let force = dist / 150.0;
                    let fx = (dx / dist) * force;
                    let fy = (dy / dist) * force;
                    
                    forces.get_mut(&source).unwrap().0 += fx;
                    forces.get_mut(&source).unwrap().1 += fy;
                    forces.get_mut(&target).unwrap().0 -= fx;
                    forces.get_mut(&target).unwrap().1 -= fy;
                }
            }
            
            // Repulsive forces between all nodes (push overlapping nodes apart)
            let nodes: Vec<NodeIndex> = g.node_indices().collect();
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let n1 = nodes[i];
                    let n2 = nodes[j];
                    
                    if let (Some(l1), Some(l2)) = 
                        (self.node_layouts.get(&n1), self.node_layouts.get(&n2)) 
                    {
                        let dx = l2.position.x - l1.position.x;
                        let dy = l2.position.y - l1.position.y;
                        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                        
                        // Calculate minimum distance to avoid overlap
                        let min_dist = ((l1.size.width + l2.size.width) / 2.0 + 
                                       (l1.size.height + l2.size.height) / 2.0 + 
                                       self.min_spacing).max(200.0);
                        
                        // Strong repulsion when too close
                        if dist < min_dist * 1.5 {
                            let force = (min_dist * min_dist) / (dist * dist + 1.0);
                            let fx = (dx / dist) * force;
                            let fy = (dy / dist) * force;
                            
                            forces.get_mut(&n1).unwrap().0 -= fx;
                            forces.get_mut(&n1).unwrap().1 -= fy;
                            forces.get_mut(&n2).unwrap().0 += fx;
                            forces.get_mut(&n2).unwrap().1 += fy;
                        }
                    }
                }
            }
            
            // Apply forces with damping (decreases over time)
            let damping = 0.9 - (iter as f32 / iterations as f32) * 0.7;
            for (node, (fx, fy)) in forces {
                if let Some(layout) = self.node_layouts.get_mut(&node) {
                    layout.position.x += fx * damping;
                    layout.position.y += fy * damping;
                }
            }
            
            if iter % 20 == 0 {
                println!("Layout iteration {}/{}", iter, iterations);
            }
        }
        
        // Final cleanup: resolve any remaining overlaps
        self.resolve_overlaps(graph);
    }
    
    /// Resolve any remaining overlaps
    fn resolve_overlaps(&mut self, graph: &ErdGraph) {
        let g = graph.graph();
        let nodes: Vec<NodeIndex> = g.node_indices().collect();
        
        for pass in 0..15 {  // Multiple passes to ensure no overlaps
            let mut moved = false;
            
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let n1 = nodes[i];
                    let n2 = nodes[j];
                    
                    let overlap = {
                        let l1 = self.node_layouts.get(&n1).unwrap();
                        let l2 = self.node_layouts.get(&n2).unwrap();
                        
                        let dx = l2.position.x - l1.position.x;
                        let dy = l2.position.y - l1.position.y;
                        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                        
                        // Check if bounding boxes overlap
                        let min_dist = (l1.size.width + l2.size.width) / 2.0 + 
                                      (l1.size.height + l2.size.height) / 2.0 + 
                                      self.min_spacing;
                        
                        if dist < min_dist {
                            Some((dx, dy, dist, min_dist))
                        } else {
                            None
                        }
                    };
                    
                    if let Some((dx, dy, dist, min_dist)) = overlap {
                        // Push nodes apart
                        let separation = (min_dist - dist) / 2.0 + 5.0;
                        let nx = dx / dist;
                        let ny = dy / dist;
                        
                        self.node_layouts.get_mut(&n1).unwrap().position.x -= nx * separation;
                        self.node_layouts.get_mut(&n1).unwrap().position.y -= ny * separation;
                        self.node_layouts.get_mut(&n2).unwrap().position.x += nx * separation;
                        self.node_layouts.get_mut(&n2).unwrap().position.y += ny * separation;
                        
                        moved = true;
                    }
                }
            }
            
            if !moved {
                println!("No overlaps after {} passes", pass + 1);
                break;
            }
        }
    }
    
    /// Route edges with orthogonal paths
    fn route_edges_orthogonal(&mut self, graph: &ErdGraph) {
        self.edge_routes.clear();
        let g = graph.graph();
        
        // First pass: count edges per node to help distribute connection points
        let mut edge_counts: HashMap<NodeIndex, (usize, usize)> = HashMap::new(); // (current_index, total_count)
        for edge in g.edge_references() {
            let source = edge.source();
            let target = edge.target();
            edge_counts.entry(source).or_insert((0, 0)).1 += 1;
            edge_counts.entry(target).or_insert((0, 0)).1 += 1;
        }
        
        // Reset current indices
        for (_, counts) in edge_counts.iter_mut() {
            counts.0 = 0;
        }
        
        for edge in g.edge_references() {
            let source = edge.source();
            let target = edge.target();
            let edge_data = edge.weight();
            let rel_type = edge_data.relationship_type;
            
            // Get table names
            let from_table = &g[source].name;
            let to_table = &g[target].name;
            
            let label = format!("{}:{}", edge_data.from_field, edge_data.to_field);
            
            // Check for self-referencing edge
            if source == target {
                if let Some(layout) = self.node_layouts.get(&source) {
                    let points = self.create_self_referencing_path(layout);
                    self.edge_routes.push(EdgeRoute { 
                        points,
                        relationship_type: rel_type,
                        is_self_referencing: true,
                        label,
                        from_table: from_table.clone(),
                        to_table: to_table.clone(),
                    });
                }
                // Increment counter
                if let Some(counts) = edge_counts.get_mut(&source) {
                    counts.0 += 1;
                }
            } else if let (Some(src_layout), Some(tgt_layout)) = 
                (self.node_layouts.get(&source), self.node_layouts.get(&target)) 
            {
                // Get current edge indices for distribution
                let src_edge_info = edge_counts.get(&source).copied().unwrap_or((0, 1));
                let tgt_edge_info = edge_counts.get(&target).copied().unwrap_or((0, 1));
                
                let points = self.create_orthogonal_path_distributed(
                    src_layout, tgt_layout, 
                    src_edge_info.0, src_edge_info.1,
                    tgt_edge_info.0, tgt_edge_info.1
                );
                
                self.edge_routes.push(EdgeRoute { 
                    points,
                    relationship_type: rel_type,
                    is_self_referencing: false,
                    label,
                    from_table: from_table.clone(),
                    to_table: to_table.clone(),
                });
                
                // Increment counters
                if let Some(counts) = edge_counts.get_mut(&source) {
                    counts.0 += 1;
                }
                if let Some(counts) = edge_counts.get_mut(&target) {
                    counts.0 += 1;
                }
            }
        }
    }
    
    /// Create self-referencing loopback path
    fn create_self_referencing_path(&self, layout: &NodeLayout) -> Vec<Point> {
        let right = layout.position.x + layout.size.width;
        let top = layout.position.y;
        let offset = 40.0; // Size of the loop
        
        // Create a loop on the right side of the table
        vec![
            Point::new(right, top + layout.size.height * 0.3),
            Point::new(right + offset, top + layout.size.height * 0.3),
            Point::new(right + offset, top + layout.size.height * 0.7),
            Point::new(right, top + layout.size.height * 0.7),
        ]
    }
    
    /// Create orthogonal path between two nodes (H-V-H or V-H-V)
    /// Create orthogonal path with distributed connection points
    fn create_orthogonal_path_distributed(
        &self, 
        src: &NodeLayout, 
        tgt: &NodeLayout,
        src_edge_idx: usize,
        src_edge_total: usize,
        tgt_edge_idx: usize,
        tgt_edge_total: usize,
    ) -> Vec<Point> {
        let src_center = Point::new(
            src.position.x + src.size.width / 2.0,
            src.position.y + src.size.height / 2.0,
        );
        let tgt_center = Point::new(
            tgt.position.x + tgt.size.width / 2.0,
            tgt.position.y + tgt.size.height / 2.0,
        );
        
        let dx = tgt_center.x - src_center.x;
        let dy = tgt_center.y - src_center.y;
        
        // Determine exit and entry points with distribution
        let src_exit = self.get_distributed_exit_point(src, tgt_center, src_edge_idx, src_edge_total);
        let tgt_entry = self.get_distributed_exit_point(tgt, src_center, tgt_edge_idx, tgt_edge_total);
        
        // Choose H-V-H or V-H-V based on which is more horizontal/vertical
        if dx.abs() > dy.abs() {
            // H-V-H path
            let mid_x = (src_exit.x + tgt_entry.x) / 2.0;
            vec![
                src_exit,
                Point::new(mid_x, src_exit.y),
                Point::new(mid_x, tgt_entry.y),
                tgt_entry,
            ]
        } else {
            // V-H-V path
            let mid_y = (src_exit.y + tgt_entry.y) / 2.0;
            vec![
                src_exit,
                Point::new(src_exit.x, mid_y),
                Point::new(tgt_entry.x, mid_y),
                tgt_entry,
            ]
        }
    }
    
    /// Get distributed exit point that spreads multiple edges along the border
    fn get_distributed_exit_point(
        &self, 
        layout: &NodeLayout, 
        target: Point,
        edge_index: usize,
        total_edges: usize,
    ) -> Point {
        let left = layout.position.x;
        let right = layout.position.x + layout.size.width;
        let top = layout.position.y;
        let bottom = layout.position.y + layout.size.height;
        
        let center_x = layout.position.x + layout.size.width / 2.0;
        let center_y = layout.position.y + layout.size.height / 2.0;
        
        // Calculate direction from center to target
        let dx = target.x - center_x;
        let dy = target.y - center_y;
        
        // Calculate distribution factor (evenly distribute edges)
        let margin = 30.0; // Margin from edges of the table
        let distribution = if total_edges > 1 {
            (edge_index as f32) / (total_edges as f32 - 1.0)
        } else {
            0.5 // Center if only one edge
        };
        
        // Determine which side based on the angle
        if dx.abs() > dy.abs() {
            // More horizontal - exit from left or right side
            let available_height = layout.size.height - 2.0 * margin;
            let exit_y = top + margin + distribution * available_height;
            
            if dx > 0.0 {
                Point::new(right, exit_y)
            } else {
                Point::new(left, exit_y)
            }
        } else {
            // More vertical - exit from top or bottom side
            let available_width = layout.size.width - 2.0 * margin;
            let exit_x = left + margin + distribution * available_width;
            
            if dy > 0.0 {
                Point::new(exit_x, bottom)
            } else {
                Point::new(exit_x, top)
            }
        }
    }
    
    pub fn get_node_layout(&self, node: NodeIndex) -> Option<&NodeLayout> {
        self.node_layouts.get(&node)
    }
    
    pub fn get_node_layout_mut(&mut self, node: NodeIndex) -> Option<&mut NodeLayout> {
        self.node_layouts.get_mut(&node)
    }
    
    pub fn get_edge_routes(&self) -> &[EdgeRoute] {
        &self.edge_routes
    }
    
    /// Recompute edge routes after nodes have moved
    pub fn recompute_edge_routes(&mut self, graph: &ErdGraph) {
        self.route_edges_orthogonal(graph);
    }
}
