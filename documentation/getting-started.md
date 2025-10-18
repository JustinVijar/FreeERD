# Getting Started with FreeERD

Welcome to FreeERD! This guide will help you get up and running with creating beautiful Entity Relationship Diagrams and graph database schemas.

## Table of Contents

1. [Installation](#installation)
2. [Your First Diagram](#your-first-diagram)
3. [Using the Interactive Window](#using-the-interactive-window)
4. [Exporting to SVG](#exporting-to-svg)
5. [Basic Concepts](#basic-concepts)
6. [Next Steps](#next-steps)

## Installation

### Prerequisites

- **Rust 1.70 or higher** - [Install Rust](https://www.rust-lang.org/tools/install)
- **Git** - For cloning the repository

**Note**: FreeERD is 100% pure Rust - no external dependencies like Graphviz required!

### Building from Source

1. **Clone the repository**:
```bash
git clone https://github.com/JustinVijar/FreeERD.git
cd FreeERD/free-erd
```

2. **Build the project**:
```bash
cargo build --release
```

This will create an optimized binary at `target/release/free-erd`.

3. **(Optional) Add to your PATH**:

**Linux/macOS**:
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH=$PATH:/path/to/FreeERD/free-erd/target/release

# Or copy to a directory in your PATH
sudo cp target/release/free-erd /usr/local/bin/
```

**Windows**:
- Add `C:\path\to\FreeERD\free-erd\target\release` to your PATH environment variable

4. **Verify installation**:
```bash
free-erd help
```

## Your First Diagram

Let's create both a relational database schema and a graph database schema.

### Relational Database Example

Create a file named `blog.frd`:

```
#title "Blog Database"

table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  password_hash: str,
  created_at: datetime [default=NOW]
}

table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],
  title: str,
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

// One-to-Many relationships
Users.id > Posts.user_id
Users.id > Comments.user_id
Posts.id > Comments.post_id
```

### Graph Database Example

Create a file named `social.frd`:

```
#title "Social Network"

node User {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique]
}

node Post {
  id: int [pk, autoincrement],
  title: str,
  content: str,
  created_at: datetime
}

node Comment {
  id: int [pk, autoincrement],
  text: str,
  created_at: datetime
}

// Complex edges with properties
edge AUTHORED (from: User, to: Post) {
  created_at: datetime
}

edge COMMENTED_ON (from: User, to: Comment) {
  timestamp: datetime
}

// Shorthand edges
User -[FOLLOWS]-> User
Comment <-[ATTACHED_TO]- Post
Post <-[TAGGED_WITH]- Tag
```

// Many-to-One (<): Many comments belong to one post
Comments.post_id < Posts.id

// One-to-One (-): One user has one profile
Users.id - UserProfiles.user_id

// Many-to-Many (<>): Posts have many tags, tags have many posts
Posts.id <> Tags.id
```

### Step 2: Check for Errors

Before opening the window, validate your schema:

```bash
free-erd check blog.frd
```

You should see:
```
📂 Reading file: blog.frd
🔍 Parsing...
✅ Parsing successful!

🔍 Validating schema...
✅ Schema is valid!

📊 Schema Statistics:
  • Tables: 6
  • Total Columns: 25
  • Relationships: 6
  • Primary Keys: 6
  • Foreign Keys: 6
```

### Step 3: Open in Interactive Window

```bash
free-erd run blog.frd
```

This opens an interactive window where you can:
- View your ERD with automatic layout
- Drag tables to reposition them
- Drag labels for better readability
- Zoom in/out with scroll wheel or +/- keys
- Pan with arrow keys
- Click tables to highlight their relationships

### Step 4: Customize and Export

1. **Arrange your diagram**:
   - Drag tables to desired positions
   - Move labels to avoid overlaps
   - Use zoom to focus on specific areas

2. **Export to SVG**:
   - Click the **Export** menu at the top
   - Select **SVG**
   - File saved as `export_YYYYMMDD_HHMMSS.svg`

## Using the Interactive Window

### Navigation Controls

- **Mouse Scroll** - Zoom in/out
- **+/- Keys** - Zoom in/out
- **Arrow Keys** - Pan the view
- **Left Click + Drag** - Move tables or labels
- **Left Click on Table** - Highlight the table and its relationships

### Window Layout

The window displays:
- **Title** - Schema title at the top (draggable)
- **Tables** - Entity boxes with columns and types
- **Relationship Lines** - Orthogonal lines connecting tables
- **Labels** - Relationship information beside lines
- **Status Bar** - Zoom level and controls at bottom
- **Menu Bar** - Export options at top

### Tips for Better Layouts

1. **Start with automatic layout** - Let the force-directed algorithm position tables
2. **Adjust spacing** - Drag tables apart if too crowded
3. **Group related tables** - Move related tables closer together
4. **Move labels** - Drag labels away from tables for clarity
5. **Use zoom** - Zoom out to see the full diagram, zoom in for details

## Exporting to SVG

### Export Process

1. Open your schema: `free-erd run myschema.frd`
2. Arrange elements as desired
3. Click **Export > SVG** in the menu
4. SVG file is created with timestamp

### Export Features

- **Exact positioning** - Preserves your custom layout
- **All elements included** - Tables, relationships, labels, title
- **Vector format** - Scales perfectly to any size
- **Publication ready** - Professional quality output

### File Naming

Exported files use the format: `export_YYYYMMDD_HHMMSS.svg`

Example: `export_20251019_143522.svg`

## Understanding the Visual Elements

### Tables

Each table shows:
- **Header** - Table name in blue
- **Columns** - Field name and data type
- **Attributes** - [pk], [fk], [unique], etc.

### Relationship Lines

- **Orthogonal routing** - Right-angle paths
- **Crow's feet** - Shows "many" side of relationships
- **Distributed connections** - Multiple relationships spread along borders

### Relationship Labels

Format: `[1:M] SourceTable.field:TargetTable.field`

- **[1:M]** - Relationship type in gray
- **Table.field** - Connected fields in white
- **Pointer line** - Connects label to relationship line

### Table Layout

Tables are automatically positioned using a force-directed algorithm that:
- Spreads tables evenly
- Minimizes line crossings
- Creates a balanced, readable layout

## Basic Concepts

### Tables

Tables represent entities in your database:

```
table TableName {
  column_name: datatype [attributes]
}
```

**Rules**:
- Table names use PascalCase (e.g., `Users`, `OrderItems`)
- Must start with a letter
- Can contain letters, numbers, underscores

### Columns

Columns define the fields in a table:

```
column_name: datatype [attribute1, attribute2]
```

**Rules**:
- Column names use snake_case (e.g., `user_id`, `created_at`)
- Must start with a letter or underscore
- Separate multiple attributes with commas

### Data Types

Common data types:
- `int` - Integer numbers
- `str` - Text strings
- `bool` - True/false values
- `float` - Decimal numbers
- `datetime` - Date and time
- `date` - Date only
- `time` - Time only

### Attributes

Common attributes:
- `[pk]` - Primary key
- `[fk]` - Foreign key
- `[unique]` - Unique constraint
- `[nullable]` - Can be null
- `[autoincrement]` - Auto-increment
- `[default=value]` - Default value

### Relationships

Relationships connect tables:

```
SourceTable.field OPERATOR TargetTable.field
```

**Operators**:
- `>` - One-to-Many (1:N)
- `<` - Many-to-One (N:1)
- `<>` - Many-to-Many (N:M)
- `-` - One-to-One (1:1)

### Comments

Use `//` for comments:

```
// This is a comment
table Users {  // Comment after code
  id: int [pk]  // Primary key
}
```

## Next Steps

Now that you've created your first diagram, explore:

1. **[Schema Syntax Reference](schema-syntax.md)** - Complete syntax guide
2. **[Data Types](data-types.md)** - All available data types
3. **[Relationships Guide](relationships.md)** - Detailed relationship documentation
4. **[Examples](examples.md)** - More complex examples

## Common Issues

### "Command not found: free-erd"

**Solution**: Make sure you've built the project and added it to your PATH, or use the full path:
```bash
./target/release/free-erd help
```

### Syntax Errors

**Problem**: Missing commas, unclosed brackets, etc.

**Solution**: Check the error message carefully. It will show the line and column where the error occurred.

Example error:
```
Error at line 5, column 10:
Expected ',', but found '}'
```

### Validation Errors

**Problem**: Referenced table or column doesn't exist.

**Solution**: Check that:
- All tables referenced in relationships exist
- All columns referenced in relationships exist
- Table and column names are spelled correctly

### SVG Not Generating

**Problem**: Validation errors prevent SVG generation.

**Solution**: Run `free-erd check schema.frd` first to see all errors, then fix them before generating the SVG.

## CLI Command Reference

### Check Schema

```bash
free-erd check <file>
```

Validates the schema without generating output.

### Generate SVG

```bash
free-erd run <file> svg [output]
```

Generates an SVG diagram. If `output` is not specified, uses the input filename with `.svg` extension.

### Get Help

```bash
free-erd help
```

Shows all available commands and usage information.

### About

```bash
free-erd about
```

Displays version, license, and project information.

## Tips for Success

1. **Start Simple**: Begin with 2-3 tables and add complexity gradually

2. **Use Auto-increment**: For primary keys
   ```
   id: int [pk, autoincrement]
   ```

3. **Mark Foreign Keys**: Always use `[fk]` attribute
   ```
   user_id: int [fk]
   ```

4. **Set Defaults**: For common values
   ```
   created_at: datetime [default=NOW]
   is_active: bool [default=TRUE]
   ```

5. **Add Comments**: Document your schema
   ```
   // User authentication and profile management
   table Users { ... }
   ```

6. **Check Often**: Run `free-erd check` frequently while building your schema

7. **Use Meaningful Names**: Choose clear, descriptive names for tables and columns

## Example Workflow

1. **Create** a `.frd` file with your schema
2. **Check** for errors: `free-erd check schema.frd`
3. **Fix** any errors reported
4. **Generate** SVG: `free-erd run schema.frd svg`
5. **View** the diagram in your browser
6. **Iterate** - refine your schema and regenerate

## Getting Help

- Review the [Schema Syntax Reference](schema-syntax.md)
- Check [Examples](examples.md) for similar schemas
- Read error messages carefully - they're designed to be helpful
- Open an issue on GitHub if you find a bug

---

**Ready to create more complex schemas?** Check out the [Examples](examples.md) section!
