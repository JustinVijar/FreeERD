# Getting Started with FreeERD

Welcome to FreeERD! This guide will help you get up and running with creating beautiful Entity Relationship Diagrams.

## Table of Contents

1. [Installation](#installation)
2. [Your First Diagram](#your-first-diagram)
3. [Understanding the Output](#understanding-the-output)
4. [Basic Concepts](#basic-concepts)
5. [Next Steps](#next-steps)

## Installation

### Prerequisites

- **Rust 1.70 or higher** - [Install Rust](https://www.rust-lang.org/tools/install)
- **Git** - For cloning the repository

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

Let's create a simple blog database schema.

### Step 1: Create a Schema File

Create a file named `blog.frd`:

```
title "Blog Database"

table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  password_hash: str,
  created_at: datetime [default=NOW]
}

table UserProfiles {
  id: int [pk, autoincrement],
  user_id: int [unique, fk],
  bio: str [nullable],
  avatar_url: str [nullable],
  created_at: datetime [default=NOW]
}

table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],
  title: str,
  content: str,
  published: bool [default=FALSE],
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW]
}

table Comments {
  id: int [pk, autoincrement],
  post_id: int [fk],
  user_id: int [fk],
  content: str,
  created_at: datetime [default=NOW]
}

table Tags {
  id: int [pk, autoincrement],
  name: str [unique]
}

table PostTags {
  post_id: int [pk, fk],
  tag_id: int [pk, fk]
}

// One-to-Many (>): One user has many posts
Users.id > Posts.user_id

// One-to-Many (>): One user has many comments
Users.id > Comments.user_id

// One-to-Many (>): One post has many comments
Posts.id > Comments.post_id

// Many-to-One (<): Many comments belong to one post
Comments.post_id < Posts.id

// One-to-One (-): One user has one profile
Users.id - UserProfiles.user_id

// Many-to-Many (<>): Posts have many tags, tags have many posts
Posts.id <> Tags.id
```

### Step 2: Check for Errors

Before generating the diagram, validate your schema:

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
  • Tables: 3
  • Total Columns: 11
  • Relationships: 3
  • Primary Keys: 3
  • Foreign Keys: 3
```

### Step 3: Generate the Diagram

```bash
free-erd run blog.frd svg blog.svg
```

You should see:
```
📂 Reading file: blog.frd
🔍 Parsing...
✅ Parsing successful!
🔍 Validating schema...
✅ Schema is valid!
🎨 Generating SVG diagram...
📐 Auto-sizing canvas: 2600x1900 for 3 tables (organic layout)
🔄 Running Fruchterman-Reingold layout algorithm...
✅ Force-directed layout complete.
💾 Writing to: blog.svg
✅ SVG diagram created successfully!

📊 Output: blog.svg
```

### Step 4: View Your Diagram

Open `blog.svg` in your web browser or any SVG viewer. You should see:
- Three tables (Users, Posts, Comments)
- Relationship lines connecting them
- Cardinality labels ([1] and [M]) showing relationship types
- Automatically positioned tables

## Understanding the Output

### Cardinality Labels

FreeERD displays cardinality on relationship lines:

- **[1]** - Black box with white "1" = "One" side
- **[M]** - Black box with white "M" = "Many" side

### Relationship Lines

- **Solid lines** - Most relationships
- **Dashed lines** - One-to-One relationships (using `-` operator)

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
