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
    <tr>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>UInt32</td>
        <td>Boolean</td>
        <td>Utf8</td>
    </tr>
</table>

- terminal_cn_dev，如果输出的边是该名称，则输出下面几列
<table>
    <th>terminal</th>
    <th>cn</th>
    <th>dev</th>
    <th>type</th>
    <tr>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>UInt32</td>
    </tr>
</table>

- point_terminal_phase： 如果输出的边是该名称，则输出下面几列
<table>
    <th>point</th>
    <th>terminal</th>
    <th>phase</th>
    <tr>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>Utf8</td>
    </tr>
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
    <tr>
        <td>UInt64</td>
        <td>UInt64</td>
    </tr>
</table>

- dev_topo: 如果输出的边是该名称，则输出下面几列
<table>
    <th>terminal</th>
    <th>cn</th>
    <th>tn</th>
    <th>dev</th>
    <tr>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>UInt64</td>
    </tr>
</table>

## 3. 输入参数准备
### 3.1 设备电气参数计算
#### 输入
- 电气岛
- 配置表格，第1列是config的key，第二列是json格式的矩阵

<table>
    <th>config</th>
    <th>ohm_per_km</th>
    <tr>
        <td>Utf8</td>
        <td>Utf8</td>
    </tr>
</table>

#### 输出

<table>
    <th>dev_id</th>
    <th>ohm</th>
    <tr>
        <td>UInt64</td>
        <td>Utf8</td>
    </tr>
</table>

### 3.2 潮流方程准备脚本
#### shunt_meas

需要输入以下两个表格
- terminal_cn_dev
- point_terminal_phase

输出下面几列
<table>
    <th>point</th>
    <th>terminal</th>
    <th>phase</th>
    <tr>
        <td>UInt64</td>
        <td>UInt64</td>
        <td>Utf8</td>
    </tr>
</table>

#### tn_input

需要输入以下两个表格
- dev_topo
- shunt_meas

输出下面几列
<table>
    <th>tn</th>
    <th>phase</th>
    <th>unit</th>
    <th>value</th>
    <tr>
        <td>UInt64</td>
        <td>Utf8</td>
        <td>Utf8</td>
        <td>Float64</td>
    </tr>
</table>

## 4. 潮流计算
###  4.1 潮流计算模型