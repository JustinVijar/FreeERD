use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Represents a table in the ERD
#[derive(Debug, Clone)]
pub struct TableNode {
    pub name: String,
    pub columns: Vec<ColumnData>,
}

#[derive(Debug, Clone)]
pub struct ColumnData {
    pub name: String,
    pub data_type: String,
    pub attributes: Vec<String>,
}

/// Represents a relationship edge
#[derive(Debug, Clone)]
pub struct RelationshipEdge {
    pub from_field: String,
    pub to_field: String,
    pub relationship_type: RelationType,
    }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// The ERD graph structure using petgraph
pub struct ErdGraph {
    pub(crate) graph: DiGraph<TableNode, RelationshipEdge>,
    pub(crate) node_map: HashMap<String, NodeIndex>,
}

impl ErdGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_table(&mut self, table: TableNode) -> NodeIndex {
        let name = table.name.clone();
        let idx = self.graph.add_node(table);
        self.node_map.insert(name, idx);
        idx
    }

    pub fn add_relationship(
        &mut self,
        from_table: &str,
        to_table: &str,
        edge: RelationshipEdge,
    ) -> Result<(), String> {
        let from_idx = self.node_map.get(from_table)
            .ok_or_else(|| format!("Table '{}' not found", from_table))?;
        let to_idx = self.node_map.get(to_table)
            .ok_or_else(|| format!("Table '{}' not found", to_table))?;
        
        self.graph.add_edge(*from_idx, *to_idx, edge);
        Ok(())
    }

    pub fn graph(&self) -> &DiGraph<TableNode, RelationshipEdge> {
        &self.graph
    }
}

impl Default for ErdGraph {
    fn default() -> Self {
        Self::new()
    }
}
