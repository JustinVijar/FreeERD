# Data Types Reference

Complete guide to all data types supported by FreeERD based on the actual implementation.

## Table of Contents

1. [Overview](#overview)
2. [String Types](#string-types)
3. [Numeric Types](#numeric-types)
4. [Boolean Type](#boolean-type)
5. [Date and Time Types](#date-and-time-types)
6. [Binary Types](#binary-types)
7. [Custom Types](#custom-types)
8. [Type Selection Guide](#type-selection-guide)

## Overview

FreeERD supports common database data types that map to standard SQL types. The parser recognizes both full names and common aliases.

### Syntax

```
column_name: datatype [attributes]
```

## String Types

### `str` / `string`

**Description**: Text strings for names, titles, descriptions, etc.

**Aliases**: `str`, `string`

**Use Cases**:
- Names (first_name, last_name)
- Titles and headings
- Usernames and emails
- Short descriptions
- URLs and paths
- Codes and identifiers

**Examples**:
```
table Users {
  username: str,
  email: string,
  first_name: str,
  last_name: str,
  phone: str,
  address: string [nullable]
}

table Products {
  name: str [unique],
  sku: str [unique],
  description: str [nullable]
}
```

**SQL Mapping**: `VARCHAR`, `TEXT`, `CHAR`

**Best Practices**:
- Use for short to medium-length text
- Mark as `[unique]` for identifiers
- Mark as `[nullable]` if optional

## Numeric Types

### `int` / `integer`

**Description**: Integer numbers (whole numbers without decimals)

**Aliases**: `int`, `integer`

**Use Cases**:
- Primary keys
- Foreign keys
- Counters and quantities
- IDs and references
- Years, ages, counts

**Examples**:
```
table Products {
  id: int [pk, autoincrement],
  category_id: int [fk],
  stock_quantity: int [default=0],
  min_stock_level: int [default=5],
  year_released: int [nullable]
}

table Orders {
  id: integer [pk, autoincrement],
  user_id: integer [fk],
  item_count: integer [default=0]
}
```

**SQL Mapping**: `INTEGER`, `INT`, `BIGINT`, `SMALLINT`

**Best Practices**:
- Always use for IDs and keys
- Use `[autoincrement]` for primary keys
- Set `[default=0]` for counters

---

### `float`

**Description**: Single-precision floating-point numbers

**Use Cases**:
- Prices and monetary values
- Measurements
- Percentages
- Ratings
- Coordinates

**Examples**:
```
table Products {
  price: float,
  cost: float,
  weight: float [nullable],
  discount_percent: float [default=0]
}

table Reviews {
  rating: float,
  score: float [default=0.0]
}
```

**SQL Mapping**: `FLOAT`, `REAL`

**Note**: For monetary values, consider using appropriate decimal precision in your database implementation.

---

### `double`

**Description**: Double-precision floating-point numbers

**Use Cases**:
- High-precision calculations
- Scientific data
- Geographic coordinates
- Large decimal values

**Examples**:
```
table Locations {
  latitude: double,
  longitude: double,
  altitude: double [nullable]
}

table Measurements {
  precise_value: double,
  calculated_result: double
}
```

**SQL Mapping**: `DOUBLE`, `DOUBLE PRECISION`, `NUMERIC`, `DECIMAL`

**Best Practices**:
- Use when precision is critical
- Prefer over `float` for financial calculations

## Boolean Type

### `bool` / `boolean`

**Description**: True/false values

**Aliases**: `bool`, `boolean`

**Use Cases**:
- Flags and switches
- Status indicators
- Permissions
- Active/inactive states
- Yes/no questions

**Examples**:
```
table Users {
  is_active: bool [default=TRUE],
  is_admin: bool [default=FALSE],
  email_verified: bool [default=FALSE],
  newsletter_subscribed: bool [default=TRUE]
}

table Products {
  is_available: bool [default=TRUE],
  is_featured: bool [default=FALSE],
  requires_shipping: bool [default=TRUE]
}
```

**SQL Mapping**: `BOOLEAN`, `BOOL`, `TINYINT(1)`, `BIT`

**Default Values**:
- `[default=TRUE]` - Defaults to true
- `[default=FALSE]` - Defaults to false

**Best Practices**:
- Always set a default value
- Use descriptive names (is_*, has_*, can_*)
- Avoid nullable booleans (use default instead)

## Date and Time Types

### `datetime`

**Description**: Date and time combined

**Use Cases**:
- Creation timestamps
- Update timestamps
- Event times
- Scheduled actions
- Deadlines with time

**Examples**:
```
table Posts {
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW],
  published_at: datetime [nullable],
  deleted_at: datetime [nullable]
}

table Events {
  start_time: datetime,
  end_time: datetime,
  registration_deadline: datetime
}
```

**SQL Mapping**: `DATETIME`, `TIMESTAMP`

**Format**: Typically `YYYY-MM-DD HH:MM:SS`

**Best Practices**:
- Use `[default=NOW]` for timestamps
- Use `created_at` and `updated_at` for tracking
- Mark as `[nullable]` for optional dates

---

### `date`

**Description**: Date only (no time component)

**Use Cases**:
- Birth dates
- Due dates
- Start/end dates
- Anniversaries
- Calendar dates

**Examples**:
```
table Users {
  birth_date: date [nullable],
  registration_date: date [default=NOW]
}

table Projects {
  start_date: date,
  end_date: date [nullable],
  deadline: date
}

table Employees {
  hire_date: date [default=NOW],
  termination_date: date [nullable]
}
```

**SQL Mapping**: `DATE`

**Format**: Typically `YYYY-MM-DD`

**Best Practices**:
- Use for day-level precision
- Don't use for timestamps (use `datetime`)

---

### `time`

**Description**: Time only (no date component)

**Use Cases**:
- Business hours
- Recurring event times
- Time slots
- Duration

**Examples**:
```
table BusinessHours {
  opening_time: time,
  closing_time: time
}

table Appointments {
  appointment_time: time,
  duration_minutes: int
}

table Schedule {
  start_time: time,
  end_time: time
}
```

**SQL Mapping**: `TIME`

**Format**: Typically `HH:MM:SS`

**Best Practices**:
- Use for time-of-day values
- Combine with `date` if you need both

## Binary Types

### `blob`

**Description**: Binary large object for storing binary data

**Use Cases**:
- File storage
- Images
- Documents
- Serialized data

**Examples**:
```
table Documents {
  file_data: blob,
  thumbnail: blob [nullable]
}
```

**SQL Mapping**: `BLOB`, `BINARY`, `VARBINARY`

---

### `tinyblob`

**Description**: Small binary object

**Use Cases**:
- Small files
- Icons
- Thumbnails

**Examples**:
```
table Icons {
  icon_data: tinyblob
}
```

**SQL Mapping**: `TINYBLOB`, `BINARY(n)` where n is small

---

### `largeblob`

**Description**: Large binary object

**Use Cases**:
- Large files
- Videos
- High-resolution images

**Examples**:
```
table Media {
  video_data: largeblob,
  high_res_image: largeblob
}
```

**SQL Mapping**: `LONGBLOB`, `MEDIUMBLOB`

## Custom Types

FreeERD allows custom type names for database-specific types.

**Examples**:
```
table CustomData {
  json_field: JSON,
  xml_field: XML,
  uuid_field: UUID,
  enum_field: ENUM,
  special_type: MyCustomType
}
```

**Note**: Custom types are passed through as-is to the diagram. Ensure they're valid for your target database.

## Type Selection Guide

### When to use `int`

✅ **Use for**:
- IDs and keys
- Whole number quantities
- Counters
- Years (as numbers)
- Counts and totals

❌ **Don't use for**:
- Prices (use `float` or `double`)
- Phone numbers (use `str`)
- ZIP codes (use `str`)
- Decimals (use `float` or `double`)

### When to use `float` or `double`

✅ **Use for**:
- Monetary values
- Measurements
- Percentages
- Coordinates
- Ratings
- Scientific calculations

❌ **Don't use for**:
- IDs (use `int`)
- Exact counting (use `int`)
- True/false (use `bool`)

**float vs double**:
- Use `float` for general decimal numbers
- Use `double` for high-precision calculations

### When to use `str`

✅ **Use for**:
- Names and titles
- Short descriptions
- Emails and URLs
- Codes and identifiers
- Phone numbers
- Addresses

❌ **Don't use for**:
- Numbers (use `int` or `float`)
- True/false (use `bool`)
- Dates (use `date`, `datetime`, or `time`)

### When to use `bool`

✅ **Use for**:
- Yes/No questions
- On/Off states
- True/False flags
- Active/Inactive status
- Enabled/Disabled settings

❌ **Don't use for**:
- Multiple states (use `str` or `int`)
- Counts (use `int`)

### When to use `datetime`

✅ **Use for**:
- Created/updated timestamps
- Event times
- Scheduled actions
- Full date-time values

❌ **Don't use for**:
- Date only (use `date`)
- Time only (use `time`)

### When to use `date`

✅ **Use for**:
- Birth dates
- Due dates
- Calendar dates
- Day-level precision

❌ **Don't use for**:
- Time-sensitive events (use `datetime`)
- Time of day (use `time`)

### When to use `time`

✅ **Use for**:
- Business hours
- Time of day
- Recurring times

❌ **Don't use for**:
- Full timestamps (use `datetime`)
- Dates (use `date`)

## Complete Example

```
title "Complete Type Example"

table Users {
  // Integer types
  id: int [pk, autoincrement],
  age: int [nullable],
  login_count: int [default=0],
  
  // String types
  username: str [unique],
  email: string [unique],
  first_name: str,
  last_name: str,
  bio: str [nullable],
  
  // Boolean types
  is_active: bool [default=TRUE],
  is_admin: bool [default=FALSE],
  email_verified: bool [default=FALSE],
  
  // Float types
  account_balance: float [default=0.0],
  rating: float [nullable],
  
  // Date/Time types
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW],
  birth_date: date [nullable],
  last_login: datetime [nullable]
}

table Products {
  id: int [pk, autoincrement],
  name: str [unique],
  description: str [nullable],
  
  // Numeric types
  price: float,
  cost: float,
  stock: int [default=0],
  weight: double [nullable],
  
  // Boolean flags
  is_available: bool [default=TRUE],
  is_featured: bool [default=FALSE],
  
  // Timestamps
  created_at: datetime [default=NOW],
  updated_at: datetime [default=NOW],
  discontinued_date: date [nullable]
}
```

## See Also

- [Schema Syntax Reference](schema-syntax.md)
- [Column Attributes](schema-syntax.md#column-attributes)
- [Examples](examples.md)
