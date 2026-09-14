# PF 性能对比

本目录记录 TensorEval/rspower 与 MATPOWER 7.1 的 PF 性能对比，以及相关 OPF/NLP 耗时。数据更新于 2026-09-14。

## 测试口径

- 所有表格保存正式样本的中位数，不包含预热时间。
- Rust 和 MATLAB 都固定到 Intel Core i9-14900 的同一个 P-core，BLAS/OpenMP 限制为单线程；MATLAB 使用 `-singleCompThread`。
- PF 总耗时和分阶段测试：预热 5 次，正式测量 21 次。
- TensorEval PF 使用 MUMPS + OpenBLAS；MATPOWER 总耗时使用 `runpf` 默认选项，分阶段对比固定使用 LU3。
- TensorEval NLP 使用 Ipopt + MKL/PARDISO MKL。除 case13659pegase 使用 5 个正式样本外，其他 NLP 算例使用 21 个正式样本。
- 拼接和索引测试预热 10 轮，记录 31 个批次平均耗时的中位数。

## 总体结果

TensorEval PF 在 13 个算例中有 11 个快于 MATPOWER。case5 慢 6.8%，case118 慢 3.9%，其余算例持平或更快。

| 算例 | MATPOWER PF（ms） | TensorEval PF/MUMPS（ms） | TensorEval/MATPOWER |
| --- | ---: | ---: | ---: |
| case300 | 3.634 | 3.401 | 0.936 |
| case1354pegase | 10.777 | 9.374 | 0.870 |
| case2737sop | 23.033 | 18.091 | 0.785 |
| case13659pegase | 129.037 | 123.579 | 0.958 |

全部算例及 OPF/NLP 结果见 [pf_performance_tensoreval_vs_matpower.csv](pf_performance_tensoreval_vs_matpower.csv)。`MATPOWER runopf` 与 `TensorEval NLP` 使用不同实现，该两列用于观察当前端到端耗时，不应解读为线性求解器的单项对比。

## case13659pegase 分析

case13659pegase 的分阶段总耗时已基本一致：TensorEval 为 128.460 ms，MATPOWER 为 128.798 ms。两者的主要差异如下。

| 阶段 | TensorEval（ms） | MATPOWER（ms） | TensorEval/MATPOWER |
| --- | ---: | ---: | ---: |
| dSbus_dV | 7.988 | 20.028 | 0.399 |
| 索引与拼接 | 10.705 | 14.307 | 0.748 |
| 左除 | 86.388 | 65.817 | 1.313 |
| makeSbus | 2.082 | 0.638 | 3.263 |
| pfsoln | 5.493 | 3.024 | 1.816 |

左除占 TensorEval 总耗时的 67.2%，是当前最主要的单项差距。MUMPS 完整调用为 16.616 ms，MATPOWER LU3 为 13.030 ms，MUMPS 慢 27.5%。MUMPS 内部约包括 7.423 ms 符号分析、8.354 ms 数值分解和 1.039 ms 回代。详细数据见 [pf_linear_solver_performance_mumps_vs_matpower.csv](pf_linear_solver_performance_mumps_vs_matpower.csv)。

makeSbus 的倍数较大，但绝对差值只有约 1.44 ms。`Cg * Sg` 稀疏乘法为 0.009 ms，MATPOWER 为 0.010 ms，乘法内核不是问题。其余差距主要来自索引向量构造、初始无功更新和结果写回：

- pfsoln 支路索引：1.029 ms 对 0.103 ms。
- pfsoln 初始无功更新：0.824 ms 对 0.160 ms。
- pfsoln 支路结果写回：0.501 ms 对 0.207 ms。
- int2ext 支路编号写回：0.915 ms 对 0.187 ms。

完整分阶段数据见 [pf_stage_performance_tensoreval_vs_matpower.csv](pf_stage_performance_tensoreval_vs_matpower.csv)。

## 拼接、索引与稀疏格式

| 内核 | TensorEval（μs） | MATLAB（μs） | TensorEval/MATLAB |
| --- | ---: | ---: | ---: |
| 稠密块拼接，1024×1024 | 272.318 | 862.250 | 0.316 |
| 稀疏块拼接，13659×13659 | 265.239 | 548.300 | 0.484 |
| 稠密二维索引，取 75% 行列 | 199.611 | 281.650 | 0.709 |
| 稀疏二维索引，取 75% 行列 | 356.496 | 523.100 | 0.682 |

当前稀疏主表示是按 `(row, column)` 排序的规范三元组；稀疏乘法临时转为 CSR，行索引临时构造行偏移。二维索引和块拼接已快于 MATLAB，也不是 PF 主要瓶颈，因此暂不增加常驻 CSR。若以后的 profiling 显示行访问或 COO→CSR 转换成为热点，优先考虑可缓存的 `row_ptr` 视图。

## 文件说明

### 归档 CSV

- [pf_performance_tensoreval_vs_matpower.csv](pf_performance_tensoreval_vs_matpower.csv)：各规模算例的 PF、MATPOWER runopf 和 TensorEval NLP 总耗时。
- [pf_stage_performance_tensoreval_vs_matpower.csv](pf_stage_performance_tensoreval_vs_matpower.csv)：三个代表算例的 PF 核心阶段耗时。
- [pf_linear_solver_performance_mumps_vs_matpower.csv](pf_linear_solver_performance_mumps_vs_matpower.csv)：MUMPS、MATPOWER LU3 和 MATLAB 默认左除的线性求解对比。

### MATLAB 基准脚本

- [benchmark_matpower_pf_opf.m](benchmark_matpower_pf_opf.m)：测量所有算例的 MATPOWER `runpf`/`runopf` 总耗时。
- [benchmark_matpower_pf_breakdown.m](benchmark_matpower_pf_breakdown.m)：强制 LU3，测量 PF 分阶段耗时。
- [benchmark_matpower_pf_solve_phases.m](benchmark_matpower_pf_solve_phases.m)：拆分 LU3 分解、回代和默认左除耗时。
- [benchmark_matpower_remaining_breakdown.m](benchmark_matpower_remaining_breakdown.m)：进一步诊断 makeSbus、pfsoln 和 int2ext 的子阶段。
- [benchmark_blockcat_select.m](benchmark_blockcat_select.m)：测量 MATLAB 块拼接、horzcat/vertcat 和二维索引内核。

### Rust 基准入口

- [benchmark_pf_breakdown.rs](../../../eig-expr/tests/benchmark_pf_breakdown.rs)：TensorEval PF 总耗时、分阶段耗时和 MUMPS 阶段耗时。
- [benchmark_blockcat_select.rs](../../../eig-expr/tests/benchmark_blockcat_select.rs)：TensorEval 拼接与索引内核。
- [benchmark_case2737_opf.rs](../../../eig-aoe/tests/benchmark_case2737_opf.rs)：TensorEval NLP 多样本耗时。
