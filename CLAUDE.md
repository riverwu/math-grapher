# Math Grapher - Claude 项目文档

## 1. 项目愿景与红线

### 愿景
构建一个类似 Desmos 的跨平台（Linux/macOS）数学图形计算器，提供：
- 直观的数学表达式输入和实时可视化
- 高性能 GPU 加速渲染
- 丰富的曲线类型支持（显式、隐式、参数、极坐标、不等式）
- 交点计算和曲线拟合等代数能力

### 红线（不可违背的原则）
- **性能优先**：渲染和计算必须保持流畅，复杂表达式不应阻塞 UI
- **数值稳定性**：正确处理边界情况（除零、NaN、无穷大）
- **代码质量**：所有新功能必须有测试覆盖
- **跨平台兼容**：不使用平台特定 API，确保 Linux/macOS 兼容
- **用户体验**：实时预览、即时反馈，操作响应延迟 < 16ms

---

## 2. 技术栈与架构

### 核心依赖
```toml
eframe = "0.28"      # egui 跨平台框架
egui = "0.28"        # 即时模式 GUI
wgpu = "0.20"        # GPU 渲染 (Vulkan/Metal/DX12)
nalgebra = "0.33"    # 线性代数
serde = "1.0"        # 序列化
thiserror = "1.0"    # 错误处理
env_logger = "0.11"  # 日志
```

### 系统架构
```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Expression  │  │  Settings   │  │    File I/O         │  │
│  │   Panel     │  │   Panel     │  │  (Save/Load Graph)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                       Core Engine                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Parser    │  │  Evaluator  │  │   Algebra Engine    │  │
│  │ (自研词法/  │  │ (CPU/GPU)   │  │ (Intersect/Fit)     │  │
│  │  语法分析)  │  │             │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      Render Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Canvas    │  │  Grid/Axes  │  │   Curve Renderer    │  │
│  │  (wgpu)     │  │             │  │   (Line/Region)     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 模块结构
```
src/
├── main.rs              # 应用入口
├── lib.rs               # 库入口，公共类型 (Point, Rect, Color)
├── parser/              # 表达式解析
│   ├── ast.rs           # AST 定义 (AstNode, ExpressionType, ParsedEquation)
│   ├── lexer.rs         # 词法分析
│   ├── validator.rs     # 语法验证
│   ├── hints.rs         # 错误提示和建议
│   └── mod.rs           # 解析器实现
├── evaluator/           # 计算引擎
│   ├── cpu_eval.rs      # CPU 求值器
│   ├── interval.rs      # 区间算术
│   ├── adaptive.rs      # 自适应采样
│   └── mod.rs           # 求值函数 (explicit, implicit, parametric, polar, inequality)
├── algebra/             # 代数运算
│   ├── intersection.rs  # 交点计算
│   ├── roots.rs         # 方程求根
│   ├── derivative.rs    # 数值微分
│   └── fitting.rs       # 曲线拟合
├── render/              # 渲染模块
│   ├── canvas.rs        # 画布管理
│   ├── grid.rs          # 网格渲染
│   ├── curve.rs         # 曲线渲染
│   ├── region.rs        # 区域渲染 (不等式)
│   └── markers.rs       # 标记点渲染
└── ui/                  # 用户界面
    ├── app.rs           # 主应用状态
    ├── expr_panel.rs    # 表达式面板
    ├── graph_view.rs    # 图形视图
    ├── slider.rs        # 参数滑块
    ├── syntax.rs        # 语法高亮
    ├── history.rs       # 撤销/重做
    └── settings.rs      # 设置面板
```

### 关键数据流
```
用户输入 → Lexer → Parser → AST → Validator
                                      ↓
                              CompiledExpression
                                      ↓
                    ┌─────────────────┼─────────────────┐
                    ↓                 ↓                 ↓
              evaluate_*()      evaluate_*()     evaluate_*()
                    ↓                 ↓                 ↓
               CurveData        LineSegments    InequalityRegion
                    ↓                 ↓                 ↓
              CurveRenderer    CurveRenderer    RegionRenderer
                    └─────────────────┼─────────────────┘
                                      ↓
                               egui Painter
```

---

## 3. 项目路线图与当前状态

### 阶段 1：基础框架 (MVP) ✅ 已完成
- [x] 项目初始化，配置 Cargo.toml
- [x] egui + wgpu 基础窗口
- [x] 表达式解析（支持 +、-、*、/、^、32种数学函数）
- [x] CPU 求值器
- [x] 基础坐标系渲染（网格、刻度、标签）
- [x] 显函数曲线绘制 `y = f(x)`
- [x] 隐函数曲线绘制 (Marching Squares) `F(x,y) = 0`
- [x] 鼠标缩放和平移
- [x] 多曲线叠加和颜色管理

### 阶段 2：高级绘图 ✅ 已完成
- [x] 参数方程支持 `[cos(t), sin(t)]`
- [x] 极坐标支持 `r = sin(3*theta)`
- [x] 不等式区域填充 `y > x^2`
- [x] 曲线样式设置（颜色、线宽、线型）
- [x] 快速示例按钮

### 阶段 3：代数能力 ✅ 已完成
- [x] 数值求根（牛顿法/二分法）
- [x] 两曲线交点计算
- [x] 交点标记和坐标显示（UI 集成）
- [x] 多项式曲线拟合
- [x] 自定义模型拟合（线性/多项式/指数/幂）
- [x] 拟合结果显示和残差分析（UI 集成）

### 阶段 4：交互增强 ✅ 已完成
- [x] 参数滑块（实时动画）
- [x] 点击查询坐标
- [x] 表达式语法高亮
- [x] 错误提示和建议
- [x] 撤销/重做（Ctrl+Z / Ctrl+Shift+Z）
- [ ] 拖拽调整点/参数（计划中）

### 阶段 5：性能优化和扩展 📋 计划中
- [ ] GPU compute shader 求值
- [ ] 自适应采样优化
- [ ] 图形导出（PNG/SVG）
- [ ] 项目保存/加载
- [ ] 表达式列表管理
- [ ] 键盘快捷键

---

## 支持的语法

| 类型 | 语法示例 | 状态 |
|------|---------|------|
| 显函数 | `y = sin(x)`, `x^2 + 1` | ✅ |
| 隐函数 | `x^2 + y^2 = 4` | ✅ |
| 参数方程 | `[cos(t), sin(t)]`, `[2*cos(t), sin(t)]` | ✅ |
| 极坐标 | `r = sin(3*theta)`, `r = 1 + cos(theta)` | ✅ |
| 不等式 | `y > x^2`, `y <= sin(x)`, `x^2 + y^2 < 4` | ✅ |

## 新增功能 (v0.2)

- **参数滑块**: 参数默认值为1，支持滑块和输入框两种方式调整
- **曲线拟合增强**: 支持点击添加和手动输入坐标两种方式
- **LaTeX输入**: 支持 `\sin{x}`, `\frac{1}{x}`, `\sqrt{x}`, `x^{2}` 等LaTeX语法
- **表达式历史**: 记录已绘制的函数，可快速重新输入

## 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `R` / `0` | 重置视图 |
| `+` / `=` | 放大 |
| `-` | 缩小 |
| 方向键 | 平移 |
| `Q` | 切换坐标查询模式 |
| `C` | 清除查询点 |
| `Escape` | 退出查询/拟合模式 |
| `Ctrl+Z` | 撤销 |
| `Ctrl+Shift+Z` / `Ctrl+Y` | 重做 |

## 运行与测试

```bash
# 运行应用
cargo run --release

# 运行测试
cargo test

# 当前测试覆盖
# 121 tests passed (parser, evaluator, algebra, render, ui)
```
