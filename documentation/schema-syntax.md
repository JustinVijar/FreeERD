# Schema Syntax Reference

Complete reference for FreeERD schema definition language based on the actual implementation.

## Table of Contents

1. [File Structure](#file-structure)
2. [Comments](#comments)
3. [Title Declaration](#title-declaration)
4. [Table Definition](#table-definition)
5. [Column Syntax](#column-syntax)
6. [Node Definition](#node-definition)
7. [Edge Definition](#edge-definition)
8. [Data Types](#data-types)
9. [Field Attributes](#field-attributes)
10. [Relationship Syntax](#relationship-syntax)
11. [Edge Syntax](#edge-syntax)
12. [Validation Rules](#validation-rules)
13. [Complete Examples](#complete-examples)

## File Structure

A FreeERD schema file (`.frd`) consists of:

```
#title "Schema Name"

// Tables for relational databases
table TableName {
  column_name: datatype [attributes],
  another_column: datatype
}

// Nodes for graph databases
node NodeName {
  field_name: datatype [attributes],
  another_field: datatype
}

// Edges for graph databases
edge EdgeName (from: SourceNode, to: TargetNode) {
  property_name: datatype [attributes]
}

// Relationships (relational) or Edges (graph)
Table1.field OPERATOR Table2.field
SourceNode -[EDGE_NAME]-> TargetNode
```

**Order**:
1. Title (optional but recommended) - must use `#title` directive
2. Table and/or Node definitions
3. Relationships and/or Edges

## Comments

FreeERD supports single-line comments using `//`:

```
// This is a comment
table Users {  // Comment after code
  id: int [pk]  // Primary key column
}

// Relationship between users and posts
Users.id > Posts.user_id
```

**Rules**:
- Comments start with `//`
- Continue to the end of the line
- Can appear on their own line or after code
- Ignored during parsing

## Title Declaration

Every schema should start with a title using the `#title` directive:

```
#title "My Database Schema"
```

**Format**: `#title "string"`

**Rules**:
- Optional but recommended
- Must be a quoted string
- Uses the `#title` directive (note the `#` prefix)
- Appears once at the beginning of the file

**Examples**:
```
#title "E-commerce Platform"
#title "Blog Database"
#title "Point of Sales System"
```

## Table Definition

### Basic Syntax

```
table TableName {
  column_name: datatype [attributes],
  another_column: datatype
}
```

### Naming Rules

**Table Names**:
- Must start with a letter
- Can contain letters, numbers, underscores
- Case-sensitive
- Convention: Use PascalCase (e.g., `Users`, `OrderItems`, `ProductCategories`)

**Valid Examples**:
```
table Users { ... }
table OrderItems { ... }
table Product_Categories { ... }
```

**Invalid Examples**:
```
table 123Users { ... }      // Cannot start with number
table Order-Items { ... }   // Cannot contain hyphens
```

### Table Structure

```
table Products {
  id: int [pk, autoincrement],
  name: str [unique],
  price: float,
  stock: int [default=0],
  is_active: bool [default=TRUE],
  created_at: datetime [default=NOW]
}
```

**Rules**:
- Columns are separated by commas
- Last column can optionally have a trailing comma
- At least one column is recommended
- Curly braces `{}` enclose the column list

## Column Syntax

### Format

```
column_name: datatype [attribute1, attribute2, ...]
```

### Column Names

**Rules**:
- Must start with a letter or underscore
- Can contain letters, numbers, underscores
- Case-sensitive
- Convention: Use snake_case (e.g., `user_id`, `created_at`, `first_name`)

**Valid Examples**:
```
id: int
user_id: int
first_name: str
_internal_flag: bool
created_at: datetime
```

**Invalid Examples**:
```
123id: int           // Cannot start with number
user-id: int         // Cannot contain hyphens
first name: str      // Cannot contain spaces
```

### Column Components

1. **Column Name**: Identifier for the column
2. **Colon** (`:`) separator
3. **Data Type**: Type of data stored
4. **Attributes** (optional): Modifiers in square brackets

## Data Types

FreeERD supports the following data types:

### String Types
- `str` or `string` - Text strings

### Numeric Types
- `int` or `integer` - Integer numbers
- `float` - Floating-point numbers
- `double` - Double-precision floating-point

### Boolean Type
- `bool` or `boolean` - True/false values

### Date/Time Types
- `datetime` - Date and time combined
- `date` - Date only
- `time` - Time only

### Binary Types
- `blob` - Binary large object
- `tinyblob` - Small binary object
- `largeblob` - Large binary object

### Custom Types
- Any unrecognized type name becomes a custom type

**Examples**:
```
name: str
age: int
price: float
is_active: bool
created_at: datetime
birth_date: date
opening_time: time
profile_image: blob
custom_field: MyCustomType
```

## Field Attributes

Attributes modify field behavior in both tables and nodes. Multiple attributes are separated by commas.

### Primary Key `[pk]`

Marks a column as the primary key.

```
table Users {
  id: int [pk]
}
```

**Rules**:
- Only one primary key per table
- Typically used with `autoincrement`

### Foreign Key `[fk]`

Indicates a column references another table.

```
table Posts {
  user_id: int [fk]
}
```

**Note**: Must be paired with a relationship declaration.

### Unique `[unique]`

Ensures column values are unique across all rows.

```
table Users {
  email: str [unique],
  username: str [unique]
}
```

### Nullable `[nullable]`

Allows the column to contain NULL values.

```
table Users {
  middle_name: str [nullable],
  bio: str [nullable]
}
```

**Note**: By default, columns are NOT NULL unless marked as nullable.

### Auto-increment `[autoincrement]`

Automatically generates incrementing values.

```
table Products {
  id: int [pk, autoincrement]
}
```

**Typical Use**: Primary key columns.

### Default Value `[default=value]`

Sets a default value for the column.

```
table Orders {
  status: str [default="pending"],
  quantity: int [default=0],
  is_active: bool [default=TRUE],
  created_at: datetime [default=NOW]
}
```

**Supported Default Values**:
- `NOW` - Current timestamp (for datetime columns)
- `TRUE` / `FALSE` - Boolean values
- `NULL` - Null value
- `"string"` - String literals (quoted)
- `123` - Numeric literals (unquoted)

### Combining Attributes

Multiple attributes are separated by commas:

```
table Users {
  id: int [pk, autoincrement],
  email: str [unique, nullable],
  created_at: datetime [default=NOW]
}

table ProductSuppliers {
  product_id: int [pk, fk],
  supplier_id: int [pk, fk]
}
```

## Node Definition

Nodes represent entities in graph databases. They are similar to tables but used for graph structures.

### Basic Syntax

```
node NodeName {
  field_name: datatype [attributes],
  another_field: datatype
}
```

### Naming Rules

**Node Names**:
- Must start with a letter
- Can contain letters, numbers, underscores
- Case-sensitive
- Convention: Use PascalCase (e.g., `User`, `Post`, `Comment`, `ProductCategory`)

### Field Syntax

Node fields follow the same syntax as table columns:

```
field_name: datatype [attribute1, attribute2, ...]
```

**Examples**:
```
node User {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique, nullable],
  age: int [nullable],
  created_at: datetime [default=NOW]
}

node Product {
  id: int [pk, autoincrement],
  name: str,
  price: decimal,
  in_stock: bool [default=TRUE]
}
```

## Edge Definition

Edges represent relationships between nodes in graph databases.

### Complex Edge Syntax

```
edge EdgeName (from: SourceNode, to: TargetNode) {
  property_name: datatype [attributes]
}
```

### Shorthand Edge Syntax

```
SourceNode -[EDGE_NAME]-> TargetNode
SourceNode <-[EDGE_NAME]- TargetNode
SourceNode <-[EDGE_NAME]-> TargetNode
```

### Edge Types

- **Outgoing** (`->`): `SourceNode -[EDGE_NAME]-> TargetNode`
- **Incoming** (`<-`): `SourceNode <-[EDGE_NAME]- TargetNode`
- **Bidirectional** (`<->`): `SourceNode <-[EDGE_NAME]-> TargetNode`

### Examples

```
edge FOLLOWS (from: User, to: User) {
  since: date,
  is_mutual: bool [default=FALSE]
}

edge AUTHORED (from: User, to: Post) {
  created_at: datetime
}

// Shorthand equivalents
User -[FOLLOWS]-> User
User -[AUTHORED]-> Post
```

## Relationship Syntax
```

## Relationship Syntax

### Format

```
SourceTable.source_column OPERATOR TargetTable.target_column
```

### Operators

| Operator | Type | Meaning | Visual |
|----------|------|---------|--------|
| `>` | One-to-Many | One source has many targets | `[1] -----> [M]` |
| `<` | Many-to-One | Many sources to one target | `[M] <----- [1]` |
| `<>` | Many-to-Many | Many to many | `[M] <----> [M]` |
| `-` | One-to-One | One to one | `[1] ----- [1]` |

### Examples

#### One-to-Many (`>`)
```
// One user has many posts
Users.id > Posts.user_id
```

#### Many-to-One (`<`)
```
// Many posts belong to one user
Posts.user_id < Users.id
```

#### Many-to-Many (`<>`)
```
// Students enroll in many courses, courses have many students
Students.id <> Courses.id
```

#### One-to-One (`-`)
```
// One user has one profile
Users.id - UserProfiles.user_id
```

### Self-Referencing Relationships

Tables can reference themselves:

```
table Categories {
  id: int [pk, autoincrement],
  name: str,
  parent_id: int [fk, nullable]
}

// Category hierarchy
Categories.id > Categories.parent_id
```

## Edge Syntax

Edges connect nodes in graph databases and can be defined using shorthand syntax.

### Shorthand Syntax

```
SourceNode -[EDGE_NAME]-> TargetNode
SourceNode <-[EDGE_NAME]- TargetNode
SourceNode <-[EDGE_NAME]-> TargetNode
```

### Edge Name Rules

- Must be uppercase letters and underscores
- Convention: Use SCREAMING_SNAKE_CASE (e.g., `FOLLOWS`, `AUTHORED`, `BELONGS_TO`)

### Direction Indicators

- `->` **Outgoing edge**: `SourceNode -[EDGE_NAME]-> TargetNode`
- `<-` **Incoming edge**: `SourceNode <-[EDGE_NAME]- TargetNode`
- `<->` **Bidirectional edge**: `SourceNode <-[EDGE_NAME]-> TargetNode`

### Examples

```
User -[FOLLOWS]-> User
User <-[AUTHORED]- Post
User <-[LIKES]-> Post
Post <-[TAGGED_WITH]- Tag
Category <-[RELATED_TO]-> Category
```

### Self-Referencing Edges

Nodes can reference themselves:

```
User <-[FRIENDS_WITH]-> User
Post <-[REPLIES_TO]-> Post
Comment <-[NESTED_IN]-> Comment
```

## Validation Rules

FreeERD validates your schema and reports errors for:

### 1. Duplicate Table Names

```
table Users { ... }
table Users { ... }  // ❌ Error: Duplicate table name
```

### 2. Duplicate Column Names

```
table Users {
  id: int [pk],
  id: int  // ❌ Error: Duplicate column name
}
```

### 3. Invalid Relationship References

```
// ❌ Error: Table 'NonExistent' does not exist
Users.id > NonExistent.user_id

// ❌ Error: Column 'invalid_field' does not exist in table 'Users'
Users.invalid_field > Posts.user_id
```

### 4. Multiple Primary Keys

```
table Users {
  id: int [pk],
  email: str [pk]  // ❌ Error: Multiple primary keys
}
```

**Note**: Composite primary keys are supported using multiple `[pk]` attributes:
```
table OrderItems {
  order_id: int [pk, fk],
  product_id: int [pk, fk]
}
```

## Complete Examples

### Relational Database Example

```
#title "E-commerce Database"

// User management
table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  password_hash: str,
  is_active: bool [default=TRUE],
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW]
}

// Product catalog
table Categories {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable],
  parent_id: int [fk, nullable]
}

table Products {
  id: int [pk, autoincrement],
  category_id: int [fk],
  name: str [unique],
  description: str [nullable],
  price: float,
  stock: int [default=0],
  is_active: bool [default=TRUE],
  created_at: datetime [default=NOW]
}

// Orders
table Orders {
  id: int [pk, autoincrement],
  user_id: int [fk],
  order_date: datetime [default=NOW],
  status: str [default="pending"],
  total_amount: float
}

table OrderItems {
  id: int [pk, autoincrement],
  order_id: int [fk],
  product_id: int [fk],
  quantity: int [default=1],
  unit_price: float
}

// Relationships
// Category hierarchy (self-referencing)
Categories.id > Categories.parent_id

// One-to-Many relationships
Categories.id > Products.category_id
Users.id > Orders.user_id
Orders.id > OrderItems.order_id
Products.id > OrderItems.product_id

// Many-to-One (alternative syntax)
OrderItems.order_id < Orders.id
```

### Graph Database Example

```
#title "Social Network Graph"

// Node definitions
node User {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  full_name: str,
  age: int [nullable],
  created_at: datetime
}

node Post {
  id: int [pk, autoincrement],
  title: str,
  content: str,
  published_at: datetime,
  view_count: int [default=0]
}

node Comment {
  id: int [pk, autoincrement],
  text: str,
  created_at: datetime
}

node Tag {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable]
}

// Complex edges with properties
edge FOLLOWS (from: User, to: User) {
  since: date,
  is_mutual: bool [default=FALSE]
}

edge AUTHORED (from: User, to: Post) {
  created_at: datetime,
  is_draft: bool [default=FALSE]
}

edge COMMENTED_ON (from: User, to: Comment) {
  timestamp: datetime
}

// Shorthand edges
User -[LIKES]-> Post
User -[BOOKMARKS]-> Post
User -[REPORTS]-> Post

Comment <-[ATTACHED_TO]- Post
Post <-[TAGGED_WITH]- Tag

// Self-referencing edges
User <-[FRIENDS_WITH]-> User
Post <-[REPLIES_TO]-> Post
```

## Best Practices

### 1. Consistent Naming

```
// ✅ Good
table Users { ... }
table OrderItems { ... }
user_id: int
created_at: datetime

// ❌ Avoid
table users { ... }
table order_items { ... }
userId: int
createdAt: datetime
```

### 2. Always Use Primary Keys

```
// ✅ Good
table Products {
  id: int [pk, autoincrement],
  name: str
}

// ❌ Avoid
table Products {
  name: str  // No primary key
}
```

### 3. Mark Foreign Keys

```
// ✅ Good
table Posts {
  user_id: int [fk]
}

// ❌ Avoid
table Posts {
  user_id: int  // Should be marked as [fk]
}
```

### 4. Use Auto-increment for IDs

```
// ✅ Good
id: int [pk, autoincrement]

// ❌ Less ideal
id: int [pk]
```

### 5. Set Sensible Defaults

```
// ✅ Good
created_at: datetime [default=NOW]
is_active: bool [default=TRUE]
quantity: int [default=0]

// ❌ Missing defaults
created_at: datetime
is_active: bool
quantity: int
```

### 6. Document with Comments

```
// ✅ Good
// User authentication and profile management
table Users { ... }

// One-to-Many: One user has many posts
Users.id > Posts.user_id
```

## See Also

- [Data Types Reference](data-types.md)
- [Relationships Guide](relationships.md)
- [Examples](examples.md)
- [Getting Started](getting-started.md)
