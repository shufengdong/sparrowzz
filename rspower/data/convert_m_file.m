function convert_m_file(input_file, output_file)
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
        error('无法打开文件');
    end

    % 写入baseMVA
    fprintf(fid, 'baseMVA = %d;\n\n', mpc.baseMVA);
    
    % 固定读取的矩阵名称并遍历、转换
    matrix_names = {'bus', 'gen', 'branch', 'gencost'};
    for i = 1:length(matrix_names)
        matrix_name = matrix_names{i};

        if isfield(mpc, matrix_name)
            matrix_data = mpc.(matrix_name);
            
            if ~isempty(matrix_data)
                formatted_matrix = format_matrix(matrix_data);
                fprintf(fid, '%s = [\n%s];\n\n', matrix_name, formatted_matrix);
            end
        end
    end

    fclose(fid);
    disp(['文件已转换并保存为: ', output_file]);
end

function formatted_matrix = format_matrix(input_matrix)
    % 输入矩阵input_matrix，输出格式化后的formatted_matrix

    [num_rows, ~] = size(input_matrix);
    formatted_matrix = [];

    for i = 1:num_rows
        row = input_matrix(i, :);
        
        % 将该行的元素转换为逗号分隔的字符串
        formatted_row = strjoin(arrayfun(@(x) num2str(x), row, 'UniformOutput', false), ',');

        if i ~= num_rows
            formatted_matrix = [formatted_matrix, sprintf('[%s],\n', formatted_row)];
        else
            formatted_matrix = [formatted_matrix, sprintf('[%s]\n', formatted_row)];
        end
    end
end