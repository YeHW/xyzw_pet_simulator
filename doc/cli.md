# CLI 规格

本文件记录 clap 子命令及参数。

## 命令入口

- 使用 `xyzw-petsim target-cost [options]` 运行目标成本模拟。
- 使用 `xyzw-petsim stock-drain [options]` 运行库存耗尽模拟。
- 必须显式指定子命令。

## 当前 target-cost 默认值

- `--target 6`
- `--trials 10000`
- `--threads 1`（`0` 表示自动）
- `--seed 123`
- `--bins 12`
- `--hist-outlier-mode none`
- `--hist-iqr-k 1.5`
- `--hist-mad-threshold 3.5`
- `--hist-quantile-alpha 0.01`
- 默认开启保底，可通过 `--disable-pity` 关闭

## 当前短参数

- `-T` = `--target`
- `-N` = `--trials`
- `-t` = `--threads`
- `-S` = `--seed`
- `-C` = `--csv`
- `-J` = `--output-json`
- `-B` = `--bins`
- `-D` = `--disable-pity`
- `-M` = `--theory-mode`
- `-I` = `--no-interactive`
- `-q` = `--quiet`

## 主命令

- `xyzw-petsim target-cost ...`
- `xyzw-petsim stock-drain ...`

## 子命令：target-cost

用途：给定目标等级，估计获得该等级所需一级蛋消耗分布。

实现状态：业务逻辑与子命令入口均已实现。

建议参数：

- `-T, --target <2..7>`
- `-N, --trials <usize>`
- `-t, --threads <usize>` (`0` 表示自动)
- `-S, --seed <u64>`
- `-D, --disable-pity`
- `-M, --theory-mode <auto|none|no-pity|pity-dp|all>`
- `-B, --bins <usize>`
- `--hist-outlier-mode <none|iqr|mad|quantile|winsor>`
- `--hist-iqr-k <f64>`
- `--hist-mad-threshold <f64>`
- `--hist-quantile-alpha <f64>`
- `-C, --csv [path]`
- `-J, --output-json [path]`
- `-q, --quiet`
- `-I, --no-interactive`

直方图参数校验：

- `--hist-iqr-k` 与 `--hist-mad-threshold` 必须大于 0。
- `--hist-quantile-alpha` 必须满足 `0<=alpha<0.5`。
- 直方图异常值参数仅提供长参数形式。

可复现性：

- 固定 `--seed` 和实际线程数时，模拟结果可复现；这也适用于 `--trials 1`。
- 改变线程数会改变工作线程的随机数流，因此可能得到不同的逐次样本。
- `--threads 0` 依赖机器可用并行度；需要跨机器复现时应显式指定线程数。

## 子命令：stock-drain

用途：给定初始 1-6 级库存，尽可能合成直到无法继续。

实现状态：单次及多线程 Monte Carlo 模拟、终端分位数摘要、CSV/JSON 导出已实现。

当 `--trials > 1` 时，终端输出最终宠物的均值、P5、P10、P20、P30、P40、P50、P60、P70、P80、P90、P95；保底开启时，另输出剩余保底进度和保底使用次数的均值。

已确认参数：

- `--stock <a1,a2,a3,a4,a5,a6>`（必填）
- `-N, --trials <usize>`
- `-t, --threads <usize>`
- `-S, --seed <u64>`
- `-D, --disable-pity`（默认保底开启）
- `-C, --csv [path]`
- `-J, --output-json [path]`
- `-q, --quiet`

`--csv` 与 `--output-json` 可不带路径；此时使用包含库存、试验数、实际线程数、保底开关和种子的默认文件名。

## stock-drain 命令示例

```powershell
# 运行 10000 次模拟，使用 4 个线程
.\target\release\xyzw-petsim.exe stock-drain --stock 200,2,0,0,0,3 -N 10000 -t 4 -S 111

# 自动选择线程数，关闭保底并使用精简输出
.\target\release\xyzw-petsim.exe stock-drain --stock 200,2,0,0,0,3 -N 10000 -t 0 -S 111 -D -q

# 导出逐次矩阵 CSV 和汇总 JSON，使用默认文件名
.\target\release\xyzw-petsim.exe stock-drain --stock 200,2,0,0,0,3 -N 10000 -t 4 -S 111 -C -J
```

## 帮助信息

- 顶层 `--help` 仅展示子命令。
- 子命令帮助展示各自参数。
- `target-cost --no-interactive` 仅禁止 CSV 导出询问；显式传入 `--csv` 或 `--output-json` 仍会导出。

## target-cost 命令示例

以下示例使用目标等级 6、样本 200000、4 线程、种子 111、50 个桶：

```powershell
# 完整线性直方图，不处理异常值
.\target\release\xyzw-petsim.exe target-cost -T 6 -N 200000 -I -t 4 -S 111 -B 50 --hist-outlier-mode none

# IQR 去异常值
.\target\release\xyzw-petsim.exe target-cost -T 6 -N 200000 -I -t 4 -S 111 -B 50 --hist-outlier-mode iqr --hist-iqr-k 1.5

# MAD 去异常值
.\target\release\xyzw-petsim.exe target-cost -T 6 -N 200000 -I -t 4 -S 111 -B 50 --hist-outlier-mode mad --hist-mad-threshold 3.5

# 双侧各裁剪 1%
.\target\release\xyzw-petsim.exe target-cost -T 6 -N 200000 -I -t 4 -S 111 -B 50 --hist-outlier-mode quantile --hist-quantile-alpha 0.01

# 双侧各截帽 1%，不减少样本数
.\target\release\xyzw-petsim.exe target-cost -T 6 -N 200000 -I -t 4 -S 111 -B 50 --hist-outlier-mode winsor --hist-quantile-alpha 0.01
```
