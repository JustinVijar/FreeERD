use petgraph::graph::NodeIndex;

#[derive(Debug, Clone, PartialEq)]
pub enum DragTarget {
    None,
    Table(NodeIndex),
    Label(usize), // Index into edge_routes
    Title,
}
