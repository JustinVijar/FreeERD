# FreeERD Documentation

Welcome to the FreeERD documentation! This comprehensive guide will help you master creating Entity Relationship Diagrams and graph database schemas with FreeERD.

## 📚 Documentation Index

### Getting Started
- **[Getting Started Guide](getting-started.md)** - Installation and your first diagram
  - Installation from source
  - Creating your first ERD or graph schema
  - Basic concepts and workflow
  - Common issues and solutions

### Core Documentation
- **[Schema Syntax Reference](schema-syntax.md)** - Complete syntax guide
  - File structure and comments
  - Table and column definitions
  - Node and edge definitions
  - Field attributes (pk, fk, unique, nullable, autoincrement, default)
  - Relationship and edge syntax and types
  - Validation rules

- **[Graph Database Concepts](graph-concepts.md)** - Understanding graph databases
  - When to use graphs vs relational databases
  - Nodes vs tables, edges vs relationships
  - Common graph patterns and migration strategies

- **[Data Types](data-types.md)** - All supported data types
  - Numeric types (int, float, double)
  - String types (str, string)
  - Boolean type (bool)
  - Date/time types (datetime, date, time)
  - Binary types (blob, tinyblob, largeblob)
  - Custom types

- **[Relationships Guide](relationships.md)** - Understanding relationships and edges
  - Relational relationships (One-to-Many, Many-to-One, etc.)
  - Graph edges (directed, bidirectional)
  - Self-referencing relationships
  - Junction tables and composite keys

### Examples
- **[Examples](examples.md)** - Real-world schema examples
  - **Relational Databases:**
    - Point of Sales System
    - E-commerce Platform
    - Blog Platform
    - School Management
  - **Graph Databases:**
    - Social Network
    - Knowledge Graph
    - Recommendation System

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/JustinVijar/FreeERD.git
cd FreeERD/free-erd
cargo build --release
```

### Basic Usage

```bash
# Check schema for errors
free-erd check schema.frd

# Generate SVG diagram
free-erd run schema.frd svg output.svg

# Get help
free-erd help
```

## 📖 Quick Reference

### Basic Schema Structure

```
#title "Schema Name"

table TableName {
  id: int [pk, autoincrement],
  field_name: datatype [attributes],
  another_field: datatype
}

// Relationships
Table1.field > Table2.field
```

### Relationship Operators

| Operator | Type | Visual | Example |
|----------|------|--------|---------|
| `>` | One-to-Many | `[1] -----> [M]` | `Users.id > Posts.user_id` |
| `<` | Many-to-One | `[M] <----- [1]` | `Posts.user_id < Users.id` |
| `<>` | Many-to-Many | `[M] <----> [M]` | `Students.id <> Courses.id` |
| `-` | One-to-One | `[1] ----- [1]` | `Users.id - Profiles.user_id` |

### Supported Data Types

| Type | Aliases | Use Case |
|------|---------|----------|
| `int` | `integer` | IDs, counters, whole numbers |
| `str` | `string` | Short text, names, emails |
| `bool` | `boolean` | True/false flags |
| `float` | - | Decimal numbers, prices |
| `double` | - | High-precision decimals |
| `datetime` | - | Date and time combined |
| `date` | - | Date only |
| `time` | - | Time only |
| `blob` | - | Binary data |

### Field Attributes

| Attribute | Meaning | Example |
|-----------|---------|---------|
| `[pk]` | Primary key | `id: int [pk]` |
| `[fk]` | Foreign key | `user_id: int [fk]` |
| `[unique]` | Unique values | `email: str [unique]` |
| `[nullable]` | Can be null | `bio: str [nullable]` |
| `[autoincrement]` | Auto-increment | `id: int [pk, autoincrement]` |
| `[default=value]` | Default value | `status: str [default="active"]` |

### Default Values

| Value | Example |
|-------|---------|
| `NOW` | `created_at: datetime [default=NOW]` |
| `TRUE` / `FALSE` | `is_active: bool [default=TRUE]` |
| `NULL` | `deleted_at: datetime [default=NULL]` |
| String | `status: str [default="pending"]` |
| Number | `quantity: int [default=0]` |

## 🎯 Common Tasks

### Create a New Schema

1. Create a `.frd` file
2. Add a title (optional but recommended)
3. Define tables with columns
4. Add relationships
5. Generate diagram

### Check for Errors

```bash
free-erd check schema.frd
```

### Generate SVG Diagram

```bash
free-erd run schema.frd svg diagram.svg
```

### View the Diagram

Open the generated `.svg` file in any web browser or SVG viewer.

## 💡 Tips and Best Practices

1. **Use Comments**: Document your schema with `//` comments
   ```
   // User authentication tables
   table Users { ... }
   ```

2. **Consistent Naming**: 
   - Tables: PascalCase (`Users`, `OrderItems`)
   - Columns: snake_case (`user_id`, `created_at`)

3. **Mark Foreign Keys**: Always use `[fk]` attribute
   ```
   user_id: int [fk]
   ```

4. **Use Auto-increment**: For primary keys
   ```
   id: int [pk, autoincrement]
   ```

5. **Set Defaults**: For common values
   ```
   created_at: datetime [default=NOW]
   is_active: bool [default=TRUE]
   ```

6. **Group Relationships**: Organize by table or feature
   ```
   // User relationships
   Users.id > Posts.user_id
   Users.id > Comments.user_id
   ```

## 🔧 CLI Commands

### `free-erd help`
Shows help menu with all available commands.

### `free-erd check <file>`
Validates the schema file and reports any errors.

### `free-erd run <file> svg [output]`
Generates an SVG diagram from the schema.
- If output is not specified, uses the input filename with `.svg` extension

### `free-erd about`
Displays information about FreeERD including version and license.

## 🎨 Visual Features

### Cardinality Labels

Relationships display clear cardinality indicators:
- **[1]** - White "1" on black background = "One" side
- **[M]** - White "M" on black background = "Many" side

### Force-Directed Layout

Tables are automatically positioned using the Fruchterman-Reingold algorithm:
- Minimizes edge crossings
- Distributes tables evenly
- Creates visually balanced diagrams
- Adapts to schema complexity

### Smart Line Routing

Relationship lines:
- Avoid overlapping with tables
- Use adaptive connection points
- Group parallel relationships
- Display clear visual paths

## 📝 Example Schema

```
#title "Blog Platform"

table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  created_at: datetime [default=NOW]
}

table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],
  post_title: str,
  content: str,
  published: bool [default=FALSE],
  created_at: datetime [default=NOW]
}

table Comments {
  id: int [pk, autoincrement],
  post_id: int [fk],
  user_id: int [fk],
  content: str,
  created_at: datetime [default=NOW]
}

// One-to-Many: One user has many posts
Users.id > Posts.user_id

// One-to-Many: One user has many comments
Users.id > Comments.user_id

// One-to-Many: One post has many comments
Posts.id > Comments.post_id
```

## 🔍 Troubleshooting

### Syntax Errors
- Check for missing commas between columns
- Ensure brackets are properly closed
- Verify table and column names are valid identifiers

### Validation Errors
- Ensure referenced tables exist
- Check that referenced columns exist
- Verify no duplicate table names
- Verify no duplicate column names within a table

### SVG Generation Issues
- Fix all validation errors first
- Check file permissions for output directory
- Ensure output filename ends with `.svg`

## 📞 Getting Help

- Review this documentation
- Check [Examples](examples.md) for similar schemas
- Run `free-erd help` for CLI usage
- Open an issue on GitHub

## 🎓 Learning Path

### Beginner
1. Read [Getting Started](getting-started.md)
2. Create a simple blog schema
3. Experiment with different relationship types

### Intermediate
1. Study [Schema Syntax Reference](schema-syntax.md)
2. Learn all [Data Types](data-types.md)
3. Create an e-commerce schema with multiple relationships

### Advanced
1. Master [Relationships Guide](relationships.md)
2. Implement many-to-many with junction tables
3. Use self-referencing relationships
4. Design complex schemas with composite keys

## 📄 License

FreeERD is licensed under the GNU General Public License v2.0 (GPL-2.0).

---

**Happy Diagramming! 🎨**
