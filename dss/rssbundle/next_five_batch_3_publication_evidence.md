# 第三批五个 ckt24 MemoryMapping 入口发布证据

## 固定输入与实现边界

- 上游固定为 `dss-extensions/electricdss-tst` 提交
  `3b208397160213cae4a9e2d0a7d1aa3528ce26e1` 的 `Version8/Distrib`。
- 发布 `tensoreval` SHA-256 为
  `68d08ff957223d27027ba5fefdcf413d22fdea950d405bf96afb9a6cf9204544`；构建命令为
  `cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval`。
- 通用实现只增加 LoadShape 直接/嵌入式 `SngFile`、`DblFile` 的小端 IEEE-754 float32/float64
  解码、依赖收集和 bundle 路径改写。空文件、错误字节宽度、非有限值、缺失文件和超过
  10,000,000 点均使用稳定诊断失败关闭，未扩展其他 Shape 类。
- OpenDSS 参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 生成；CSV SHA-256 为
  `67e5d8f8bfc9e5ec4bce0e4a2356f8418c35fbdee5a0e9b6d87d22b14300700e`，连续两次生成
  byte-identical。所有 PSA consumer 显式设置 `line_model=exact`。

## 入口身份与 portable bundle

| 官方入口 | 源 SHA-256 | manifest 文件数 | manifest SHA-256 | 搬移输出 |
|---|---|---:|---|---:|
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24-mm-txt-p.dss` | `bcd0e76fbc8aba0509b8dc93388ade7eab3ece45dc8cb5bd49c7483ff15b09dc` | 25 | `e6e363b1eca76f216982981d1aa77a38248678bacee66573e245fd5e91405a7e` | 752,200×11 |
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24-mm-txt-pq.dss` | `b231f63161a2511c34ef93a8559cbb1bf8ea5f195a1738a7f10d59c2b270db89` | 25 | `2d32d2cbe383fde4caac871002b5f26eab37146130d7406f1b7da4663457d324` | 752,200×11 |
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24-mm-dbl-p.dss` | `e469503699a443459cf92501695288c0e704c984dc53e386c662ea11a09b6f34` | 25 | `48df054e9029068b4148bbb2906a5f544f9a8e4ec13294e033d3850a3f05e197` | 752,200×11 |
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24-mm-sng-p.dss` | `ff9b994520af9e1be6a93592a4abe2481754af54c1521542b5cd7a62656ff5b8` | 25 | `c30883860c6c5ee4a14fdad6a7c01ff8a937b867ea71e98168c2a282760f40b9` | 752,200×11 |
| `Examples/MemoryMappingLoadShapes/ckt24/master_ckt24.dss` | `f27c46d4cc3e8bfb2a0d2aa1ab55dff1a36d63f3921f245f6187f3c4da6a6621` | 23 | `47bec74101f12104399ac97548a1487186ec90556ccf4d76c8f5332cb01900f6` | 752,200×11 |

五个 schema-v2 manifest 均通过 artifact、entry、投影、唯一相对路径、目录闭包和逐文件
SHA-256 校验；实际文件集合严格等于 `.files[]` 加 `manifest.json`。五个 bundle 分别复制到
独立 `/tmp` 父目录并改名后直接执行成功。DBL 及两个 SNG bundle 中的二进制资产与官方源
逐字节相同；提交 `a478c52` 的 123 个 manifest 声明 Git blob 复算 SHA-256 后零不匹配。

## 数值门禁

五个入口均执行官方 `mode=yearly number=100`，最终时刻为 360,000 秒。每例比较全部 7,522 个
原生母线相复电压和 `Vsource.source` 三相 P/Q：

| 案例组 | 最大复电压相对误差 | RMS 复电压相对误差 | 源相 P/Q 最大误差 | 门限 |
|---|---:|---:|---:|---|
| TXT-P / TXT-PQ / DBL-P | `1.9256655491171382e-2` | `2.0890161566336843e-3` | `1.2091782186519844e-2` | `2.5e-2` / `3e-3` / `1.5e-2` |
| SNG-P / legacy SNG | `1.9256655486532877e-2` | `2.0890161535589056e-3` | `1.2091782231033592e-2` | 同上 |

聚焦测试 `cargo test -p psa --features mumps-static --test dss_import next_five_batch_3 -- --nocapture`
为 5/5 通过，参考清单完整性测试为 1/1 通过。目录、文档和父仓库 gitlink 提交后统一运行
`cargo test -p psa --features mumps-static --test dss_import`，结果为 542/542 通过、0 失败、
0 忽略，耗时 504.92 秒。

## 目录结论

`conversion_report.csv` 只把上述五行从 `unsupported` 改为 `converted`。计数由
`171/98/579` 更新为 `176/93/579`，独立入口分母保持 269。该批只证明 100 个一小时时点的
最终电气状态，不声明 8,760 小时性能、跨机器 SLA 或其他未执行研究。
