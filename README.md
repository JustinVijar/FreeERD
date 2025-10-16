# FreeERD

**FreeERD** is a powerful, open-source Entity Relationship Diagram (ERD) generator written in Rust. Create beautiful, professional ERD diagrams from simple text-based schema definitions.

## ✨ Features

- 🎨 **Beautiful SVG Output** - Generate clean, scalable vector graphics
- 🔄 **Force-Directed Layout** - Automatic table positioning using Fruchterman-Reingold algorithm
- 📊 **Cardinality Notation** - Clear visual indicators (1/M) for relationship types
- 🎯 **Smart Routing** - Intelligent relationship line routing to avoid overlaps
- 🔍 **Schema Validation** - Built-in validation with helpful error messages
- ⚡ **Fast & Efficient** - Written in Rust for maximum performance
- 📝 **Simple Syntax** - Easy-to-learn schema definition language

## 🚀 Installation

### From Source

```bash
git clone https://github.com/JustinVijar/FreeERD.git
cd FreeERD/free-erd
cargo build --release
```

The binary will be available at `target/release/free-erd`

## 📖 Usage

### Basic Command

```bash
free-erd run <input-file.frd> svg [output-file.svg]
```

### Example

```bash
free-erd run examples/test_schema.frd svg diagram.svg
```

## 📝 Schema Syntax

### Basic Structure

```
title "My Database Schema"

table Users {
  id: int [pk],
  name: str,
  email: str [unique],
  created_at: datetime
}

table Posts {
  id: int [pk],
  user_id: int [fk],
  title: str,
  content: text,
  published: bool
}

Users.id > Posts.user_id
```

### Data Types

- `int` - Integer
- `str` - String/VARCHAR
- `text` - Long text
- `bool` - Boolean
- `float` - Floating point number
- `datetime` - Date and time
- `date` - Date only
- `time` - Time only

### Field Attributes

- `[pk]` - Primary Key
- `[fk]` - Foreign Key
- `[unique]` - Unique constraint
- `[nullable]` - Nullable field

### Relationship Types

- `>` - One-to-Many (1:N)
- `<` - Many-to-One (N:1)
- `<>` - Many-to-Many (N:M)
- `-` - One-to-One (1:1)

### Relationship Syntax

```
SourceTable.field > TargetTable.field
```

Examples:
```
Users.id > Posts.user_id          # One user has many posts
Posts.id < Comments.post_id       # Many comments belong to one post
Students.id <> Courses.id         # Many-to-many relationship
Users.id - UserProfiles.user_id  # One-to-one relationship
```

## 🎨 Visual Features

### Cardinality Labels

Relationships display clear cardinality indicators:
- **[1]** - Indicates the "one" side of a relationship
- **[M]** - Indicates the "many" side of a relationship

Labels appear as white text on black backgrounds at relationship connection points.

### Force-Directed Layout

Tables are automatically positioned using a physics-based algorithm that:
- Minimizes edge crossings
- Distributes tables evenly
- Creates visually balanced diagrams
- Adapts to schema complexity

### Smart Line Routing

Relationship lines intelligently route around tables to:
- Avoid overlapping with other elements
- Maintain clear visual paths
- Use adaptive connection points
- Group parallel relationships

## 📁 Examples

Check out the `examples/` directory for sample schemas:

- `test_schema.frd` - Comprehensive example with various relationship types
- `composite_keys.frd` - Complex schema with composite keys
- `test_errors.frd` - Examples of validation errors

## 🛠️ Development

### Project Structure

```
free-erd/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lexer.rs             # Tokenization
│   ├── parser.rs            # Syntax parsing
│   ├── ast.rs               # Abstract Syntax Tree
│   ├── interpreter.rs       # Schema validation
│   └── svg_generator/       # SVG generation
│       ├── mod.rs           # Module definition
│       ├── generator.rs     # Main generator logic
│       ├── renderer.rs      # SVG rendering
│       ├── layout.rs        # Layout calculations
│       └── force_layout.rs  # Force-directed algorithm
├── examples/                # Example schemas
└── Cargo.toml              # Project dependencies
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with example
cargo run -- run examples/test_schema.frd svg
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the GNU General Public License v2.0 (GPL-2.0).

See the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by the need for simple, beautiful ERD generation
- Built with Rust for performance and reliability
- Uses Fruchterman-Reingold algorithm for graph layout

## 📞 Support

For issues, questions, or suggestions, please open an issue on GitHub.

---

**Made with ❤️ and Rust**
