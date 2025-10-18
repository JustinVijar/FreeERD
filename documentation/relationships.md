# Relationships and Edges Guide

Complete guide to defining and understanding relationships in relational databases and edges in graph databases with FreeERD.

## Table of Contents

1. [Overview](#overview)
2. [Relational Relationships](#relational-relationships)
3. [Graph Edges](#graph-edges)
4. [Cardinality Notation](#cardinality-notation)
5. [Self-Referencing](#self-referencing)
6. [Many-to-Many](#many-to-many)
7. [Composite Keys](#composite-keys)
8. [Best Practices](#best-practices)
9. [Common Patterns](#common-patterns)

## Overview

FreeERD supports both traditional relational database relationships and modern graph database edges. Relationships connect tables using foreign keys, while edges connect nodes with rich properties.

### Relational Syntax

```
SourceTable.source_column OPERATOR TargetTable.target_column
```

### Graph Syntax

```
SourceNode -[EDGE_NAME]-> TargetNode
SourceNode <-[EDGE_NAME]- TargetNode
SourceNode <-[EDGE_NAME]-> TargetNode
```

## Relational Relationships

### One-to-Many (1:N)

**Operator**: `>`

**Meaning**: One record in the source table relates to many records in the target table.

**Visual**: `[1] SourceTable -----> TargetTable [M]`

**Example**:
```
table Users {
  id: int [pk, autoincrement],
  username: str [unique]
}

table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],
  title: str
}

// One-to-Many: One user has many posts
Users.id > Posts.user_id
```

**Explanation**: One user can have many posts, but each post belongs to one user.

**Common Use Cases**:
- User → Posts
- Category → Products
- Department → Employees
- Customer → Orders
- Post → Comments

---

### Many-to-One (N:1)

**Operator**: `<`

**Meaning**: Many records in the source table relate to one record in the target table.

**Visual**: `[M] SourceTable <----- TargetTable [1]`

**Example**:
```
table Comments {
  id: int [pk, autoincrement],
  post_id: int [fk],
  content: str
}

table Posts {
  id: int [pk, autoincrement],
  title: str
}

// Many-to-One: Many comments belong to one post
Comments.post_id < Posts.id
```

**Explanation**: Many comments belong to one post.

**Note**: This is the inverse of One-to-Many. Use whichever reads more naturally.

**Equivalent Relationships**:
```
// These are equivalent:
Users.id > Posts.user_id
Posts.user_id < Users.id
```

---

### Many-to-Many (N:M)

**Operator**: `<>`

**Meaning**: Many records in the source table relate to many records in the target table.

**Visual**: `[M] SourceTable <-----> TargetTable [M]`

**Example**:
```
table Students {
  id: int [pk, autoincrement],
  name: str
}

table Courses {
  id: int [pk, autoincrement],
  name: str
}

// Many-to-Many: Students enroll in many courses
Students.id <> Courses.id
```

**Explanation**: A student can enroll in many courses, and a course can have many students.

**Implementation**: Typically requires a junction table (see [Many-to-Many Relationships](#many-to-many-relationships))

**Common Use Cases**:
- Students ↔ Courses
- Products ↔ Tags
- Users ↔ Roles
- Authors ↔ Books
- Posts ↔ Categories

---

### One-to-One (1:1)

**Operator**: `-`

**Meaning**: One record in the source table relates to exactly one record in the target table.

**Visual**: `[1] SourceTable ----- TargetTable [1]` (dashed line)

**Example**:
```
table Users {
  id: int [pk, autoincrement],
  username: str [unique]
}

table UserProfiles {
  id: int [pk, autoincrement],
  user_id: int [unique, fk],
  bio: str [nullable],
  avatar_url: str [nullable]
}

// One-to-One: One user has one profile
Users.id - UserProfiles.user_id
```

**Explanation**: Each user has exactly one profile, and each profile belongs to exactly one user.

**Note**: The foreign key column should be marked as `[unique]` to enforce the one-to-one constraint.

**Common Use Cases**:
- User → Profile
- Country → Capital
- Employee → Desk
- Product → ProductDetails

## Graph Edges

Graph databases use edges to connect nodes. Edges can be directed, bidirectional, and can have rich properties.

### Edge Syntax

#### Shorthand Syntax
```
SourceNode -[EDGE_NAME]-> TargetNode    // Outgoing edge
SourceNode <-[EDGE_NAME]- TargetNode    // Incoming edge
SourceNode <-[EDGE_NAME]-> TargetNode   // Bidirectional edge
```

#### Complex Syntax
```
edge EdgeName (from: SourceNode, to: TargetNode) {
  property_name: datatype [attributes]
}
```

### Edge Types

#### Directed Edges

**Outgoing (`->`)**: `SourceNode -[FOLLOWS]-> TargetNode`
- Direction matters: A follows B, but B doesn't necessarily follow A
- Visual: Arrow pointing right

**Incoming (`<-`)**: `SourceNode <-[AUTHORED]- TargetNode`
- Direction matters: Post is authored by User
- Visual: Arrow pointing left

#### Bidirectional Edges

**Bidirectional (`<->`)**: `SourceNode <-[FRIENDS_WITH]-> TargetNode`
- Mutual relationship: A and B are friends
- Visual: Arrows pointing both ways

### Examples

```
node User {
  id: int [pk],
  name: str
}

node Post {
  id: int [pk],
  title: str
}

// Directed edges
User -[FOLLOWS]-> User
User -[AUTHORED]-> Post

// Bidirectional edges
User <-[FRIENDS_WITH]-> User
User <-[MARRIED_TO]-> User

// Complex edges with properties
edge PURCHASED (from: User, to: Product) {
  quantity: int,
  purchase_date: datetime,
  rating: int [nullable]
}
```

### Edge Properties

Unlike relational foreign keys, edges can store rich relationship data:

```
edge FOLLOWS (from: User, to: User) {
  since: date,
  closeness: str,  // "close", "acquaintance"
  notifications: bool [default=TRUE]
}
```

## Cardinality Notation

FreeERD displays cardinality labels on relationship lines using black boxes with white text.

### Visual Indicators

- **[1]** - Black box with white "1" = "One" side
- **[M]** - Black box with white "M" = "Many" side

### Examples

#### One-to-Many
```
Users.id > Posts.user_id
```
Displays as: `[1] Users -----> Posts [M]`

#### Many-to-One
```
Comments.post_id < Posts.id
```
Displays as: `[M] Comments <----- Posts [1]`

#### Many-to-Many
```
Students.id <> Courses.id
```
Displays as: `[M] Students <-----> Courses [M]`

#### One-to-One
```
Users.id - Profiles.user_id
```
Displays as: `[1] Users ----- Profiles [1]` (dashed line)

## Self-Referencing Relationships

Tables can have relationships with themselves.

### Example: Employee Hierarchy

```
table Employees {
  id: int [pk, autoincrement],
  name: str,
  manager_id: int [fk, nullable]
}

// One-to-Many: One employee manages many employees
Employees.id > Employees.manager_id
```

**Explanation**: An employee can manage many other employees (one-to-many to itself).

### Example: Category Tree

```
table Categories {
  id: int [pk, autoincrement],
  name: str [unique],
  parent_id: int [fk, nullable]
}

// Category hierarchy
Categories.id > Categories.parent_id
```

### Use Cases

- Organizational hierarchies
- Category/folder trees
- Comment threads (parent comments)
- Social connections (followers)
- File systems

**Best Practice**: Mark the self-referencing foreign key as `[nullable]` for root nodes.

## Many-to-Many Relationships

Many-to-many relationships typically require a junction (or bridge) table.

### Method 1: Simplified Notation

```
Students.id <> Courses.id
```

FreeERD will display this as a many-to-many relationship with `[M]` on both sides.

### Method 2: Explicit Junction Table

```
table Students {
  id: int [pk, autoincrement],
  name: str
}

table Courses {
  id: int [pk, autoincrement],
  name: str
}

table Enrollments {
  student_id: int [pk, fk],
  course_id: int [pk, fk],
  enrolled_date: datetime [default=NOW],
  grade: str [nullable]
}

// One-to-Many: One student has many enrollments
Students.id > Enrollments.student_id

// One-to-Many: One course has many enrollments
Courses.id > Enrollments.course_id
```

**Advantages of Explicit Junction Table**:
- Can store additional data (enrollment date, grade, etc.)
- More accurate representation of database structure
- Better for complex relationships
- Allows composite primary keys

### Common Junction Table Patterns

#### Product Tags
```
table Products {
  id: int [pk, autoincrement],
  name: str
}

table Tags {
  id: int [pk, autoincrement],
  name: str [unique]
}

table ProductTags {
  product_id: int [pk, fk],
  tag_id: int [pk, fk],
  created_at: datetime [default=NOW]
}

Products.id > ProductTags.product_id
Tags.id > ProductTags.tag_id
```

#### User Roles
```
table Users {
  id: int [pk, autoincrement],
  username: str [unique]
}

table Roles {
  id: int [pk, autoincrement],
  name: str [unique]
}

table UserRoles {
  user_id: int [pk, fk],
  role_id: int [pk, fk],
  assigned_at: datetime [default=NOW]
}

Users.id > UserRoles.user_id
Roles.id > UserRoles.role_id
```

## Composite Keys

FreeERD supports composite primary keys using multiple `[pk]` attributes.

### Example

```
table OrderItems {
  order_id: int [pk, fk],
  product_id: int [pk, fk],
  quantity: int,
  unit_price: float
}

table Orders {
  id: int [pk, autoincrement]
}

table Products {
  id: int [pk, autoincrement]
}

Orders.id > OrderItems.order_id
Products.id > OrderItems.product_id
```

**Note**: Both `order_id` and `product_id` are marked as `[pk]`, creating a composite primary key.

## Best Practices

### 1. Use Descriptive Column Names

✅ **Good**:
```
Users.id > Posts.user_id
```

❌ **Avoid**:
```
Users.id > Posts.uid
```

### 2. Consistent Foreign Key Naming

Use a consistent pattern for foreign keys:

```
// Pattern: {table_name}_id
user_id, post_id, category_id, product_id

// Or: {table_name}_{column_name}
user_account_id, post_content_id
```

### 3. Mark Foreign Keys

Always mark foreign key columns with `[fk]`:

```
table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],  // Clearly marked
  category_id: int [fk]
}
```

### 4. Use Unique for One-to-One

For one-to-one relationships, mark the foreign key as unique:

```
table UserProfiles {
  user_id: int [unique, fk]  // Enforces one-to-one
}
```

### 5. Group Related Relationships

```
// User relationships
Users.id > Posts.user_id
Users.id > Comments.user_id
Users.id > Likes.user_id

// Post relationships
Posts.id > Comments.post_id
Posts.id > Likes.post_id
```

### 6. Use Nullable for Optional Relationships

```
table Posts {
  id: int [pk, autoincrement],
  category_id: int [fk, nullable]  // Posts can be uncategorized
}

table Employees {
  id: int [pk, autoincrement],
  manager_id: int [fk, nullable]  // CEO has no manager
}
```

### 7. Document Complex Relationships

```
// Many-to-many: Students can enroll in multiple courses
// Junction table stores enrollment date and grade
Students.id > Enrollments.student_id
Courses.id > Enrollments.course_id
```

## Common Patterns

### Blog Platform

```
// One-to-Many: One user has many posts
Users.id > Posts.user_id

// One-to-Many: One post has many comments
Posts.id > Comments.post_id

// One-to-Many: One user has many comments
Users.id > Comments.user_id

// Many-to-Many: Posts have many tags
Posts.id <> Tags.id
```

### E-commerce

```
// One-to-Many: One customer has many orders
Customers.id > Orders.customer_id

// One-to-Many: One order has many order items
Orders.id > OrderItems.order_id

// One-to-Many: One product appears in many order items
Products.id > OrderItems.product_id

// One-to-Many: One category has many products
Categories.id > Products.category_id

// Self-referencing: Category hierarchy
Categories.id > Categories.parent_id
```

### Social Media

```
// One-to-Many: One user has many posts
Users.id > Posts.user_id

// One-to-Many: One post has many likes
Posts.id > Likes.post_id

// One-to-Many: One user has many likes
Users.id > Likes.user_id

// Many-to-Many: User friendships (self-referencing)
Users.id > Friendships.user_id
Users.id > Friendships.friend_id
```

## Validation

FreeERD validates relationships and will report errors for:

### Invalid Table References
```
// ❌ Error: Table 'NonExistent' does not exist
NonExistent.id > Posts.user_id
```

### Invalid Column References
```
// ❌ Error: Column 'invalid_field' does not exist in table 'Users'
Users.invalid_field > Posts.user_id
```

### Mismatched Types (Best Practice)

While FreeERD doesn't enforce type matching, it's best practice to match types:

✅ **Good**:
```
Users.id > Posts.user_id  // Both int
```

❌ **Avoid**:
```
Users.id > Posts.user_code  // int to str
```

## Complete Example

```
title "Social Media Platform"

table Users {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  created_at: datetime [default=NOW]
}

table UserProfiles {
  id: int [pk, autoincrement],
  user_id: int [unique, fk],
  bio: str [nullable],
  avatar_url: str [nullable]
}

table Posts {
  id: int [pk, autoincrement],
  user_id: int [fk],
  content: str,
  created_at: datetime [default=NOW]
}

table Comments {
  id: int [pk, autoincrement],
  post_id: int [fk],
  user_id: int [fk],
  parent_comment_id: int [fk, nullable],
  content: str,
  created_at: datetime [default=NOW]
}

table Likes {
  id: int [pk, autoincrement],
  post_id: int [fk],
  user_id: int [fk],
  created_at: datetime [default=NOW]
}

table Followers {
  follower_id: int [pk, fk],
  following_id: int [pk, fk],
  created_at: datetime [default=NOW]
}

table Hashtags {
  id: int [pk, autoincrement],
  tag: str [unique]
}

table PostHashtags {
  post_id: int [pk, fk],
  hashtag_id: int [pk, fk]
}

// One-to-One: One user has one profile
Users.id - UserProfiles.user_id

// One-to-Many: One user has many posts
Users.id > Posts.user_id

// One-to-Many: One user has many comments
Users.id > Comments.user_id

// One-to-Many: One post has many comments
Posts.id > Comments.post_id

// One-to-Many: One post has many likes
Posts.id > Likes.post_id

// One-to-Many: One user has many likes
Users.id > Likes.user_id

// Self-referencing: Comment threads
Comments.id > Comments.parent_comment_id

// Many-to-Many: User followers (self-referencing)
Users.id > Followers.follower_id
Users.id > Followers.following_id

// Many-to-Many: Posts and hashtags
Posts.id > PostHashtags.post_id
Hashtags.id > PostHashtags.hashtag_id
```

## See Also

- [Schema Syntax Reference](schema-syntax.md)
- [Data Types](data-types.md)
- [Examples](examples.md)
- [Getting Started](getting-started.md)
