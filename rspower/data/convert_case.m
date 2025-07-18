% 批量转换case*.m文件内容
files = dir('case*.m');
for k = 1:length(files)
    mfile = files(k).name;
    [~, name, ~] = fileparts(mfile);
    txtfile = [name, '.txt'];
    
    fin = fopen(mfile, 'r');
    fout = fopen(txtfile, 'w');
    
    while ~feof(fin)
        line = fgetl(fin);
        if ~ischar(line), break; end
        
        % 注释行：% -> //
        if contains(line, '%')
            line = regexprep(line, '%+', '//');
            fprintf(fout, '%s\n', line);
            continue;
        end
        
        % 变量名行：mpc.bus = [  -> bus = [
        var_match = regexp(line, '^\s*mpc\.(\w+)\s*=\s*\[', 'tokens');
        if ~isempty(var_match)
            varname = var_match{1}{1};
            fprintf(fout, '%s = [\n', varname);
            % 进入矩阵模式
            while true
                matline = fgetl(fin);
                if ~ischar(matline), break; end
                % 跳过空行
                if isempty(strtrim(matline)), continue; end
                % 检查是否矩阵结束
                if contains(matline, '];')
                    fprintf(fout, '];\n\n');
                    break;
                end
                % 去掉行尾分号
                matline = regexprep(matline, ';', '');
                % 去掉行首和行尾的方括号
                matline = regexprep(matline, '^\s*\[|\]\s*$', '');
                % 多个空格或tab转为单个空格
                matline = regexprep(matline, '[\s,]+', ' ');
                % 去掉首尾空格
                matline = strtrim(matline);
                if isempty(matline), continue; end
                % 用逗号分隔
                nums = strsplit(matline, ' ');
                matline_new = ['[', strjoin(nums, ', '), '],'];
                fprintf(fout, '\t%s\n', matline_new);
            end
            continue;
        end
        
        % 其他行直接跳过（如baseMVA等可按需保留）
    end
    
    fclose(fin);
    fclose(fout);
    fprintf('已转换: %s -> %s\n', mfile, txtfile);
end