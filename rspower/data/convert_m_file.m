function convert_m_file(input_file, output_file)
    % input_file: ����.m�ļ�·�������� 'case14.m'��
    % output_file�����.txt�ļ�·�������� 'case14_mems.txt'��
    % ��matpower�����ʽת��Ϊmems֧�ֵĽű������ʽ
    % ��ʶ��mpc��bus��gen��branch��gencost���󣨵�21�д����ֶ���ɾ��

    % ��ȡ�����ļ�
    run(input_file); 
    mpc = ans;
    
    % ������ļ�
    fid = fopen(output_file, 'w');
    if fid == -1
        error('�޷����ļ�');
    end

    % д��baseMVA
    fprintf(fid, 'baseMVA = %d;\n\n', mpc.baseMVA);
    
    % �̶���ȡ�ľ������Ʋ�������ת��
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
    disp(['�ļ���ת��������Ϊ: ', output_file]);
end

function formatted_matrix = format_matrix(input_matrix)
    % �������input_matrix�������ʽ�����formatted_matrix

    [num_rows, ~] = size(input_matrix);
    formatted_matrix = [];

    for i = 1:num_rows
        row = input_matrix(i, :);
        
        % �����е�Ԫ��ת��Ϊ���ŷָ����ַ���
        formatted_row = strjoin(arrayfun(@(x) num2str(x), row, 'UniformOutput', false), ',');

        if i ~= num_rows
            formatted_matrix = [formatted_matrix, sprintf('[%s],\n', formatted_row)];
        else
            formatted_matrix = [formatted_matrix, sprintf('[%s]\n', formatted_row)];
        end
    end
end