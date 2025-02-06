# 插件化开发实例之配电网潮流计算软件

## 1. 项目简介
### 1.1 项目背景

## 2. 拓扑分析
###  2.1 静态拓扑分析

#### 输入

1. 电气岛
2. 属性定义
3. 资源定义

#### 输出

- static_topo，如果输出的边是该名称，或者输出的边不是下面两种情况，输出静态拓扑，即下面六列
<table>
    <th>source</th>
    <th>target</th>
    <th>id</th>
    <th>type</th>
    <th>open</th>
    <th>name</th>
</table>

- terminal_cn_dev，如果输出的边是该名称，则输出下面几列
<table>
    <th>terminal</th>
    <th>cn</th>
    <th>dev</th>
    <th>type</th>
</table>

- point_terminal_phase： 如果输出的边是该名称，则输出下面几列
<table>
    <th>point</th>
    <th>terminal</th>
    <th>phase</th>
</table>

###  2.2 动态拓扑分析

#### 输入

- 电气岛
- 量测
- 静态拓扑：上述输出的三个表格

#### 输出

- dyn_topo: 如果输出的边是该名称，或者不是下面的情况，默认输出下面几列
<table>
    <th>cn</th>
    <th>tn</th>
</table>

- dev_topo: 如果输出的边是该名称，则输出下面几列
<table>
    <th>terminal</th>
    <th>cn</th>
    <th>tn</th>
    <th>dev</th>
</table>

## 3. 输入参数准备
### 3.1 设备电气参数计算
#### 输入
- 电气岛
- 配置表格，第1列是config的key，第二列是json格式的矩阵

<table>
    <th>config</th>
    <th>ohm_per_km</th>
</table>

#### 输出

<table>
    <th>dev_id</th>
    <th>ohm</th>
</table>

### 3.2 潮流已知量输入
#### 输入
- static_topo
- terminal_cn_dev
- point_terminal_phase
- dev_topo
- shunt_meas: 如果想要输出tn_input，需要此项

#### 输出

- shunt_meas: 如果输入的边有static_topo，则输出下面几列
<table>
    <th>point</th>
    <th>terminal</th>
    <th>phase</th>
</table>

- tn_input: 如果输出的边是该名称，则输出下面几列
<table>
    <th>tn</th>
    <th>phase</th>
    <th>unit</th>
    <th>value</th>
</table>

## 4. 潮流计算
###  4.1 潮流计算模型