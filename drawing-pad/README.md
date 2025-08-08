# Drawing Pad

A simple drawing application built with Rust using the rust-skia library.

## Features

- **Draw with mouse**: Click and drag to draw on the canvas
- **Clear canvas**: Press the Space key to clear the drawing
- **Smooth lines**: Anti-aliased drawing for smooth strokes
- **Resizable window**: The canvas adjusts when you resize the window

## Usage

1. Build and run the application:
   ```bash
   cargo run
   ```

2. Controls:
   - **Left mouse button**: Hold down and move to draw
   - **Space key**: Clear the canvas

## Dependencies

- `skia-safe`: Rust bindings for the Skia graphics library
- `winit`: Window creation and event handling
- `glutin`: OpenGL context creation
- `glutin-winit`: Integration between glutin and winit

## Requirements

- Rust 1.70 or later
- OpenGL 3.3 or later support
- Build tools for compiling native dependencies (LLVM, Python 3, Ninja)

## Architecture

The application uses:
- **Skia** for 2D graphics rendering
- **OpenGL** as the GPU backend
- **winit** for cross-platform window management
- **glutin** for OpenGL context management

The drawing is implemented by tracking mouse movements and building a path that gets rendered each frame using Skia's path drawing capabilities.