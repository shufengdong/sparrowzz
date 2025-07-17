function results = execute_and_parse()
    % 执行命令行并解析复数矩阵结果
    % 返回一个结构体，包含所有测试文件的解析结果

    % 设置路径和文件
    tensoreval_path = '..\..\..\eig-rc\target\release\examples\tensoreval.exe';
    test_files = {
        'test_make_jac.txt'
        'test_make_sbus.txt'
        'test_make_ybus.txt'
        'test_runpf.txt'
    };

    % 初始化结果结构体
    results = struct();

    fprintf('开始执行并解析测试文件...\n');

    % 逐一执行和解析
    for i = 1:length(test_files)
        test_file = test_files{i};
        fprintf('正在处理: %s\n', test_file);

        % 构建并执行命令
        cmd = sprintf('%s --complex %s', tensoreval_path, test_file);
        [status, output] = system(cmd);

        % 解析结果
        if status == 0 && ~isempty(strtrim(output))
            try
                parsed_matrix = parse_complex_matrix_from_output(output);
                results.(sprintf('test_%d_%s', i, strrep(test_file, '.txt', ''))) = parsed_matrix;
                fprintf('  成功解析矩阵: %dx%d\n', size(parsed_matrix, 1), size(parsed_matrix, 2));
            catch ME
                fprintf('  解析失败: %s\n', ME.message);
                results.(sprintf('test_%d_%s', i, strrep(test_file, '.txt', ''))) = [];
            end
        else
            fprintf('  无输出或执行失败\n');
            results.(sprintf('test_%d_%s', i, strrep(test_file, '.txt', ''))) = [];
        end
    end

    fprintf('\n解析完成!\n');

    % 显示结果概要
    fieldNames = fieldnames(results);
    for i = 1:length(fieldNames)
        matrix = results.(fieldNames{i});
        if ~isempty(matrix)
            fprintf('%s: %dx%d 复数矩阵\n', fieldNames{i}, size(matrix, 1), size(matrix, 2));
        else
            fprintf('%s: 空矩阵\n', fieldNames{i});
        end
    end
end

function matrix = parse_complex_matrix_from_output(output)
    % 从命令输出中解析复数矩阵，自动检测矩阵大小

    % 使用正则表达式匹配所有复数对
    pattern = 'Complex\s*{\s*re:\s*([-\d.eE]+),\s*im:\s*([-\d.eE]+),?\s*}';
    [matches] = regexp(output, pattern, 'tokens');

    if isempty(matches)
        error('未找到复数数据');
    end

    % 将所有匹配转换为复数
    complexNums = zeros(1, length(matches));
    for i = 1:length(matches)
        re = str2double(matches{i}{1});
        im = str2double(matches{i}{2});
        complexNums(i) = complex(re, im);
    end

    % 从输出中提取矩阵维度信息
    [rows, cols] = extract_matrix_dimensions(output);

    if rows * cols ~= length(complexNums)
        if rows * cols > length(complexNums)
            error('  矩阵维度大于元素数量: %dx%d > %d', ...
                rows, cols, length(complexNums));
        else
            fprintf('  矩阵维度小于元素数量: %dx%d < %d，取前%dx%d个元素', ...
            rows, cols, length(complexNums),rows, cols);
        end
    end

    % 重塑为指定维度的矩阵
    % 注意：根据输出格式调整reshape的维度顺序
    matrix = reshape(complexNums(1:cols*rows), [cols, rows]).';
end

function [rows, cols] = extract_matrix_dimensions(output)
    % 从输出中提取矩阵维度信息

    % 优先尝试匹配 shape=[rows, cols] 模式
    shape_pattern = 'shape=\[(\d+),\s*(\d+)\]';
    shape_matches = regexp(output, shape_pattern, 'tokens');

    if ~isempty(shape_matches)
        rows = str2double(shape_matches{1}{1});
        cols = str2double(shape_matches{1}{2});
        fprintf('  从shape信息中提取矩阵大小: %dx%d\n', rows, cols);
        return;
    end

    % 如果没有找到shape信息，尝试通过解析矩阵结构推断
    % 查找矩阵行的模式 [Complex{...}, Complex{...}, ...]
    row_pattern = '\[Complex[^\]]+\]';
    row_matches = regexp(output, row_pattern, 'match');

    if ~isempty(row_matches)
        rows = length(row_matches);
        % 从第一行计算列数
        first_row = row_matches{1};
        col_pattern = 'Complex\s*{\s*re:\s*[-\d.eE]+,\s*im:\s*[-\d.eE]+\s*}';
        col_matches = regexp(first_row, col_pattern, 'match');
        cols = length(col_matches);
        fprintf('  从矩阵结构中推断大小: %dx%d\n', rows, cols);
        return;
    end

    % 如果无法推断，尝试假设为方阵
    total_elements = length(regexp(output, 'Complex\s*{\s*re:\s*[-\d.eE]+,\s*im:\s*[-\d.eE]+\s*}', 'match'));
    sqrt_total = sqrt(total_elements);

    if abs(sqrt_total - round(sqrt_total)) < 1e-10
        rows = round(sqrt_total);
        cols = round(sqrt_total);
        fprintf('  推断为方阵: %dx%d\n', rows, cols);
        return;
    end

    error('无法确定矩阵维度，总元素数: %d', total_elements);
end

function matrix = parse_complex_matrix_from_file(filename)
    % 从文件中解析复数矩阵（兼容原有功能）

    % 读取文件内容
    fileContent = fileread(filename);

    % 使用解析函数
    matrix = parse_complex_matrix_from_output(fileContent);
end

% 使用示例函数
function demo_usage()
    % 演示如何使用这个函数

    fprintf('执行示例...\n');

    % 执行并解析所有测试
    results = execute_and_parse();

    % 访问特定结果
    fieldNames = fieldnames(results);
    for i = 1:length(fieldNames)
        matrix = results.(fieldNames{i});
        if ~isempty(matrix)
            fprintf('\n%s 矩阵统计:\n', fieldNames{i});
            fprintf('  大小: %dx%d\n', size(matrix, 1), size(matrix, 2));
            fprintf('  最大实部: %.6f\n', max(max(real(matrix))));
            fprintf('  最大虚部: %.6f\n', max(max(imag(matrix))));
            fprintf('  矩阵范数: %.6f\n', norm(matrix, 'fro'));
        end
    end
end

% 使用说明:
% 1. 在 MATLAB 中切换到 rspower/examples 目录
% 2. 运行: results = execute_and_parse()
% 3. 或者运行演示: demo_usage()
% 4. 结果存储在返回的结构体中，每个字段对应一个测试文件的矩阵
