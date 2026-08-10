# xyzw_pet_simulator

基于 Rust 的宠物合成 Monte Carlo 模拟器，支持目标成本和库存耗尽两种模式。

## 构建与运行

需要稳定版 Rust 工具链。开发构建：

```bash
cargo build
cargo run -- --help
```

发布构建：

```bash
cargo build --release
./target/release/xyzw-petsim --help
```

## 快速开始

估计获得 6 级宠物所需的一级蛋数量：

```bash
cargo run --release -- target-cost -T 6 -N 10000 -t 4 -S 123 -I
```

从指定 1-6 级库存开始合成，直到无法继续：

```bash
cargo run --release -- stock-drain --stock 200,2,0,0,0,3 -N 10000 -t 4 -S 123
```

使用 `-C` 和 `-J` 可按默认文件名导出 CSV 和 JSON；也可在参数后显式提供路径。完整选项请运行对应子命令的 `--help`。

## 可复现性

固定随机种子和实际线程数时，逐次模拟结果可复现。改变线程数会改变随机数流分配；跨机器复现时请显式指定 `--threads`，不要依赖自动线程数。

## 开发检查

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

GitHub Actions 会在推送到 `main` 或针对 `main` 的 pull request 上执行等价检查。

## 文档导航

- [规则总览](doc/rules.md)
- [CLI 规格](doc/cli.md)
- [target-cost 模拟](doc/target-cost-simulation.md)
- [stock-drain 模拟](doc/stock-drain-simulation.md)
- [输出格式规范（CSV/JSON）](doc/output-spec.md)
- [版本与兼容性策略](doc/versioning.md)

## 模拟模式

- `target-cost`：根据目标等级估计一级蛋消耗，已实现
- `stock-drain`：单次及多线程 Monte Carlo 模拟、CSV/JSON 导出已实现
- CLI 使用 clap 子命令模式，调用时必须指定模拟模式
