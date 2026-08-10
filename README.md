# xyzw_pet_simulator

项目文档入口。当前文档采用单层 `doc/` 目录。

## 文档导航

- [规则总览](doc/rules.md)
- [CLI 规格](doc/cli.md)
- [target-cost 模拟](doc/target-cost-simulation.md)
- [stock-drain 模拟](doc/stock-drain-simulation.md)
- [输出格式草案（CSV/JSON）](doc/output-spec.md)

## 模拟模式

- `target-cost`：根据目标等级估计一级蛋消耗，已实现
- `stock-drain`：单次及多线程 Monte Carlo 模拟、CSV/JSON 导出已实现
- CLI 使用 clap 子命令模式，调用时必须指定模拟模式
