# 插件化开发实例之配电网潮流计算软件

## 1. 项目简介
### 1.1 项目背景

## 2. 拓扑分析
###  2.1 静态拓扑分析

输入:
1. 电气岛
2. 属性定义
3. 资源定义

输出:

- 默认输出下面六列
<table>
    <th>source</th>
    <th>target</th>
    <th>id</th>
    <th>type</th>
    <th>open</th>
    <th>name</th>
</table>

- terminal_cn_dev，如果输出的边是该名称，则输出下面四列
<table>
    <th>terminal</th>
    <th>cn</th>
    <th>dev</th>
    <th>type</th>
</table>

- point_terminal_phase： 如果输出的边是该名称，则输出下面三列
<table>
    <th>point</th>
    <th>terminal</th>
    <th>phase</th>
</table>

###  2.2 动态拓扑分析

输入:
1. 电气岛
2. 属性定义
3. 资源定义


## 3. 输入参数准备
### 3.1 设备电气参数计算
### 3.2 潮流已知量输入

## 4. 潮流计算
###  4.1 潮流计算模型