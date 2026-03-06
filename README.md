# Math Grapher

[![Release](https://img.shields.io/github/v/release/riverwu/math-grapher)](https://github.com/riverwu/math-grapher/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Desmos-like mathematical graphing calculator built with Rust, featuring GPU-accelerated rendering and real-time expression evaluation.

[中文](#中文说明)

## Download

**[Download Latest Release (v0.1.0)](https://github.com/riverwu/math-grapher/releases/latest)**

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [math-grapher-macos-arm64](https://github.com/riverwu/math-grapher/releases/download/v0.1.0/math-grapher-macos-arm64) |

### macOS: First Run Instructions

Since the binary is not signed with an Apple Developer certificate, macOS Gatekeeper will block it. To run:

**Method 1: Remove quarantine attribute (Recommended)**
```bash
# Download and make executable
chmod +x math-grapher-macos-arm64
# Remove quarantine attribute
xattr -cr math-grapher-macos-arm64
# Run
./math-grapher-macos-arm64
```

**Method 2: System Preferences**
1. Try to open the app (it will be blocked)
2. Go to **System Preferences > Privacy & Security**
3. Find the blocked app message and click **"Open Anyway"**

## Features

### Expression Types
- **Explicit functions**: `y = sin(x)`, `y = x^2 + 1`
- **Implicit functions**: `x^2 + y^2 = 4` (circles, ellipses, etc.)
- **Parametric equations**: `[cos(t), sin(t)]`, `[2*cos(t), sin(t)]`
- **Polar coordinates**: `r = sin(3*theta)`, `r = 1 + cos(theta)`
- **Inequalities**: `y > x^2`, `y <= sin(x)`, `x^2 + y^2 < 4`

### Mathematical Functions (32+)
- Trigonometric: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`
- Exponential/Log: `exp`, `ln`, `log`, `log2`
- Power/Root: `sqrt`, `cbrt`, `pow`, `^`
- Utilities: `abs`, `sign`, `floor`, `ceil`, `round`, `min`, `max`, `factorial`

### Algebra Tools
- **Intersection detection**: Automatic marking of curve intersections with coordinates
- **Curve fitting**: Linear, Quadratic, Cubic, Exponential, Power models
  - Click-to-add data points
  - R² and residual analysis
  - Add fitted curves to graph

### Interactive Features
- Pan and zoom with mouse/keyboard
- Real-time coordinate display
- Click-to-query coordinates with markers
- Parameter sliders with real-time animation (default value: 1.0)
- Expression syntax highlighting
- Error hints and suggestions
- Undo/redo support (Ctrl+Z / Ctrl+Shift+Z)
- LaTeX input support (`\sin{x}`, `\frac{1}{x}`, `\sqrt{x}`, `x^{2}`)
- Expression history for quick re-entry
- Multiple curves with automatic coloring
- Visibility toggles for each expression
- Quick-add example buttons

## Installation

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))

### Build & Run
```bash
git clone https://github.com/riverwu/math-grapher.git
cd math-grapher
cargo run --release
```

### Run Tests
```bash
cargo test
```

## Usage

### Basic Input
1. Type an expression in the input field (e.g., `y = sin(x)`)
2. Press Enter or click "Add"
3. The curve appears on the graph

### Syntax Examples
```
y = x^2                    # Parabola
y = sin(x) + cos(2*x)      # Composite trig
x^2 + y^2 = 4              # Circle (radius 2)
[cos(t), sin(t)]           # Parametric circle
[2*cos(t), sin(t)]         # Parametric ellipse
r = sin(3*theta)           # Rose curve (polar)
r = 1 + cos(theta)         # Cardioid (polar)
y > x^2                    # Region above parabola
y < sin(x)                 # Region below sine
x^2 + y^2 < 4              # Disk interior
```

### Keyboard Shortcuts
| Key | Action |
|-----|--------|
| `R` or `0` | Reset view |
| `+` / `=` | Zoom in |
| `-` | Zoom out |
| Arrow keys | Pan |
| `Q` | Toggle coordinate query mode |
| `C` | Clear query point |
| `Escape` | Exit query/fit mode |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` / `Ctrl+Y` | Redo |

### Curve Fitting
1. Click "Curve Fit" in the toolbar
2. Click "Add Points" to enter point-adding mode
3. Click on the graph to add data points
4. Select a fit model (Linear, Quadratic, etc.)
5. View the fitted equation and R² value
6. Click "Add to Graph" to keep the fitted curve

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| GUI Framework | egui + eframe |
| Graphics | wgpu (Vulkan/Metal/DX12) |
| Linear Algebra | nalgebra |
| Serialization | serde |

## Project Structure

```
src/
├── main.rs              # Application entry
├── lib.rs               # Library exports, common types
├── parser/              # Expression parsing
│   ├── ast.rs           # Abstract syntax tree
│   ├── lexer.rs         # Tokenization
│   └── mod.rs           # Parser implementation
├── evaluator/           # Expression evaluation
│   ├── cpu_eval.rs      # CPU evaluator
│   ├── adaptive.rs      # Adaptive sampling
│   └── mod.rs           # Evaluation functions
├── algebra/             # Algebraic operations
│   ├── intersection.rs  # Curve intersections
│   ├── roots.rs         # Root finding
│   ├── fitting.rs       # Curve fitting
│   └── derivative.rs    # Numerical derivatives
├── render/              # Rendering
│   ├── canvas.rs        # Graph canvas
│   ├── grid.rs          # Coordinate grid
│   ├── curve.rs         # Curve rendering
│   ├── region.rs        # Inequality regions
│   └── markers.rs       # Point markers
└── ui/                  # User interface
    ├── app.rs           # Main application
    └── ...              # UI components
```

## Roadmap

- [x] **Phase 1**: Basic framework (expressions, explicit/implicit curves, pan/zoom)
- [x] **Phase 2**: Advanced plotting (parametric, polar, inequalities)
- [x] **Phase 3**: Algebra tools (intersections, curve fitting)
- [x] **Phase 4**: Interaction (sliders, syntax highlighting, undo/redo, LaTeX input)
- [ ] **Phase 5**: Performance & export (GPU compute, PNG/SVG export, point dragging)

## License

MIT License

---

# 中文说明

一个类似 Desmos 的数学图形计算器，使用 Rust 构建，支持 GPU 加速渲染和实时表达式求值。

## 下载

**[下载最新版本 (v0.1.0)](https://github.com/riverwu/math-grapher/releases/latest)**

| 平台 | 下载 |
|------|------|
| macOS (Apple Silicon) | [math-grapher-macos-arm64](https://github.com/riverwu/math-grapher/releases/download/v0.1.0/math-grapher-macos-arm64) |

### macOS 首次运行说明

由于二进制文件未经 Apple 开发者签名，macOS 会阻止运行。解决方法：

**方法 1：移除隔离属性（推荐）**
```bash
# 添加执行权限
chmod +x math-grapher-macos-arm64
# 移除隔离属性
xattr -cr math-grapher-macos-arm64
# 运行
./math-grapher-macos-arm64
```

**方法 2：系统偏好设置**
1. 尝试打开应用（会被阻止）
2. 打开 **系统偏好设置 > 隐私与安全性**
3. 找到被阻止的应用提示，点击 **"仍要打开"**

## 功能特性

### 支持的表达式类型
- **显式函数**: `y = sin(x)`, `y = x^2 + 1`
- **隐式函数**: `x^2 + y^2 = 4`（圆、椭圆等）
- **参数方程**: `[cos(t), sin(t)]`
- **极坐标**: `r = sin(3*theta)`
- **不等式**: `y > x^2`, `x^2 + y^2 < 4`

### 代数工具
- **交点检测**: 自动标记曲线交点并显示坐标
- **曲线拟合**: 支持线性、二次、三次、指数、幂函数模型

## 快速开始

```bash
git clone https://github.com/riverwu/math-grapher.git
cd math-grapher
cargo run --release
```

## 语法示例

```
y = x^2                    # 抛物线
x^2 + y^2 = 4              # 圆
[cos(t), sin(t)]           # 参数圆
r = sin(3*theta)           # 玫瑰线
y > x^2                    # 抛物线上方区域
```

## 技术栈

- **语言**: Rust
- **GUI**: egui + eframe
- **图形**: wgpu
- **线性代数**: nalgebra
