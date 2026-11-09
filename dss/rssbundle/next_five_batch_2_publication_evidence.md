# 第二批五个执行入口发布证据

## 固定输入与合同

- 上游固定为 `dss-extensions/electricdss-tst` 提交
  `3b208397160213cae4a9e2d0a7d1aa3528ce26e1` 的 `Version8/Distrib`。
- 发布用 `tensoreval` SHA-256 为
  `53b3358d1b9d7b9e3ed7956d31d1a77e708ffdfbbe061c2db1d916c8be9dac3e`；构建命令显式使用
  `--no-default-features --features tensoreval-cli,with_dsa_mumps`。
- 数值参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 生成；锁定 CSV SHA-256 为
  `1f7fd4fe6bac7d1ace680c0167164d5ad28396ab54f6237562c8f15ca24ccf45`。
- 所有 PSA/OpenDSS 数值 consumer 都显式设置 `line_model=exact`。三个 Ckt24 入口执行 100 个
  Daily 时点；Paulo 入口执行 Snapshot 加 24 个 Daily 时点；Matlab IEEE 519 文件只发布其自身的
  初始化 Snapshot，不声明外部 `Main_Script.m` 驱动的谐波循环等价。

## 入口身份与 portable bundle

| 官方入口 | 源 SHA-256 | manifest 文件数 | manifest SHA-256 | 搬移输出 |
|---|---|---:|---|---:|
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24-nomm.dss` | `8a26e40705c7ecf1cc2bd5de4cec1b831aee87638b37295246c40f1d80d21866` | 25 | `3e6a01b90d54817dfe9b0941b84c790476d9295488d107a2782f92807b77dcd9` | 752,200×11 |
| `Examples/Paulo_Example/DSSFiles/MASTER_RedeTeste13Barras.dss` | `4f89440ea97eaa762baa6463a2c1852296abc80ebbc75b95964b3351594201ac` | 21 | `d357e4e4b92e8f6a386da6e48777d91eaaa56f36f121b44bc81c0047ec6bb632` | 1,025×11 |
| `Examples/Matlab/HarmonicT_MATLAB/IEEE_519.DSS` | `41e09c7d8a032095a2d673a74b07c47ad2f35b21f47f2fb01ce7d62723c60404` | 7 | `656d1b033e6dedbc7cd759a8abe50617a19f85136ce26c33ab47b570c0b815c0` | 24×11 |
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24-mm-csv-p.dss` | `a7f381d4aaab2817d947ea95859c7827b3cdea11d2e79bd2327137d0936f0225` | 25 | `c2a683eabb98ff9f36bbb9dfe87a32d7c5d615b805feb12514bba5fb8ffc6b43` | 752,200×11 |
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24-mm-csv-pq.dss` | `fea128432622113ab6262bf0332347a687340f27e3f2d13bfca76504e0db7134` | 25 | `dc69ec5c7b98484e0435b4d645863e4f4084ae8d3bfdc50a4b7532d32950f89d` | 752,200×11 |

五个 schema-v2 manifest 均通过 artifact、entry、投影、排序唯一相对路径、目录边界、声明文件
SHA-256 和结果表名称校验。实际文件集合严格等于 `.files[]` 加 `manifest.json`，无未声明文件。
五个目录复制到独立 `/tmp` 父目录并改名后直接执行成功。

官方 `AllocationFactors_Base.Txt` 使用 CRLF 且扩展名大小写混合。bundle 的 `.gitattributes` 已补充
`*.Txt -text`；对新提交的全部 103 个 manifest 声明 Git blob 重新计算 SHA-256，零不匹配，确保
fresh checkout 不会因换行归一化破坏 manifest。

## 数值门禁

| 案例/状态 | 电压覆盖 | 最大归一化误差 | RMS 误差 | 源相 P/Q 最大误差 | 门限 |
|---|---:|---:|---:|---:|---|
| Ckt24 non-MM、CSV-P、CSV-PQ 各自最终时点 | 各 7,522 | `1.925665549117e-2` | `2.089016156633e-3` | `1.20917821865e-2` | `2.5e-2` / `3e-3` / `1.5e-2` |
| Paulo Snapshot | 41 | `1.354779913e-2` | `7.4598057985e-3` | `9.8921419858e-2` | `2e-2` / `1e-2` / `1.1e-1` |
| Paulo Daily 最终时点 | 41 | `6.98888e-5` | `1.6077e-5` | `2.728e-5` | 同上 |
| Matlab IEEE 519 初始化 Snapshot | 24 | `1.5376e-7` | `1.2742e-7` | `1.03128e-6` | `1e-6` / `2e-7` / `2e-6` |

每个状态同时锁定三相源端复功率。DSS C-API 0.14.5 在 Linux 上对旧式
`MemoryMapping=Yes` 与 `CSVFile`/`PQCSVFile` 组合触发 SIGFPE，因此两个 MM 原生探针使用完全相同
的官方网络和数据资产，只移除不改变电气样本的存储提示；PSA 另行要求 CSV-P/CSV-PQ 与 non-MM
原生状态通过相同门禁，并要求非法 MemoryMapping 值失败关闭。

## 目录结论

`conversion_report.csv` 只把上述五行从 `unsupported` 改为 `converted`。计数由
`166/103/579` 更新为 `171/98/579`，独立入口分母保持 269。完整逐案例摘要记录在
`numerical_comparison.csv`；源参考生成器与逐点锁定 CSV 位于 PSA 测试资产中。
