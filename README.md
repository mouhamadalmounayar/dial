# Dial
Dial is a code snippet manager built using rust and ratatui.
<img width="2558" height="1438" alt="image" src="https://github.com/user-attachments/assets/9065b349-5d47-4eca-baa8-c84db345ae3a" />


## Features

-   **Terminal-First Workflow**: Manage all your code snippets without leaving the command line.

-   **Search & Syntax Highlighting**: Quickly find the code you need and review it with clear syntax highlighting.

-   **Responsive Text Editing**: Make quick modifications using a gap buffer based editor.

-   **Cross-Platform**: Runs on Linux, macOS, and Windows, storing data in the appropriate system-native directories.

## Installation

To get started with Dial, you need to have the Rust toolchain installed on your system.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/mouhamadalmounayar/dial.git
    ```

2.  **Navigate into the project directory:**
    ```bash
    cd dial
    ```

3.  **Build and run the application:**
    ```bash
    cargo run
    ```
    The application will automatically create a configuration directory and a `snippets.json` file if they don't exist.

## Roadmap

- [x] Navigate Snippet List
- [x] Display Snippets in Editor Panel
- [x] Syntax Highlighting for Code
- [x] Responsive Text Editing with Gap Buffer
- [x] Persist Snippets to Disk
- [x] Live Search by Snippet Title
- [x] Add Snippets from within the TUI
- [ ] Remove Snippets from within the TUI
- [ ] Implement Command-Line Interface (CLI)
- [ ] Add Snippets from the Command Line
- [ ] Remove Snippets from the Command Line
- [ ] Add Snippets from Clipboard via CLI
- [ ] Implement Smooth Scrolling for Snippet List
- [ ] Add Vertical Cursor Navigation in Editor
- [ ] Implement Fuzzy Finder for Advanced Search
- [ ] Tag and Filter Snippets
- [ ] Customizable UI and Editor Theming

## Configuration

Dial stores its data in a simple JSON file located in your system's standard config directory.

-   **Linux:** `~/.config/dial/snippets.json`
-   **macOS:** `~/Library/Application Support/com.mouhamadalmounayar.dial/snippets.json`
-   **Windows:** `C:\Users\{YourUser}\AppData\Roaming\mouhamadalmounayar\dial\data\snippets.json`

You can manually add or edit snippets in this file. The expected format for each snippet is:

```json
[
    {
        "language": "rust",
        "title": "Rust Hello World",
        "code": "fn main() {\n    println!(\"Hello, Rust!\");\n}"
    },
    {
        "language": "py",
        "title": "Simple Python Function",
        "code": "def greet(name):\n    print(f\"Hello, {name}!\")\n\ngreet(\"World\")"
    }
]
```

## Technology Stack

-   **Core Application**: [Rust](https://www.rust-lang.org/)
-   **TUI Framework**: [ratatui](https://ratatui.rs/)
-   **JSON Handling**: [serde_json](https://github.com/serde-rs/json)
-   **Cross-Platform Directories**: [directories-rs](https://github.com/dirs-dev/directories-rs)
-   **Error Handling**: [anyhow](https://github.com/dtolnay/anyhow)

## Contributing

Contributions are welcome! If you have ideas for new features or have found a bug, please open an issue or submit a pull request.


