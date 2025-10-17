# FreeERD

**FreeERD** is a powerful, open-source Entity Relationship Diagram (ERD) generator written in Rust. Create beautiful, professional ERD diagrams from simple text-based schema definitions.

## ✨ Features

- 🎨 **Beautiful SVG Output** - Generate clean, scalable vector graphics using Graphviz
- 📐 **Hierarchical Layout** - Automatic table positioning with top-to-bottom flow
- 🔗 **Orthogonal Routing** - Clean, professional relationship lines with right-angle connections
- 🔍 **Schema Validation** - Built-in validation with helpful error messages
- ⚡ **Fast & Efficient** - Written in Rust for maximum performance
- 📝 **Simple Syntax** - Easy-to-learn schema definition language
- 🎯 **Rich Data Types** - Support for int, string, bool, float, double, date, datetime, time, and blob types
- 🔑 **Composite Keys** - Full support for composite primary and foreign keys
- 📊 **Comprehensive Examples** - Includes complex ERP system example

## 🚀 Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform from the [Releases page](https://github.com/JustinVijar/FreeERD/releases):

- **Linux**: `free-erd-linux-x86_64`
- **Windows**: `free-erd-windows-x86_64.exe`
- **macOS (Intel)**: `free-erd-macos-x86_64`
- **macOS (Apple Silicon)**: `free-erd-macos-aarch64`

#### Linux/macOS
```bash
# Download and make executable
chmod +x free-erd-*
sudo mv free-erd-* /usr/local/bin/free-erd
```

#### Windows
Download the `.exe` file and add it to your PATH or run it directly.

### From Source

```bash
git clone https://github.com/JustinVijar/FreeERD.git
cd FreeERD/free-erd
cargo build --release
```

The binary will be available at `target/release/free-erd`

## ⚠️ Prerequisites

### Graphviz Installation Required

**Important**: FreeERD requires Graphviz to be installed on your system for SVG generation.

> **Note**: I apologize for using Graphviz, which is written in C. While FreeERD itself is written in Rust, we currently rely on Graphviz's mature and battle-tested graph rendering capabilities. This dependency may be replaced with a pure Rust solution in the future.

#### Installation Instructions

**Linux (Debian/Ubuntu)**:
```bash
sudo apt-get install graphviz
```

**Linux (Fedora/RHEL)**:
```bash
sudo dnf install graphviz
```

**macOS**:
```bash
brew install graphviz
```

**Windows**:
- Download the installer from [Graphviz Download Page](https://graphviz.org/download/)
- Run the installer and add Graphviz to your PATH
- Or use Chocolatey: `choco install graphviz`

**Verify Installation**:
```bash
dot -V
```

You should see output like: `dot - graphviz version X.X.X`

For more installation options and detailed guides, visit: **https://graphviz.org/download/**

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
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique],
  is_active: bool [default=TRUE],
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

Users.id > Posts.user_id
```

### Data Types

- `int` / `integer` - Integer numbers
- `str` / `string` - Text strings
- `bool` / `boolean` - True/false values
- `float` - Single-precision floating point
- `double` - Double-precision floating point
- `datetime` - Date and time combined
- `date` - Date only
- `time` - Time only
- `blob` - Binary large object
- `tinyblob` - Small binary object
- `largeblob` - Large binary object
- Custom types - Any custom database type

### Field Attributes

- `[pk]` - Primary Key
- `[fk]` - Foreign Key
- `[unique]` - Unique constraint
- `[nullable]` - Nullable field
- `[autoincrement]` - Auto-increment field
- `[default=value]` - Default value (supports NOW, TRUE, FALSE, NULL, strings, numbers)

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

### Hierarchical Layout

Tables are automatically positioned using Graphviz's hierarchical layout algorithm:
- Top-to-bottom flow showing data dependencies
- Minimizes edge crossings
- Clean, professional appearance
- Adapts to schema complexity

### Orthogonal Line Routing

Relationship lines use orthogonal (right-angle) routing:
- Professional, clean appearance
- Clear visual paths between tables
- Proper spacing to avoid overlaps
- Relationship labels positioned clearly on lines

### Relationship Labels

Each relationship displays the connected fields:
- Format: `SourceTable.field → TargetTable.field`
- Labels positioned along relationship lines
- Clear indication of which fields are connected

### Arrow Styles

Different relationship types have distinct visual styles:
- **One-to-Many**: Crow's foot arrow (→)
- **Many-to-One**: Crow's foot arrow (←)
- **Many-to-Many**: Crow's foot arrows on both ends (↔)
- **One-to-One**: Dashed line with no arrows (--)

## 📁 Examples

Check out the `examples/` directory for sample schemas:

- `test_schema.frd` - Simple blog platform with all relationship types
- `composite_keys.frd` - Complex schema demonstrating composite primary keys
- `complex_schema.frd` - Enterprise ERP system with 28 tables and 47 relationships
- `test_errors.frd` - Examples of validation errors
- `test_syntax_errors.frd` - Examples of syntax errors

For detailed documentation, see the `documentation/` directory:
- `README.md` - Complete documentation index
- `data-types.md` - All supported data types
- `examples.md` - Real-world schema examples
- `schema-syntax.md` - Complete syntax reference
- `relationships.md` - Relationship types guide

## 🛠️ Development

### Project Structure

```
free-erd/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lexer.rs             # Tokenization
│   ├── parser.rs            # Syntax parsing
│   ├── ast.rs               # Abstract Syntax Tree
│   ├── interpreter.rs       # Schema validation & security
│   └── svg_generator/       # SVG generation
│       ├── mod.rs           # Module definition
│       └── generator.rs     # Graphviz-based SVG generation
├── examples/                # Example schemas
├── documentation/           # Comprehensive documentation
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
- Uses Graphviz for professional graph layout and rendering
- Security-hardened with comprehensive input validation

## 📞 Support

For issues, questions, or suggestions, please open an issue on GitHub.

---

**Made with ❤️ and Rust**
