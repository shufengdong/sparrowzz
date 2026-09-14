function benchmark_matpower_pf_solve_phases()
%BENCHMARK_MATPOWER_PF_SOLVE_PHASES  比较 PF Jacobian 的稀疏线性求解方式。
%   构造已收敛潮流点的 Jacobian，分别测量 MATPOWER LU3、
%   AMD+LU 分解、三角回代和 MATLAB 默认左除。
%   每项先预热 10 轮，再取 31 个批次平均耗时的中位数，
%   数据用于 pf_linear_solver_performance_mumps_vs_matpower.csv。
    cases = {'case300', 'case1354pegase', 'case13659pegase'};
    mpopt = mpoption('verbose', 0, 'out.all', 0);

    for case_index = 1:length(cases)
        case_name = cases{case_index};
        solved = runpf(loadcase(case_name), mpopt);
        mpc = ext2int(solved);
        [~, pv, pq] = bustypes(mpc.bus, mpc.gen);
        [Ybus, ~, ~] = makeYbus(mpc.baseMVA, mpc.bus, mpc.branch);
        V = mpc.bus(:, 8) .* exp(1j * pi / 180 * mpc.bus(:, 9));
        [dSbus_dVa, dSbus_dVm] = dSbus_dV(Ybus, V);
        pv_pq = [pv; pq];
        J = [real(dSbus_dVa(pv_pq, pv_pq)), real(dSbus_dVm(pv_pq, pq)); ...
             imag(dSbus_dVa(pq, pv_pq)),       imag(dSbus_dVm(pq, pq))];
        rhs = ones(size(J, 1), 1);

        complete_ms = median_time_ms(@() mplinsolve(J, rhs, 'LU3'), 5, 31);
        backslash_ms = median_time_ms(@() J \ rhs, 5, 31);
        factor_ms = median_time_ms(@() lu3_factor(J), 5, 31);
        [L, U, p, q] = lu3_factor(J);
        triangular_ms = median_time_ms(@() lu3_triangular(L, U, p, q, rhs), 10, 31);

        fprintf('matpower_solve,%s,jacobian_rows,%d\n', case_name, size(J, 1));
        fprintf('matpower_solve,%s,jacobian_nnz,%d\n', case_name, nnz(J));
        fprintf('matpower_solve,%s,LU3_complete,%.6f\n', case_name, complete_ms);
        fprintf('matpower_solve,%s,LU3_factor_with_amd,%.6f\n', case_name, factor_ms);
        fprintf('matpower_solve,%s,LU3_triangular,%.6f\n', case_name, triangular_ms);
        fprintf('matpower_solve,%s,backslash,%.6f\n', case_name, backslash_ms);
    end
end

function [L, U, p, q] = lu3_factor(A)
    q = amd(A);
    [L, U, p] = lu(A(q, q), 1.0, 'vector');
end

function x = lu3_triangular(L, U, p, q, b)
    x = zeros(size(b));
    x(q) = U \ (L \ b(q(p)));
end

function result = median_time_ms(operation, iterations, samples)
    for index = 1:10
        operation();
    end
    elapsed = zeros(samples, 1);
    for sample = 1:samples
        tic;
        for iteration = 1:iterations
            operation();
        end
        elapsed(sample) = toc * 1e3 / iterations;
    end
    result = median(elapsed);
end
