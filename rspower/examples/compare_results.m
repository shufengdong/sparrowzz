function compare_results(case_name)
    % 比较 execute_and_parse 结果与 matpower 计算结果
    % case_name: 算例名称，如 'case14', 'case30', 'case57' 等

    if nargin < 1
        case_name = 'case14';  % 默认使用 case14
    end

    fprintf('==============开始为算例 %s 执行测试...==============\n', case_name);

    % 更新测试文件以使用指定算例
    update_test_files(case_name);

    fprintf('\n==============开始执行并解析测试文件...==============\n');

    % 执行并解析所有测试文件
    parsed_results = execute_and_parse();

    fprintf('\n==============开始与 matpower 结果比较...==============\n');

    % 比较的field_name和使用matpower计算函数
    field_names = {
        'ybus'
        'jac'
        'sdzip'
        'sbus'
        'runpf'
    };
    cal_makers = {
        @cal_makeybus
        @cal_makejac
        @cal_makesdzip
        @cal_makesbus
        @cal_runpf
    };

    % 依次比较每个矩阵
    for i = 1:length(field_names)
        field_name = field_names{i};
        cal_maker = cal_makers{i};

        fprintf('\n=== 比较 %s 矩阵 ===\n', field_name);

        % 获取解析的矩阵
        tensor_field = find_field_by_name(parsed_results, field_name);
        if isempty(tensor_field)
            fprintf('未找到 %s 矩阵结果\n', field_name);
            continue;
        end

        tensor_r = parsed_results.(tensor_field);
        if isempty(tensor_r)
            fprintf('%s 矩阵为空\n', field_name);
            continue;
        end

        % 计算 matpower 的矩阵
        matpower_r = cal_maker(case_name);

        % 比较矩阵
        compare_matrices(tensor_r, matpower_r, field_name);
    end

    fprintf('\n所有比较完成!\n\n');
end

function ybus = cal_makeybus(case_name)
    % 计算 Ybus 矩阵
    mpc = loadcase(case_name);
    ybus = full(makeYbus(mpc));
end

function jac = cal_makejac(case_name)
    % 计算 Jacobian 矩阵
    mpc = loadcase(case_name);
    jac = full(makeJac(mpc, 1));
end

function sdzip = cal_makesdzip(case_name)
    mpc = loadcase(case_name);
    sd = makeSdzip(mpc.baseMVA, mpc.bus);
    sdzip = [sd.z, sd.i, sd.p];
end

function sbus = cal_makesbus(case_name)
    mpc = loadcase(case_name);
    baseMVA = mpc.baseMVA;
    bus = mpc.bus;
    gen = mpc.gen;
    sbus = makeSbus(baseMVA, bus, gen);
end

function pfv = cal_runpf(case_name)
    mpc = loadcase(case_name);
    bus = runpf(mpc, mpoption('OUT_ALL',0','VERBOSE',0)).bus;
    pfv = bus(:, 8) .* exp(1j * pi/180 * bus(:, 9));
end


%% utils 一些工具函数
function field_name = find_field_by_name(results, target_name)
    % 在结果结构体中查找包含特定名称的字段

    field_names = fieldnames(results);
    field_name = '';

    for i = 1:length(field_names)
        if contains(lower(field_names{i}), lower(target_name))
            field_name = field_names{i};
            break;
        end
    end
end

function compare_matrices(matrix1, matrix2, matrix_name)
    % 比较两个矩阵的详细函数

    fprintf('比较 %s 矩阵:\n', matrix_name);
    fprintf('  张量结果大小: %dx%d\n', size(matrix1, 1), size(matrix1, 2));
    fprintf('  Matpower结果大小: %dx%d\n', size(matrix2, 1), size(matrix2, 2));

    % 检查矩阵大小是否一致
    if ~isequal(size(matrix1), size(matrix2))
        fprintf('  ❌ 矩阵大小不一致!\n');
        return;
    end

    % 计算误差
    error_matrix = matrix1 - matrix2;
    max_error = max(max(abs(error_matrix)));
    mean_error = mean(mean(abs(error_matrix)));
    relative_error = max_error / max(max(abs(matrix2)));

    % 显示比较结果
    fprintf('  最大绝对误差: %.2e\n', max_error);
    fprintf('  平均绝对误差: %.2e\n', mean_error);
    fprintf('  最大相对误差: %.2e\n', relative_error);

    % 判断是否相等（使用容差）
    tolerance = 1e-10;
    if max_error < tolerance
        fprintf('  ✅ 矩阵相等 (容差: %.0e)\n', tolerance);
    else
        fprintf('  ❌ 矩阵不相等 (容差: %.0e)\n', tolerance);

        % 显示最大误差的位置
        [max_row, max_col] = find(abs(error_matrix) == max_error, 1);
        fprintf('  最大误差位置: (%d, %d)\n', max_row, max_col);
        fprintf('  张量值: %.6f + %.6fi\n', real(matrix1(max_row, max_col)), imag(matrix1(max_row, max_col)));
        fprintf('  Matpower值: %.6f + %.6fi\n', real(matrix2(max_row, max_col)), imag(matrix2(max_row, max_col)));
    end

    % 可选：显示误差矩阵的统计信息
    if max_error >= tolerance
        fprintf('  误差矩阵统计:\n');
        fprintf('    实部误差范围: [%.2e, %.2e]\n', min(min(real(error_matrix))), max(max(real(error_matrix))));
        fprintf('    虚部误差范围: [%.2e, %.2e]\n', min(min(imag(error_matrix))), max(max(imag(error_matrix))));
    end
end

function update_test_files(case_name)
    % 更新所有测试文件以使用指定的算例
    % case_name: 算例名称，如 'case14', 'case30', 'case57' 等

    if nargin < 1
        case_name = 'case14';
    end

    % 定义数据文件路径
    data_file = sprintf('../data/%s.txt', case_name);

    % 定义测试文件列表
    test_files = {
        'test_make_jac.txt'
        'test_make_sbus.txt'
        'test_make_ybus.txt'
        'test_runpf.txt'
    };

    fprintf('正在更新测试文件以使用算例: %s\n', case_name);
    fprintf('  数据文件: %s\n', data_file);

    % 更新每个测试文件
    for i = 1:length(test_files)
        update_single_test_file(test_files{i}, data_file);
    end

    fprintf('所有测试文件已更新完成!\n');
end

function update_single_test_file(filename, data_file)
    % 更新单个测试文件的算例引用

    fprintf('  更新文件: %s\n', filename);

    % 检查文件是否存在
    if ~exist(filename, 'file')
        fprintf('    警告: 文件 %s 不存在，跳过更新\n', filename);
        return;
    end

    % 读取现有文件内容
    fid = fopen(filename, 'r', 'n', 'UTF-8');
    if fid == -1
        error('无法读取文件: %s', filename);
    end
    content = fread(fid, '*char')';
    fclose(fid);
    lines = strsplit(content, '\n', 'CollapseDelimiters', false);

    % 判断是否需要更新第一行
    need_update = true;

    if ~isempty(lines) && contains(lines{1}, '#include')
        % 提取当前include的文件路径
        current_include = strtrim(extractAfter(lines{1}, '#include'));

        % 判断是否是data目录下的算例文件
        if contains(current_include, '../data/case') && contains(current_include, '.txt')
            % 是算例文件，检查是否与目标算例相同
            if strcmp(current_include, data_file)
                fprintf('    已经是目标算例，无需修改\n');
                need_update = false;
            else
                fprintf('    当前算例: %s，将更新为: %s\n', current_include, data_file);
            end
        else
            % 不是算例文件，需要在第一行插入算例include
            fprintf('    第一行include不是算例文件，将插入算例include\n');
            lines = [{sprintf('#include %s', data_file)}, lines];
            need_update = true;
        end
    else
        % 第一行不是include语句，插入算例include
        fprintf('    文件第一行不是include语句，将插入算例include\n');
        lines = [{sprintf('#include %s', data_file)}, lines];
    end

    % 如果需要更新，则写回文件
    if need_update
        % 更新第一行的 #include 语句(如果是已有的算例include)
        if ~isempty(lines) && contains(lines{1}, '#include') && contains(lines{1}, '../data/case') && ~strcmp(lines{1}, sprintf('#include %s', data_file))
            lines{1} = sprintf('#include %s', data_file);
        end

        % 写回文件
        fid = fopen(filename, 'w', 'n', 'UTF-8');
        if fid == -1
            error('无法写入文件: %s', filename);
        end

        for i = 1:length(lines)
            fprintf(fid, '%s', lines{i});
            if i < length(lines)
                fprintf(fid, '\n');
            end
        end
        fclose(fid);

        fprintf('    文件已更新\n');
    end
end

% 使用说明:
% 1. 在 MATLAB 中切换到 rspower/examples 目录
% 2. 运行: compare_results()
% 3. 程序会自动执行测试文件、解析结果，并与 matpower 计算结果比较
% 4. 可选择调用 visualize_comparison() 进行可视化比较
