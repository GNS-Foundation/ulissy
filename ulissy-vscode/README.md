# ULissy for Visual Studio Code

Official Visual Studio Code extension for the **ULissy** programming language - *A Programming Language for Moving Machines*.

![ULissy Syntax Highlighting](https://raw.githubusercontent.com/GNS-Foundation/ulissy-vscode/main/images/screenshot.png)

## Features

### Syntax Highlighting

Full syntax highlighting for all ULissy constructs:

- Keywords (`identity`, `every`, `when`, `after`, `fn`, `type`, `enum`, `config`)
- Types (`Identity`, `H3Cell`, `Breadcrumb`, `Trajectory`, `Duration`)
- Handles (`@alice`, `@bob`)
- Facets (`dix@alice`, `home@bob/lights`, `pay@merchant`)
- Unit values (`10.minutes`, `500.meters`, `80.percent`)
- String interpolation (`"Hello, \(name)!"`)
- Comments (line `//` and block `/* */`)

### Code Snippets

Type these prefixes and press Tab:

| Prefix | Description |
|--------|-------------|
| `identity` | Identity declaration |
| `every` | Periodic execution block |
| `everywhen` | Conditional periodic block |
| `when` | Condition trigger |
| `after` | Delayed execution |
| `config` | Configuration block |
| `type` | Type definition |
| `enum` | Enum definition |
| `fn` | Function definition |
| `computed` | Computed property |
| `send` | Send message |
| `breadcrumb` | Breadcrumb collection |
| `trajectory` | Full trajectory loop |
| `epoch` | Epoch publishing |
| `ulissy` | Program template |
| `ulissyt` | Trajectory program template |

### Commands

Access via Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

| Command | Shortcut | Description |
|---------|----------|-------------|
| ULissy: Build | `Ctrl+Shift+B` | Compile current file |
| ULissy: Check | `Ctrl+Shift+C` | Type check current file |
| ULissy: Run | | Build and run |
| ULissy: Create New Project | | Scaffold new project |
| ULissy: Show Tokens | | Debug: show lexer output |
| ULissy: Show AST | | Debug: show parsed AST |

### Real-time Diagnostics

- Type errors shown inline as you save
- Quick fixes and suggestions (coming soon)
- Error codes link to documentation

### Auto-formatting

- Auto-close brackets and quotes
- Auto-indent on enter
- Smart comment continuation

## Requirements

- [ULissy Compiler](https://github.com/GNS-Foundation/ULissy_Program) installed and in PATH
- VS Code 1.85.0 or higher

## Installation

### From VS Code Marketplace

1. Open VS Code
2. Go to Extensions (`Ctrl+Shift+X`)
3. Search for "ULissy"
4. Click Install

### From VSIX

1. Download the `.vsix` file from [releases](https://github.com/GNS-Foundation/ulissy-vscode/releases)
2. In VS Code: Extensions → `...` → Install from VSIX

### From Source

```bash
git clone https://github.com/GNS-Foundation/ulissy-vscode
cd ulissy-vscode
npm install
npm run compile
code --install-extension ulissy-0.2.0.vsix
```

## Extension Settings

Configure in Settings (`Ctrl+,`):

| Setting | Default | Description |
|---------|---------|-------------|
| `ulissy.compilerPath` | `ulissy` | Path to compiler executable |
| `ulissy.enableLSP` | `true` | Enable language server features |
| `ulissy.checkOnSave` | `true` | Type check on file save |
| `ulissy.formatOnSave` | `false` | Format on save |

## Example

```ulissy
// Proof-of-Trajectory Collection
import ulissy.prelude

identity me = Keychain.primary

config {
    resolution: 7,
    interval: 10.minutes,
    minBreadcrumbsPerEpoch: 100
}

// Main collection loop
every config.interval when battery > 20 && gps.available {
    let crumb = breadcrumb(
        cell: here.h3(config.resolution),
        context: sensors.digest(),
        previous: me.trajectory.lastHash ?? "genesis"
    ).signed(me)
    
    me.trajectory.append(crumb)
}

// Epoch publishing
when me.trajectory.pending >= config.minBreadcrumbsPerEpoch {
    let epoch = me.trajectory.bundleEpoch().signed(me)
    network.publish(epoch)
}
```

## Color Themes

The extension works with any VS Code theme. For best results, use a theme with good support for:

- `keyword.control` - Control flow keywords
- `storage.type` - Type declarations
- `entity.name.type` - Type names
- `variable.other` - Variables and handles
- `constant.numeric` - Numbers and units
- `string.quoted` - Strings

## Troubleshooting

### Compiler not found

```
ULissy compiler not found. Please install it or set the path in settings.
```

**Solution:** Install the ULissy compiler and either:
1. Add it to your PATH, or
2. Set `ulissy.compilerPath` in settings

### No syntax highlighting

Make sure your file has the `.ul` extension.

### Language server not starting

Check the Output panel (View → Output → ULissy) for error messages.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

MIT - GNS Foundation

## Links

- [ULissy Language](https://github.com/GNS-Foundation/ULissy_Program)
- [GNS Protocol](https://gns.foundation)
- [Report Issues](https://github.com/GNS-Foundation/ulissy-vscode/issues)
