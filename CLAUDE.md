# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all crates
cargo build

# Build release
cargo build --release

# Run the application
cargo run -p rk-frontend

# Run tests
cargo test

# Run tests for a specific crate
cargo test -p rk-core

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Architecture

RK is a 3D CAD editor built with Rust. The codebase is organized as a Cargo workspace with three crates:

### Crate Dependencies

```
rk-frontend (egui application)
    └── rk-renderer (wgpu rendering)
            └── rk-core (data structures)
```

### rk-core

Core data structures and logic:

- `Part`: Mesh with metadata and joint points
- `Assembly`: Scene graph for hierarchical structure
- `Project`: Serializable project file (RON format)
- Mesh import/export (STL, OBJ, DAE, URDF)

### rk-renderer

WGPU-based 3D renderer with plugin architecture:

- `SubRenderer` trait: Interface for custom renderers
- `RendererRegistry`: Plugin system for sub-renderers
- `RenderContext`: GPU context abstraction
- `Scene` / `RenderObject`: Scene management
- `MeshManager`: GPU mesh resource management
- Built-in sub-renderers: Grid, Mesh, Axis, Marker, Gizmo

### rk-frontend

egui-based GUI application:

- `AppState`: Central application state with action queue pattern
- `AppAction`: Enum defining all possible state mutations
- `SharedAppState`: Thread-safe state wrapper (`Arc<Mutex<AppState>>`)
- Panels in `panels/` module for UI components

## Key Patterns

- **Action Queue**: UI components queue `AppAction` variants, which are processed centrally in the update loop
- **Plugin Renderer**: New rendering features implement `SubRenderer` trait and register with `RendererRegistry`
- **Shared State**: `SharedAppState` (`Arc<Mutex<AppState>>`) is passed to panels and the renderer

## Platform Support

- Native: Linux (X11/Wayland), Windows, macOS
- WASM: Web browser support with conditional compilation (`cfg(target_arch = "wasm32")`)
