# FreeERD

**FreeERD** is a powerful, open-source diagram generator written in Rust. Create beautiful, professional Entity Relationship Diagrams (ERD) and graph database schemas with an interactive visual editor or export to SVG.

## ✨ Features

- **Interactive Window** - Visualize and interact with your diagrams in real-time
- **Drag & Drop** - Move tables, nodes, and labels to customize layout
- **Smart Layout** - Automatic force-directed positioning with collision avoidance
- **Export to SVG** - Generate publication-ready vector graphics
- **Zoom & Pan** - Navigate large schemas with ease
- **Selection Highlighting** - Click entities to highlight their relationships/connections
- **Orthogonal Routing** - Clean, professional relationship lines


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

## 📖 Usage

### Commands

```bash
# Open interactive window to view and edit diagram
free-erd run <input-file.frd>

# Validate schema without opening window
free-erd check <input-file.frd>

# Show help
free-erd help

# Show version and about information
free-erd about
```

### Interactive Window

The `run` command opens an interactive window where you can:

- **View** your diagram with automatic layout
- **Drag** tables, nodes, and labels to customize positions
- **Zoom** in/out using mouse scroll or +/- keys
- **Pan** using arrow keys or mouse drag
- **Select** entities to highlight their relationships/connections
- **Export** to SVG via the Export menu

### Keyboard Controls

- **Scroll Wheel** / **+/-** - Zoom in/out
- **Arrow Keys** - Pan the view
- **Mouse Drag** - Move tables and labels
- **Left Click** - Select table (highlights relationships)

### Export to SVG

1. Open your schema: `free-erd run myschema.frd`
2. Arrange tables and labels as desired
3. Click **Export > SVG** in the menu bar
4. SVG file will be saved with timestamp (e.g., `export_20251019_143022.svg`)

### Example

```bash
# Validate a schema
free-erd check examples/test_schema.frd

# Open in interactive window
free-erd run examples/test_schema.frd
```

## 📝 Schema Syntax

### Relational Database (ERD) Example

```
#title "My Database Schema"

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
  post_title: str,
  content: str,
  published: bool [default=FALSE],
  created_at: datetime [default=NOW]
}

Users.id > Posts.user_id
```

### Graph Database Example (not working on window)

```
#title "Social Network Graph"

node User {
  id: int [pk, autoincrement],
  username: str [unique],
  email: str [unique]
}

node Post {
  id: int [pk, autoincrement],
  content: str,
  created_at: datetime
}

edge FOLLOWS (from: User, to: User) {
  since: date
}

edge AUTHORED (from: User, to: Post) {
  created_at: datetime
}

// Shorthand syntax
User -[LIKES]-> Post
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

### Interactive Window

FreeERD provides a modern, interactive window for visualizing and editing your ERD:

- **Real-time Rendering** - See your schema come to life instantly
- **Draggable Elements** - Move tables and labels with your mouse
- **Zoom Controls** - Scroll to zoom in/out, +/- keys also work
- **Pan Navigation** - Arrow keys to navigate, or drag the canvas
- **Selection Highlighting** - Click any table to highlight its relationships
- **Customizable Layout** - Arrange elements exactly how you want them

### Force-Directed Layout

Tables are automatically positioned using a sophisticated force-directed algorithm:

- **Collision Avoidance** - Tables never overlap
- **Natural Spacing** - Related tables positioned closer together
- **Hierarchical Flow** - Top-to-bottom organization based on relationships
- **Adaptive Positioning** - Adjusts to schema complexity

### Orthogonal Line Routing

Relationship lines use smart orthogonal (right-angle) routing:

- **Professional Appearance** - Clean, right-angle paths between tables
- **Collision Avoidance** - Lines avoid overlapping with tables
- **Distributed Connection Points** - Multiple relationships spread along table borders
- **Clear Visual Paths** - Easy to trace relationships

### Relationship Labels

Each relationship displays comprehensive information:

- **Format**: `[1:M] SourceTable.field:TargetTable.field`
- **Collision Avoidance** - Labels positioned to avoid overlaps with tables and other labels
- **Pointer Lines** - Visual indicator connecting label to its relationship line
- **Draggable** - Reposition labels for optimal readability
- **Color-Coded** - Relationship type ([1:M]) in gray, field names in white

### Visual Relationship Types

Different relationship types have distinct visual representations:

- **One-to-Many (>)**: Single line with crow's foot at the "many" end
- **Many-to-One (<)**: Crow's foot at the "many" end
- **Many-to-Many (<>)**: Crow's feet at both ends
- **One-to-One (-)**: Single line with no crow's feet

### SVG Export

The export feature creates pixel-perfect SVG files:

- **Exact Positioning** - Exports current window state including custom positioning
- **Vector Graphics** - Scalable to any size without quality loss
- **Complete Rendering** - All tables, relationships, labels, and title included
- **Professional Quality** - Publication-ready output

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
│   └── renderer/            # Interactive renderer (pure Rust)
│       ├── mod.rs           # Module definition
│       ├── canvas.rs        # Main rendering & interaction logic
│       ├── graph.rs         # ERD graph data structures
│       └── layout.rs        # Force-directed layout engine
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
cargo run -- run examples/test_schema.frd
```

### Key Technologies

- **egui** - Immediate mode GUI framework
- **eframe** - egui framework for desktop applications
- **petgraph** - Graph data structures for ERD representation
- **Force-directed layout** - Custom implementation for automatic positioning
- **Orthogonal routing** - Custom algorithm for clean relationship lines

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
- Uses egui for interactive, cross-platform GUI
- Security-hardened with comprehensive input validation
- **100% Pure Rust** - No external dependencies required

## 📞 Support

For issues, questions, or suggestions, please open an issue on GitHub.

---

**Made with ❤️ and Rust**
