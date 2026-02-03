# `pretty-console`

[![Crates.io](https://img.shields.io/crates/v/pretty_console)](https://crates.io/crates/pretty_console)
[![Documentation](https://docs.rs/pretty_console/badge.svg)](https://docs.rs/pretty_console)
[![License](https://img.shields.io/crates/l/pretty_console)](LICENSE)

A fluent, zero-cost API for styling terminal text with colors and attributes. Write beautifully formatted console output with an intuitive, chainable interface.

## Features

- 🎨 **Rich Color Support**: 16 named colors, 256 colors, and true color RGB
- ⚡ **Zero-Cost Abstractions**: Builder pattern compiles to efficient code
- 🔧 **Fluent API**: Chainable methods for intuitive styling
- 📝 **Text Attributes**: Bold, italic, underline, blink, reverse, and more
- 🌍 **Cross-Platform**: ANSI escape codes with Windows 10+ support
- 🎯 **Flexible Output**: Print immediately or get formatted strings
- 🔄 **Reusable Styles**: Create style templates and reuse them
- 🚫 **No-Color Support**: Optional feature to disable all coloring

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
pretty_console = "0.1.0"
```

Basic usage:

```rust
use pretty_console::Console;

fn main() {
    Console::new("Hello, world!").red().bold().println();
    Console::new("This is a warning").yellow().underline().println();
    Console::new("Success!").green().on_black().println();
}
```

## Examples

### Basic Styling

```rust
use pretty_console::Console;

// Simple color and attribute combinations
Console::new("Error message").red().bold().println();
Console::new("Warning message").yellow().italic().println();
Console::new("Info message").blue().underline().println();

// Background colors
Console::new("Inverted").white().on_red().println();
```

### RGB Colors

```rust
Console::new("Custom color!").fg_rgb(255, 0, 128).println();
Console::new("Gradient").bg_rgb(128, 255, 0).println();
```

### Multiple Attributes

```rust
Console::new("Important notice!")
    .bright_red()
    .on_bright_yellow()
    .bold()
    .italic()
    .underline()
    .blink()
    .println();
```

### Reusable Styles

```rust
use pretty_console::Console;

// Create a style template
let error_style = Console::new("").red().bold();
let warning_style = Console::new("").yellow().italic();
let success_style = Console::new("").green().bold();

// Reuse with different text
error_style.with_text("Error: File not found").println();
warning_style.with_text("Warning: Deprecated API").println();
success_style.with_text("Success: Operation completed").println();
```

### Using with Format Strings

```rust
use pretty_console::Console;

let username = "Alice";
let status = Console::new("online").green().bold();

println!("User {} is {}", username, status);
// Output: User Alice is online (with "online" in green and bold)
```

### Advanced Style Management

```rust
use pretty_console::{Console, Style, Color};

// Create predefined styles
let heading_style = Style::new()
    .fg(Color::BRIGHT_BLUE)
    .bold()
    .underline();

let note_style = Style::new()
    .fg(Color::BRIGHT_BLACK)
    .italic();

// Use the styles
Console::new_with_style("Chapter 1: Introduction", heading_style).println();
Console::new_with_style("Note: This is important", note_style).println();
```

## API Reference

### Color Constants

```rust
// Basic colors
Color::BLACK, Color::RED, Color::GREEN, Color::YELLOW
Color::BLUE, Color::MAGENTA, Color::CYAN, Color::WHITE

// Bright colors
Color::BRIGHT_BLACK, Color::BRIGHT_RED, Color::BRIGHT_GREEN
Color::BRIGHT_YELLOW, Color::BRIGHT_BLUE, Color::BRIGHT_MAGENTA
Color::BRIGHT_CYAN, Color::BRIGHT_WHITE
```

### Console Methods

#### Color Setters
- `fg(color)`, `bg(color)` - Set foreground/background color
- `fg_rgb(r, g, b)`, `bg_rgb(r, g, b)` - Set RGB colors
- `red()`, `green()`, `blue()`, etc. - Foreground color shortcuts
- `on_red()`, `on_green()`, `on_blue()`, etc. - Background color shortcuts

#### Attribute Setters
- `bold()`, `italic()`, `underline()`, `blink()`
- `dim()`, `reverse()`, `hidden()`, `strikethrough()`

#### Output Methods
- `print()` - Print without newline
- `println()` - Print with newline
- `write_to(writer)` - Write to any `std::io::Write`
- `to_string()` - Get formatted string

## No-Color Support

For environments where terminal colors aren't supported or desired:

```toml
[dependencies]
pretty_console = { version = "0.1.0", features = ["no-color"] }
```

With the `no-color` feature, all ANSI escape codes are omitted, and text is printed as-is.

## Performance

The builder pattern uses zero-cost abstractions - all styling is computed at compile time where possible. The ANSI code generation is optimized and only occurs when actually printing.

## Platform Support

- **Unix-like systems**: Full support via ANSI escape codes
- **Windows 10+**: Full support via virtual terminal sequences
- **Older Windows**: Limited support (requires enabling virtual terminal)

For best Windows compatibility, ensure your terminal supports ANSI escape codes or use Windows 10+.

## Alternatives Comparison

| Feature | pretty_console | termcolor | colored |
|---------|---------------|-----------|---------|
| Fluent API | ✅ | ❌ | ✅ |
| True Color | ✅ | ✅ | ✅ |
| Zero-Cost | ✅ | ✅ | ❌ |
| Reusable Styles | ✅ | ❌ | ❌ |
| Attribute Combos | ✅ | ✅ | ✅ |
| No Dependencies | ✅ | ❌ | ✅ |

## License

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.

---

**Happy styling!** 🎨
