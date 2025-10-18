# Graph Database Concepts

Understanding graph databases and when to use them with FreeERD.

## Table of Contents

1. [What are Graph Databases?](#what-are-graph-databases)
2. [When to Use Graph Databases](#when-to-use-graph-databases)
3. [Nodes vs Tables](#nodes-vs-tables)
4. [Edges vs Relationships](#edges-vs-relationships)
5. [Graph Database Patterns](#graph-database-patterns)
6. [Migration from Relational](#migration-from-relational)

## What are Graph Databases?

Graph databases store data as **nodes** (entities) connected by **edges** (relationships). Unlike relational databases that use tables and foreign keys, graph databases make relationships first-class citizens.

### Key Characteristics

- **Nodes**: Represent entities (users, products, concepts)
- **Edges**: Represent relationships between nodes
- **Properties**: Key-value data stored on nodes and edges
- **Traversal**: Efficient navigation through connected data

### Advantages

- **Relationship-focused**: Perfect for highly connected data
- **Flexible schema**: Easy to add new relationship types
- **Fast traversals**: Efficient for "friends of friends" queries
- **Intuitive modeling**: Mirrors real-world relationships

## When to Use Graph Databases

### Use Graph Databases When:

- **Relationships are complex**: Many-to-many relationships, hierarchies
- **Data is highly connected**: Social networks, recommendation systems
- **Queries follow relationships**: "What products do similar users buy?"
- **Schema evolves frequently**: Adding new relationship types
- **Path finding**: Shortest paths, network analysis

### Use Relational Databases When:

- **Data is tabular**: Simple entity types with fixed attributes
- **Relationships are simple**: Mostly one-to-many or many-to-one
- **ACID transactions**: Strict consistency requirements
- **Complex aggregations**: SUM, COUNT, GROUP BY operations
- **Fixed schema**: Well-defined, stable data structure

### Hybrid Approach

Many applications benefit from **both**:

```
# Relational for core business data
table Orders { ... }
table Products { ... }

# Graph for recommendations and relationships
node User { ... }
node Product { ... }
User -[PURCHASED]-> Product
Product <-[SIMILAR_TO]-> Product
```

## Nodes vs Tables

| Aspect | Tables (Relational) | Nodes (Graph) |
|--------|-------------------|---------------|
| **Purpose** | Store structured data | Represent entities |
| **Relationships** | Via foreign keys | Direct connections |
| **Schema** | Fixed columns | Flexible properties |
| **Queries** | JOIN operations | Traversal operations |
| **Best for** | Business transactions | Network analysis |

### Example Comparison

**Relational Table:**
```sql
CREATE TABLE Users (
  id INT PRIMARY KEY,
  name VARCHAR(255),
  email VARCHAR(255) UNIQUE
);
```

**Graph Node:**
```
node User {
  id: int [pk, autoincrement],
  name: str,
  email: str [unique]
}
```

## Edges vs Relationships

| Aspect | Relationships (Relational) | Edges (Graph) |
|--------|---------------------------|---------------|
| **Syntax** | `Table1.field > Table2.field` | `Node1 -[EDGE]-> Node2` |
| **Direction** | Bidirectional by nature | Can be directed or undirected |
| **Properties** | Limited (usually just FK) | Rich properties supported |
| **Multiplicity** | One-to-one, one-to-many, many-to-many | Any cardinality |
| **Self-reference** | Possible but complex | Natural and common |

### Relationship Types

**Relational:**
- One-to-One: `Users.id - Posts.user_id` (unique constraint)
- One-to-Many: `Users.id > Posts.user_id`
- Many-to-One: `Posts.user_id < Users.id`
- Many-to-Many: Junction tables required

**Graph:**
- Directed: `User -[FOLLOWS]-> User`
- Bidirectional: `User <-[FRIENDS_WITH]-> User`
- Complex: Edges with properties and multiple types

## Graph Database Patterns

### Social Networks

```
node User {
  id: int [pk],
  name: str
}

edge FOLLOWS (from: User, to: User) {
  since: date
}

User -[FOLLOWS]-> User
User <-[FRIENDS_WITH]-> User
```

### Recommendation Systems

```
node User { id: int [pk] }
node Product { id: int [pk] }

edge PURCHASED (from: User, to: Product) {
  rating: int,
  date: datetime
}

User -[VIEWED]-> Product
Product <-[SIMILAR_TO]-> Product
```

### Knowledge Graphs

```
node Concept {
  id: int [pk],
  name: str
}

Concept <-[RELATED_TO]-> Concept
Concept <-[SPECIALIZES]-> Concept
```

### Access Control

```
node User { id: int [pk] }
node Resource { id: int [pk] }
node Permission { name: str }

User -[HAS_PERMISSION]-> Permission
Permission -[GRANTS_ACCESS]-> Resource
```

## Migration from Relational

### Step 1: Identify Entities
Convert tables to nodes:

```
table Users {      →      node User {
  id: int [pk]               id: int [pk]
  name: str                  name: str
}                           }
```

### Step 2: Convert Relationships
Foreign keys become edges:

```
table Posts {           edge AUTHORED (from: User, to: Post) {
  user_id: int [fk]         created_at: datetime
}                         }

Users.id > Posts.user_id  →  User -[AUTHORED]-> Post
```

### Step 3: Add Graph-Specific Relationships
Add relationships that weren't practical in relational:

```
User <-[FRIENDS_WITH]-> User
Post <-[TAGGED_WITH]- Tag
Product <-[SIMILAR_TO]-> Product
```

### Step 4: Consider Properties on Edges
Move junction table data to edge properties:

```
table UserLikesPost {     →     edge LIKES (from: User, to: Post) {
  user_id: int [fk]               liked_at: datetime
  post_id: int [fk]               reaction: str
  liked_at: datetime
}                               }
```

## Best Practices

### 1. Choose Clear Edge Names
Use descriptive, action-oriented names:

```
// ✅ Good
User -[FOLLOWS]-> User
User -[PURCHASED]-> Product

// ❌ Avoid
User -[REL1]-> User
User -[CONNECTS_TO]-> Product
```

### 2. Use Consistent Direction
Establish conventions for edge direction:

```
// Users follow other users
User -[FOLLOWS]-> User

// Users author posts
User -[AUTHORED]-> Post

// Posts belong to categories
Post -[BELONGS_TO]-> Category
```

### 3. Leverage Edge Properties
Store relationship metadata on edges:

```
edge FOLLOWS (from: User, to: User) {
  since: date,
  closeness: str  // "close", "acquaintance", "distant"
}
```

### 4. Consider Bidirectional Edges
Use `<->` for mutual relationships:

```
User <-[FRIENDS_WITH]-> User
User <-[MARRIED_TO]-> User
```

### 5. Plan for Traversals
Design your graph for common query patterns:

```
// For "friends of friends"
User -[FRIENDS_WITH]-> User -[FRIENDS_WITH]-> User

// For recommendations
User -[PURCHASED]-> Product <-[PURCHASED]- User
```

## See Also

- [Schema Syntax Reference](schema-syntax.md)
- [Examples](examples.md)
- [Getting Started](getting-started.md)</content>
<parameter name="filePath">/home/psg420/vscode/FreeERD/free-erd/documentation/graph-concepts.md