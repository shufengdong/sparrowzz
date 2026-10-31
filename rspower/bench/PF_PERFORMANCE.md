# PF/OPF 性能记录

2026-10-27～28，读取相同的 `rspower/data/*.txt`。现有[汇总CSV](pf_performance_tensoreval_vs_matpower.csv)的表头、列顺序保持原样。

## 测量口径

i9-14900的CPU0 P-core（最高5.8GHz），Rust release、MUMPS/OpenBLAS；MATLAB R2022a Update1、MATPOWER7.1。固定同一核心，RAYON/OPENBLAS/OMP/MKL/BLIS/POLARS六个线程变量均为1，MATLAB使用 `-singleCompThread`。每个成功算例20次预热、21次正式样本，取中位数；原始样本全部保留。正式计时期间没有并行构建、测试或其他测量。

- PF：NR极坐标，误差阈值1e-6标幺，最多100次迭代，无Q限切换；读取、编译和输入初始化在计时外，计时覆盖生产runpf的建模、求解与结果还原。
- OPF：通用AC极坐标、双端S²限额，主要最优性/可行性容差1e-9，最多300次迭代。CSV的MATPOWER runopf列为MIPS，最后一列为Rust脚本/IPOPT3.14.20；两者均包含建模、求解、还原，排除源码编译和文件读取。Rust执行还包含输入字面量初始化。
- NLP/MKL列保留2026-09-14的符号模型、Ipopt+MKL/PARDISO历史数据，未在本轮重测；case13659有5个样本，其余21个。“无算例”也是历史记录。该列与当前脚本OPF使用不同入口和后端。

## 当前PF

单位ms，单元格为中位数/P90（21个样本的90百分位）。Rust改前/后使用相同新benchmark、相同输入和正确的发电机行映射；只切换本轮通用计算实现。

| 算例 | Rust改前 | Rust当前/P90 | MATPOWER当前/P90 | NR迭代 |
|---|---:|---:|---:|---:|
| case5 | 0.558 | 0.556 / 0.568 | 0.566 / 0.827 | 3 |
| case9 | 0.571 | 0.551 / 0.574 | 0.614 / 3.739 | 3 |
| case14 | 0.521 | 0.522 / 0.537 | 0.551 / 0.713 | 2 |
| case30 | 0.754 | 0.715 / 0.731 | 0.775 / 6.878 | 3 |
| case39 | 0.531 | 0.511 / 0.529 | 0.559 / 6.600 | 1 |
| case57 | 0.878 | 0.870 / 0.880 | 0.897 / 3.982 | 3 |
| case118 | 1.197 | 1.175 / 1.183 | 1.219 / 5.988 | 3 |
| case118zh | 0.957 | 0.956 / 0.977 | 0.935 / 0.954 | 3 |
| case300 | 3.223 | 3.193 / 3.212 | 7.069 / 12.528 | 5 |
| case1354pegase | 9.482 | 9.375 / 9.424 | 10.346 / 23.549 | 4 |
| case2736sp | 17.861 | 17.757 / 18.134 | 18.846 / 18.989 | 4 |
| case2737sop | 18.719 | 17.811 / 18.168 | 18.976 / 37.532 | 4 |
| case13659pegase | 127.340 | 126.282 / 127.221 | 124.921 / 132.334 | 5 |

13项PF均成功，两端迭代数相同；Rust独立节点功率平衡最大误差3.43e-7标幺。当前Rust中位数11项低于MATPOWER，case118zh及case13659略高；小差值不足以认定稳定优势。

MATLAB第一轮case300由3.4ms波动至16.9ms，追加完整13算例同协议复测。第一轮/复测的case300中位数为9.421/7.069ms，复测P90仍为12.528ms；本表统一采用整轮复测，未逐项挑选较小值。两轮都留档，这项差距不能解释为稳定的语言速度优势。MATLAB函数来源已核对为安装目录中的MATPOWER7.1。

本轮还修复了 `pfsoln` 对停运发电机的行映射：原代码把“在线列表中的位置”当作“原发电机行号”，case2736虽然返回成功，输出平衡误差却达0.569标幺。现已修复，并增加在在线机组前插入停运机组的回归；改前/后性能均使用修正后的有效结果。

## 当前OPF与内存

11项两端均成功，目标值绝对差小于2e-6。case118zh：MIPS第3次迭代失败，Rust第23次返回-2；case13659：MIPS第46次失败，Rust达到300次上限。失败状态不能当作成功耗时，也不据此断言模型不可行。逐项耗时已填写CSV末列，数值、P90、阶段和内存见[OPF记录](../../../eig-aoe/benches/opf_performance.md)。

本轮普通稀疏乘法去掉一次已拥有结果的深复制，4096阶算子快约11%～14%，累计申请少约13%～15%，峰值不变；tensor Jacobian中间数值仅在一次调用内复用。PSA单次F/J因此增加约0.3%～1.4%的释放成本。具体时间和内存见[通用算子记录](../../../eig-expr/benches/autodiff_performance.md)。

## 复现与文件

```sh
export RAYON_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1
export MKL_NUM_THREADS=1 BLIS_NUM_THREADS=1 POLARS_MAX_THREADS=1
taskset -c 0 cargo bench -p eig-expr --locked --features mumps-static --bench rspower_pf_benchmark -- all
taskset -c 0 cargo bench -p eig-aoe --locked --features mumps-static --bench opf_benchmark -- case300 --normal-only
taskset -c 0 matlab -singleCompThread -batch "addpath('sparrowzz/rspower/bench'); benchmark_matpower_pf();"
```

MATLAB的[PF](benchmark_matpower_pf.m)/[OPF](benchmark_matpower_opf.m)共用[TXT读取](read_rspower_case.m)。Rust基准分别为[rspower_pf_benchmark](../../../eig-expr/benches/rspower_pf_benchmark.rs)、[opf_benchmark](../../../eig-aoe/benches/opf_benchmark.rs)。命令中的数据名可替换；缩短检查用 `--warmups 1 --samples 1`，正式20/21为默认值。

`pf_case30/141/145/1354pegase/13659pegase.txt`共用[函数清单](pf_bench_common.txt)和[预热/计时循环](pf_bench_run.txt)，直接调用生产runpf；删除了复制算法的旧诊断程序。
[PF阶段CSV](pf_stage_performance_tensoreval_vs_matpower.csv)和[线性求解CSV](pf_linear_solver_performance_mumps_vs_matpower.csv)保留2026-09-14归档数据，未作为本轮重测结果。

本轮相关回归：eig-expr/eig-aoe682通过，PSA单元1384通过/2忽略、power_flow268通过。更广DSS导入仍有5项已确认改前也失败的测试，详见OPF记录；未删失败测试。
证据：`/tmp/eig-generic-cleanup-20261027.ciga8gcp/`，before/after保存程序、样本和独立内存数据；matlab-formal.log、matlab-pf-repeat.log保存MATLAB两轮原始样本，summary.json为临时汇总。原始产物不提交。
