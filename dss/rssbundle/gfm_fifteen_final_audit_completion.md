# GFM 十五例最终统一审计摘要

状态：Item 11 已完成，固定 GFM 清单达到 15/15，第 1—11 项全部完成。

本摘要同步父仓库机器记录 `psa/benchmarks/gfm_fifteen_final_audit.json` 的可独立核对信息，避免
本仓库 README 使用越出仓库的相对链接。机器记录 schema 为
`psa.gfm-fifteen-final-audit.v2`，文件 SHA-256 为
`e6c012400dee197177c9541964454d50bba52b3780a85920a7862f55e69aadf2`。

## 固定身份与结果

- 被审计 bundle 提交：`9c85a31e7898871091595a53682044bf8bc4bebb`；
- 父仓库审计实现提交：`69b1b3bfc8ed743cba25f4efba2c2a3c9ca8fec9`；
- 上游源码提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`；
- 十五个入口的固定集合 SHA-256：
  `5d92da7d4e9c81718fb3aa66ead26390759fc9da6eb970f14209ac20e0d7e5b0`；
- canonical `tensoreval` SHA-256：
  `9439e4daa95b67dd38b8a3c9c1297aa6c06965cc4f488e685286118280585e23`；
- case spec SHA-256：
  `9053c828e67ba5c8961788a2bf3183225bcc9f3338c93a9f4074487a5154c506`；
- 工具链：`rustc 1.99.1 (58cf8f9c3 2026-10-13)`、
  `cargo 1.99.1 (5f94df478 2026-08-27)`；
- 构建命令：
  `cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval`。

十五例从同一清洁固定源树重建两轮。每轮均完成 15 次原位执行和 15 次改名搬移执行，两轮
合计 60 次 bundle 执行；journal 的 270/270 个阶段全部通过。每轮公开状态合计为 1200 个
`ENERGIZED_CONVERGED`、8 个 `DEENERGIZED_NO_REFERENCE` 和 4 个
`GFM_CAPACITY_INSUFFICIENT`。候选树与发布树逐例一致，原位和搬移结果逐文件 SHA-256 一致。

catalog 保持 848 行，其中 202 个入口已发布、68 个独立入口仍为 unsupported、578 个文件不是
独立入口；十五个 GFM 目标全部属于 202 个已发布入口。

## 保留边界

全部十五例仍明确标识 `psa_gfm_approx_v1` 和 `numerical_parity=false`。该模型用于执行固定
OpenDSS 示例的 PSA 有界近似，不等价于 OpenDSS 原生 GFM 数值轨迹，不是 EMT，也不声明完整
OpenDSS Dynamic Mode、内部控制器或 IEEE 1547 认证能力。
