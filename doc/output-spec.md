# 输出格式规范（CSV/JSON）

本文档描述当前导出的字段和默认文件名。字段名属于对脚本和下游工具可见的接口；修改时应同步更新导出测试和本文档。

## target-cost

CSV（样本）：

- `trial,eggs`
- `trial` 从 1 开始，按试验执行顺序编号；`eggs` 为该次试验消耗的一级蛋数量。

JSON（摘要）：

- `config`：目标等级、保底开关、理论模式、种子、完整命令参数和导出路径
- `result`：`trials/threads/mean/std_dev/ci95_low/ci95_high/min/p50/p90/p95/max`
- `theory`：无保底精确值、有保底近似值和相对误差；未输出项为 `null`

默认文件名包含运行参数，例如：

- `samples_t7_n100000_th4_pon_s123.csv`
- `report_t7_n100000_th4_pon_s123.json`

`--csv` 与 `--output-json` 可不带路径；此时使用默认文件名。

## stock-drain

CSV（逐次状态矩阵）：

通用列：

- `trial,level1,level2,level3,level4,level5,level6,level7`

保底开启追加列：

- `pity2,pity3,pity4,pity5,pity6,pity7`
- `pity_used2,pity_used3,pity_used4,pity_used5,pity_used6,pity_used7`

JSON（摘要）：

- `config`：`stock/enable_pity/seed/command/output_csv_path/output_json_path`
- `result`：`trials/threads/pets_by_level/pity`
- `pets_by_level`：1-7 级各自的 `level/mean/min/p5/p10/p20/p30/p40/p50/p60/p70/p80/p90/p95/max`
- `pity.progress_by_level`：2-7 级结束保底进度的同类统计
- `pity.used_by_level`：2-7 级保底使用次数的同类统计
- 无保底时 `pity` 为 `null`

默认文件名包含库存、试验数、实际线程数、保底开关与种子，例如：

- `samples_stock_200-2-0-0-0-3_n10000_th4_pon_s111.csv`
- `report_stock_200-2-0-0-0-3_n10000_th4_pon_s111.json`

`--csv` 与 `--output-json` 可不带路径；此时使用默认文件名。
