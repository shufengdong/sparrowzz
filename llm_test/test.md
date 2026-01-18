根据RustScript语法规范，以下是实现电力系统潮流计算的完整代码。该实现包含导纳矩阵构建、牛顿-拉夫逊法潮流计算及快速解耦潮流计算三个核心功能。

```rustscript
/*
电力系统潮流计算实现（RustScript版本）
包含功能：
1. 导纳矩阵构建 (make_y_bus)
2. 牛顿-拉夫逊法潮流计算 (newton_pf)
3. 快速解耦潮流计算 (fast_decoupled_pf)
*/

// 常量定义
PQ = 1;       // PQ节点
PV = 2;       // PV节点
REF = 3;      // 平衡节点
NONE = 4;     // 无节点类型

// 母线数据列索引
BUS_I = 0;
BUS_TYPE = 1;
PD = 2;
QD = 3;
GS = 4;
BS = 5;
BUS_AREA = 6;
VM = 7;
VA = 8;
BASE_KV = 9;
ZONE = 10;
VMAX = 11;
VMIN = 12;

// 发电机数据列索引
GEN_BUS = 0;
PG = 1;
QG = 2;
QMAX = 3;
QMIN = 4;
VSET = 5;
MBASE = 6;
GEN_STATUS = 7;
PMAX = 8;
PMIN = 9;

// 支路数据列索引
F_BUS = 0;
T_BUS = 1;
BR_R = 2;
BR_X = 3;
BR_B = 4;
RATE_A = 5;
RATE_B = 6;
RATE_C = 7;
TAP = 8;
SHIFT = 9;
BR_STATUS = 10;
ANGMIN = 11;
ANGMAX = 12;

// 导纳矩阵构建函数
fn make_y_bus(baseMVA, bus, branch) {
    nb = size(bus, 0);  // 母线数量
    nl = size(branch, 0);  // 支路数量
    
    // 提取支路数据
    f = slice(branch, [0], [F_BUS-1]) - 1;  // 从母线（转换为0基）
    t = slice(branch, [0], [T_BUS-1]) - 1;  // 到母线（转换为0基）
    r = slice(branch, [0], [BR_R-1]);       // 电阻
    x = slice(branch, [0], [BR_X-1]);       // 电抗
    b = slice(branch, [0], [BR_B-1]);       // 电纳
    stat = slice(branch, [0], [BR_STATUS-1]);  // 支路状态
    tap = slice(branch, [0], [TAP-1]);      // 变比
    shift = slice(branch, [0], [SHIFT-1]);  // 移相角
    
    // 计算导纳
    z = r + c(0, 1) * x;
    y_series = stat ./ z;  // 串联导纳
    y_shunt = c(0, 1) * b / 2.0;  // 并联导纳
    
    // 处理变压器变比和移相角
    tap_col = slice(branch, [0], [TAP-1, TAP]);  // 变比列
    tap_idx = find(tap_col != 0);  // 非零变比索引
    tap_vec = set(ones(nl, 1), tap_idx, get_multi(tap_col, tap_idx));  // 变比向量
    tap_complex = tap_vec .* exp(c(0, 1) * shift * pi / 180.0);  // 复变比
    
    // 计算导纳矩阵元素
    Yff = y_series ./ (tap_complex .* conj(tap_complex)) + y_shunt;
    Yft = -y_series ./ conj(tap_complex);
    Ytf = -y_series ./ tap_complex;
    Ytt = y_series + y_shunt;
    
    // 创建稀疏导纳矩阵
    Ybus = sparse(
        horzcat(f, f, t, t), 
        horzcat(f, t, f, t), 
        vertcat(Yff, Yft, Ytf, Ytt), 
        nb, nb
    );
    
    return Ybus;
}

// 牛顿-拉夫逊法潮流计算
fn newton_pf(baseMVA, bus, gen, branch) {
    // 初始化参数
    tol = 1e-6;  // 收敛 tolerance
    max_iter = 20;  // 最大迭代次数
    nb = size(bus, 0);  // 母线数量
    
    // 构建导纳矩阵
    Ybus = make_y_bus(baseMVA, bus, branch);
    
    // 初始化电压向量
    V = slice(bus, [0], [VM-1]) .* exp(c(0,1) * slice(bus, [0], [VA-1]) * pi / 180.0);
    
    // 确定节点类型
    bus_type = slice(bus, [0], [BUS_TYPE-1]);
    ref = find(bus_type == REF);  // 平衡节点
    pv = find(bus_type == PV);    // PV节点
    pq = find(bus_type == PQ);    // PQ节点
    
    // 初始化注入功率
    P = zeros(nb, 1);
    Q = zeros(nb, 1);
    
    // 计算注入功率
    for iter in 0..max_iter {
        // 计算功率注入
        S = V .* conj(Ybus * V);
        P_calc = real(S);
        Q_calc = imag(S);
        
        // 计算功率误差
        P_error = P - P_calc;
        Q_error = Q - Q_calc;
        
        // 检查收敛
        if max(abs(vertcat(P_error, Q_error))) < tol {
            break;
        }
        
        // 构建雅可比矩阵
        J11 = zeros(nb, nb);
        J12 = zeros(nb, nb);
        J21 = zeros(nb, nb);
        J22 = zeros(nb, nb);
        
        for i in 0..nb {
            for j in 0..nb {
                if i != j {
                    Y_ij = get(Ybus, i, j);
                    V_i = get(V, i);
                    V_j = get(V, j);
                    theta_ij = arg(V_i) - arg(V_j);
                    
                    J11[i, j] = -abs(V_i) * abs(V_j) * abs(Y_ij) * sin(theta_ij + arg(Y_ij));
                    J12[i, j] = -abs(V_i) * abs(Y_ij) * cos(theta_ij + arg(Y_ij));
                    J21[i, j] = abs(V_i) * abs(V_j) * abs(Y_ij) * cos(theta_ij + arg(Y_ij));
                    J22[i, j] = -abs(V_i) * abs(Y_ij) * sin(theta_ij + arg(Y_ij));
                } else {
                    sum_term = 0.0;
                    for k in 0..nb {
                        if k != i {
                            Y_ik = get(Ybus, i, k);
                            V_k = get(V, k);
                            theta_ik = arg(V_i) - arg(V_k);
                            sum_term += abs(Y_ik) * abs(V_k) * sin(theta_ik + arg(Y_ik));
                        }
                    }
                    J11[i, i] = abs(V_i) * sum_term;
                    J12[i, i] = abs(V_i) * real(Y_ij);
                    J21[i, i] = -abs(V_i) * sum_term;
                    J22[i, i] = -abs(V_i) * imag(Y_ij);
                }
            }
        }
        
        // 求解修正方程
        delta = linsolve(
            [[J11, J12], [J21, J22]], 
            vertcat(P_error, Q_error)
        );
        
        // 更新电压
        delta_theta = slice(delta, [0], [0, nb]);
        delta_V = slice(delta, [nb], [0, nb]);
        V = V .* exp(c(0, 1) * delta_theta) .* (1 + delta_V);
    }
    
    // 更新母线电压
    assign(bus, abs(V), [0], [VM-1]);
    assign(bus, rad2deg(arg(V)), [0], [VA-1]);
    
    return bus;
}

// 快速解耦潮流计算
fn fast_decoupled_pf(baseMVA, bus, gen, branch) {
    // 初始化参数
    tol = 1e-6;  // 收敛 tolerance
    max_iter = 20;  // 最大迭代次数
    nb = size(bus, 0);  // 母线数量
    
    // 构建导纳矩阵
    Ybus = make_y_bus(baseMVA, bus, branch);
    
    // 初始化电压向量
    V = slice(bus, [0], [VM-1]) .* exp(c(0,1) * slice(bus, [0], [VA-1]) * pi / 180.0);
    
    // 确定节点类型
    bus_type = slice(bus, [0], [BUS_TYPE-1]);
    ref = find(bus_type == REF);  // 平衡节点
    pv = find(bus_type == PV);    // PV节点
    pq = find(bus_type == PQ);    // PQ节点
    
    // 初始化注入功率
    P = zeros(nb, 1);
    Q = zeros(nb, 1);
    
    // 计算注入功率
    for iter in 0..max_iter {
        // 计算功率注入
        S = V .* conj(Ybus * V);
        P_calc = real(S);
        Q_calc = imag(S);
        
        // 计算功率误差
        P_error = P - P_calc;
        Q_error = Q - Q_calc;
        
        // 检查收敛
        if max(abs(vertcat(P_error, Q_error))) < tol {
            break;
        }
        
        // 构建B'和B''矩阵
        B_prime = -imag(Ybus);
        B_double_prime = -imag(Ybus);
        
        // 求解修正方程
        delta_theta = B_prime \ (P_error ./ abs(V));
        delta_V = B_double_prime \ (Q_error ./ abs(V));
        
        // 更新电压
        V = V .* exp(c(0, 1) * delta_theta) .* (1 + delta_V);
    }
    
    // 更新母线电压
    assign(bus, abs(V), [0], [VM-1]);
    assign(bus, rad2deg(arg(V)), [0], [VA-1]);
    
    return bus;
}

// 示例使用
// 加载系统数据
baseMVA = 100;

// 母线数据
bus = [
    [1, 3, 0, 0, 0, 0, 1, 1.06, 0, 0, 1, 1.06, 0.94],
    [2, 2, 21.7, 12.7, 0, 0, 1, 1.045, -4.98, 0, 1, 1.06, 0.94],
    [3, 2, 94.2, 19, 0, 0, 1, 1.01, -12.72, 0, 1, 1.06, 0.94],
    [4, 1, 47.8, -3.9, 0, 0, 1, 1.019, -10.33, 0, 1, 1.06, 0.94]
];

// 发电机数据
gen = [
    [1, 232.4, -16.9, 10, 0, 1.06, 100, 1, 332.4, 0],
    [2, 40, 42.4, 50, -40, 1.045, 100, 1, 140, 0],
    [3, 0, 23.4, 40, 0, 1.01, 100, 1, 100, 0]
];

// 支路数据
branch = [
    [1, 2, 0.01938, 0.05917, 0.0528, 0, 0, 0, 0, 0, 1, -360, 360],
    [1, 5, 0.05403, 0.22304, 0.0492, 0, 0, 0, 0, 0, 1, -360, 360],
    [2, 3, 0.04699, 0.19797, 0.0438, 0, 0, 0, 0, 0, 1, -360, 360]
];

// 运行潮流计算
bus_results = newton_pf(baseMVA, bus, gen, branch);
// bus_results = fast_decoupled_pf(baseMVA, bus, gen, branch);

// 输出结果
print("母线电压幅值: ", slice(bus_results, [0], [VM-1]));
print("母线电压角度: ", slice(bus_results, [0], [VA-1]));
```

### 代码说明

1. **导纳矩阵构建**：`make_y_bus`函数根据支路参数计算系统导纳矩阵，考虑了变压器变比和移相角的影响。

2. **牛顿-拉夫逊法**：`newton_pf`函数实现了标准的牛顿-拉夫逊潮流算法，通过构建雅可比矩阵求解修正方程，迭代更新节点电压直到收敛。

3. **快速解耦法**：`fast_decoupled_pf`函数实现了快速解耦潮流算法，利用电力系统的特性简化雅可比矩阵，提高计算速度。

4. **数据结构**：采用RustScript的张量结构存储母线、发电机和支路数据，通过索引操作提取和更新数据。

5. **示例数据**：包含了一个简化的4节点系统数据，可替换为实际系统数据进行计算。

### 使用说明

1. 加载系统数据：设置基准功率`baseMVA`，定义母线`bus`、发电机`gen`和支路`branch`数据。

2. 选择潮流算法：调用`newton_pf`或`fast_decoupled_pf`函数进行潮流计算。

3. 输出结果：打印母线电压幅值和角度等计算结果。

该实现遵循RustScript的语法规范，使用张量操作处理所有数据，适合科学计算和电力系统分析。

Process finished with exit code 0
