# Change Log

All notable changes to the ULissy VS Code extension will be documented in this file.

## [0.2.0] - 2025-01-30

### Added
- Full syntax highlighting for ULissy 0.2.0
  - `config { }` blocks
  - `computed` properties (expression and object forms)
  - Optional chaining `?.` and nil coalescing `??`
  - Interpolated strings `\(expr)`
- 40+ code snippets for common patterns
- Commands: Build, Check, Run, New Project
- Real-time diagnostics on save
- Language configuration (brackets, comments, folding)
- File icon for `.ul` files

### Technical
- TextMate grammar with 15+ scope categories
- Integration with ULissy compiler CLI
- Support for error code parsing (E0xxx format)
- Output channel for build logs

## [0.1.0] - 2025-01-29

### Added
- Initial release
- Basic syntax highlighting
- Language detection for `.ul` files

---

## Roadmap

### 0.3.0 (Planned)
- [ ] Language Server Protocol (LSP) integration
- [ ] Go to definition
- [ ] Find all references
- [ ] Rename symbol
- [ ] Code actions / quick fixes

### 0.4.0 (Planned)
- [ ] Debugger integration
- [ ] Test runner
- [ ] Code lens for breadcrumb counts
- [ ] Trajectory visualization

### 1.0.0 (Planned)
- [ ] Full LSP feature parity
- [ ] Stable API
- [ ] Published to VS Code Marketplace
