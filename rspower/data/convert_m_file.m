function convert_m_file3(input_file, output_file)
    % input_file: 输入.m文件路径（例如 'case14.m'）
    % output_file：输出.txt文件路径（例如 'case14_mems.txt'）
    % 将matpower矩阵格式转换为mems支持的脚本矩阵格式
    % 可识别mpc的bus、gen、branch、gencost矩阵（第21行处可手动增删）

    % 读取输入文件
    run(input_file);
    mpc = ans;

    % 打开输出文件
    fid = fopen(output_file, 'w');
    if fid == -1
        error('无法打开输出文件');
    end

    % 写入baseMVA
    fprintf(fid, 'baseMVA = %d;\n\n', mpc.baseMVA);

    % 固定矩阵名称并遍历、转换
    matrix_names = {'bus', 'gen', 'branch', 'gencost'};
    for i = 1:length(matrix_names)
        name = matrix_names{i};
        if isfield(mpc, name)
            data = mpc.(name);
            if ~isempty(data)
                formatted = format_matrix_aligned(data);
                fprintf(fid, '%s = [\n%s];\n\n', name, formatted);
            end
        end
    end

    fclose(fid);
    disp(['文件已转换并保存为: ', output_file]);
end

function formatted_matrix = format_matrix_aligned(input_matrix)
    % 输出带右对齐的格式化矩阵字符串，从第二行开始每行缩进4个空格
    [num_rows, num_cols] = size(input_matrix);

    % 计算每列的最大宽度
    col_widths = zeros(1, num_cols);
    for j = 1:num_cols
        col_strs = arrayfun(@(x) num2str(x), input_matrix(:, j), 'UniformOutput', false);
        col_widths(j) = max(cellfun(@length, col_strs));
    end

    % 构造格式化矩阵行
    formatted_matrix = '';
    for i = 1:num_rows
        row = input_matrix(i, :);

        % 对每列使用最大宽度进行右对齐，格式化每个元素
        formatted_row = arrayfun(@(x, w) sprintf(['%' num2str(w) 's'], num2str(x)), row, col_widths, 'UniformOutput', false);
        joined_row = strjoin(formatted_row, ', ');

        if i ~= num_rows
            formatted_matrix = [formatted_matrix, sprintf('    [%s],\n', joined_row)];  % 从第二行起缩进
        else
            formatted_matrix = [formatted_matrix, sprintf('    [%s]\n', joined_row)];  % 最后一行没有逗号
        end
    end
end
