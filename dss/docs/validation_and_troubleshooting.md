# 验证与故障排查

RSS 脚本的验证分三层：语法和依赖图、DataFrame/分析契约、与独立基准的数值一致性。
“脚本能运行”只覆盖前两层；迁移 OpenDSS 工作时还必须完成第三层。

## 1. 最小验证流程

先运行脚本：

```bash
cargo run -p mems \
  --features tensoreval-cli,with_dsa \
  --bin tensoreval -- \
  --df sparrowzz/dss/ieee13_power_flow.rss
```

再运行仓库中的 OpenDSS 对照测试：

```bash
cargo test -p mems --features with_dsa --test main \
  dff::dff_script::dsa_tests
```

谐波整网测试在 debug 下可能很慢。包含 IEEE/CIGRE 谐波基准的整组检查应使用：

```bash
cargo test --release -p mems --features with_dsa --test main \
  imports_ieee13_dss_and_runs_harmonics
```

Newton QSTS、Newton PF 和状态估计改用项目可用的 `with_dsa_mumps` 或
`with_dsa_mkl`。不要同时启用两个后端。

## 2. 当前案例的对照规则

`mems/tests/dff/dff_script.rs` 直接执行 `sparrowzz/dss/*.rss`，并复用
`psa/tests/bmopf` 的 OpenDSS CSV 基准。当前规则为：

| 案例 | 比较量 | 误差定义 | 容差 |
|---|---|---|---:|
| IEEE13/34/123 静态潮流 | 每个 bus/phase 的复电压 | `abs(actual-reference) / max(abs(reference), 1)` | `1e-3` |
| IEEE13/34 谐波 | 每个 harmonic/bus/phase 的复电压 | 同上 | `< 1e-3` |
| IEEE13 短路 | `voc`, `v_f`, `i_f` 的第一参与相复相量 | 同上 | `< 5e-3` |
| IEEE34 短路 | 同上 | 同上 | `< 1e-4` |
| 4Bus Daily QSTS | 每个 sample-time/bus/phase 的复电压 | 同上 | `< 1e-3` |

因此 dff_script 与 PSA 使用同一批 OpenDSS fixture，但“所有 PSA 测试共用一个容差”并不
成立。PSA 不同对象族和量测有各自的 `QuantitySpec` 与 tolerance。新增 RSS 案例时，应复用
它所对应的 PSA 基准和误差定义，而不是复制某个无关案例的阈值。

QSTS 的结果时间是步末，曲线在步首采样。4Bus 对照把输出 `time` 减去 `step_size_s` 后再
与 OpenDSS 的 sample time 连接。这是当前时间契约，不应通过平移 CSV 来掩盖 off-by-one。

## 3. 生成可靠 OpenDSS 基准

1. 保留同一份声明式 `.dss` 网络；OpenDSS 运行命令放在独立 Python/driver 中。
2. 固定 OpenDSS 版本、control mode、tap/state、负荷电压模型和求解模式。
3. 以复数实部/虚部导出原始量，避免先舍入幅值和角度。
4. 保存稳定键：母线为 `lower(bus), phase`，谐波再加 `harmonic`，时间序列再加采样时刻。
5. 对所有预期键做一一覆盖检查；不能只比较两个结果的交集。
6. 在误差报告中保存最大误差键、actual、reference、误差公式和阈值。

角度比较前要统一单位。PF `bus_phase.v_ang` 使用 rad；谐波输出
`voltage_angle` 使用 degree。OpenDSS 的端口电流通常以流入元件为正，比较线路末端电流前
要按 `branch_flow` 的方向约定转换符号。

## 4. 解析和编译错误

### `expected ...`、语句无法解析

- 每条赋值/调用以 `;` 结束；
- 多返回值写成 `[a, b] = function(...)`；
- 注释使用 `//` 或 `/* ... */`；
- `#include` 是预处理指令，不是普通函数调用。

### unknown variable / frame

RSS 图按变量依赖求值。裸标识符在表达式参数中常被翻译为字符串字面量；DataFrame 位置的
名称则必须已由输入或前一条语句定义。列引用使用 `col(name)`，多列选择使用 `cols(a,b)`。

### 返回数量不匹配

`para.result` 决定 DSA 多返回值的数量和顺序。若写入
`"bus_phase,summary"`，左侧必须正好有两个绑定：

```text
[voltage, summary] = ds_radial_pf(..., para);
```

## 5. DataFrame schema 错误

### missing column / invalid row,column

先定位函数的位置参数，再对照 [DataFrame schema 参考](dataframe_schema_reference.md)。完整
网络形式的前 20 张表是固定顺序；QSTS 另外 13 张表也是固定顺序。零行表可以占可选位置，
非空的错误表会进入解析器并在具体行列失败。

### 类型无法转换

常见原因包括：

- 本应是 `list<str>` 的 `terminal_map` 被构造成单个字符串；
- `tcc_curve.c_array/t_array` 不是列表或长度与 `npts` 不等；
- `number`, `shots`, `terminal` 等整数语义由浮点或字符串产生了非整数值；
- 同一 `load_shape` 的 `period_s/interpolation/use_actual` 不一致；
- 用 null 表示缺省，但该列的解析器要求有限数值。

### unknown/duplicate result

`para.result` 会转为小写并按逗号拆分。空项、未知名和重复名都会失败。可用结果必须从
[DSA 函数参考](dsa_function_reference.md) 选择。

## 6. DSS 导入错误

导入器采用 fail-closed：有错误诊断时不返回部分网络。优先查看稳定 code 和 category，
不要依赖 message 文本。

| category/code 类型 | 含义 | 处理 |
|---|---|---|
| `UnknownCommand` | 上游 parser 未识别语句 | 检查拼写、版本或拆出执行命令 |
| `UnsupportedCommand` | 命令存在但不属于当前 profile | 把 Solve/Export 等改为 RSS `ds_*` 和后处理 |
| `UnsupportedObject/Property` | 无无损 DataFrame 映射 | 查兼容矩阵，显式构造表或缩小模型 |
| `InvalidTopology` | 引用、端子或几何无法组成网络 | 检查对象名、相数和端子映射 |
| `IncludeError` | include 路径、深度或预算失败 | 使用本地入口，检查相对路径和循环引用 |
| `UnresolvedReference` | 曲线、设备或控制目标不唯一/不存在 | 使用完整对象名并消除大小写后重名 |
| `UnrepresentableStateTransition` | 有序运行时动作无法映射 | 改用显式 QSTS 表或等待对应 adapter 公开 |

`import_dss_url` 只获取一个远程 source，不能假定它拥有本地目录中的 Redirect 文件树。

## 7. 求解和数值差异

### 不收敛

先请求 `summary` 并缩小到静态潮流。检查电源、孤岛、开关状态、变压器 tap、负荷功率和
line model。Radial 与 Newton 都失败时，优先检查模型；只有一种失败时，再检查该算法的
拓扑假设和后端。

### 电压整体偏差

依次检查：源电压/相角、变压器连接和额定基准、线路单位、负荷 W/var 换算、tap 与
控制器初态。不要先扩大容差。

### 个别相或末端符号相反

检查 `terminal_map_*` 和 delta/wye 连接。结果连接必须使用 bus/phase 或
element/terminal/phase 稳定键，不能按行号比较。

### QSTS 第一个点错位

`solution.hour/seconds` 是步进前的时钟，曲线在这个时刻采样，首个结果标记为
`start + step_size_s`。检查曲线 `time_s`、period/interpolation 和测试中的时刻映射。

### 谐波很慢

使用 release；只计算需要的 `h_max`；先在 IEEE13 验证频谱和输出，再扩到更大馈线。
debug 下 CIGRE-LV 整网谐波可能需要约 30 分钟，不应据此判断死锁。

## 8. 新脚本合入前检查表

- `.rss` 中解释了模型、分析、关键参数和输出；
- 使用完整网络参数，没有依赖含糊的 legacy 短形式；
- 所有位置参数与 DSA reference 一致；
- OpenDSS 与 RSS 使用同一电气假设和单位；
- 基准按稳定键覆盖全部预期行；
- 容差来自同类 PSA 测试或有明确的数值依据；
- 直接 `tensoreval --df` 运行通过；
- 对应 `mems` 集成测试通过；
- 大型谐波测试在 release 下运行；
- 文档、案例和机器可读 catalog 同步更新。
