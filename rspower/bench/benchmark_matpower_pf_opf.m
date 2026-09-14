function benchmark_matpower_pf_opf()
%BENCHMARK_MATPOWER_PF_OPF  生成 PF/OPF 总耗时的 MATPOWER 基准数据。
%   使用 MATPOWER 默认 runpf/runopf 配置；每个算例预热 5 次，
%   随后测量 21 次并输出中位数。预热不计入耗时。
%   输出 matpower_total CSV 记录，用于 pf_performance_tensoreval_vs_matpower.csv。
%   运行前需将 MATPOWER 及本目录加入 MATLAB path。
    cases = {'case5', 'case9', 'case14', 'case30', 'case39', 'case57', ...
        'case118', 'case118zh', 'case300', 'case1354pegase', ...
        'case2736sp', 'case2737sop', 'case13659pegase'};
    pfopt = mpoption('verbose', 0, 'out.all', 0);
    opfopt = mpoption('verbose', 0, 'out.all', 0);

    for case_index = 1:length(cases)
        case_name = cases{case_index};
        original = loadcase(case_name);
        pf_ms = benchmark_solver(@() runpf(original, pfopt), 5, 21);
        fprintf('matpower_total,%s,pf,%.6f\n', case_name, pf_ms);

        try
            [opf_ms, converged] = benchmark_solver(@() runopf(original, opfopt), 5, 21);
            if converged
                fprintf('matpower_total,%s,runopf,%.6f\n', case_name, opf_ms);
            else
                fprintf('matpower_total,%s,runopf,not_converged\n', case_name);
            end
        catch exception
            fprintf('matpower_total,%s,runopf,unavailable:%s\n', ...
                case_name, strrep(exception.message, newline, ' '));
        end
    end
end

function [result_ms, converged] = benchmark_solver(operation, warmups, samples)
    converged = true;
    for index = 1:warmups
        result = operation();
        if isfield(result, 'success') && ~result.success
            converged = false;
            result_ms = NaN;
            return;
        end
    end
    elapsed = zeros(samples, 1);
    for sample = 1:samples
        started = tic;
        result = operation();
        elapsed(sample) = toc(started) * 1e3;
        if isfield(result, 'success') && ~result.success
            converged = false;
            result_ms = NaN;
            return;
        end
    end
    result_ms = median(elapsed);
end
