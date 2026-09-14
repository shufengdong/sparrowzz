function benchmark_matpower_pf_breakdown()
%BENCHMARK_MATPOWER_PF_BREAKDOWN  生成 MATPOWER 牛顿潮流的分阶段耗时。
%   对 case300、case1354pegase 和 case13659pegase 强制使用 LU3，
%   分别计时导数、索引、Jacobian 拼接、左除和 pfsoln 等阶段。
%   每个算例预热 5 次，测量 21 次并输出中位数，
%   数据用于 pf_stage_performance_tensoreval_vs_matpower.csv。
    cases = {'case300', 'case1354pegase', 'case13659pegase'};
    mpopt = mpoption('verbose', 0, 'out.all', 0, 'pf.nr.lin_solver', 'LU3');
    labels = {'total', 'prepare', 'make_ybus', 'make_sbus', 'initial_mismatch', ...
        'derivative', 'select', 'blockcat', 'left_div', 'voltage_update', ...
        'iteration_mismatch', 'pfsoln', 'int2ext', 'iterations'};

    for case_index = 1:length(cases)
        case_name = cases{case_index};
        original = loadcase(case_name);
        for warmup = 1:5
            profile_one(original, mpopt);
        end
        samples = zeros(21, length(labels));
        for sample = 1:size(samples, 1)
            samples(sample, :) = profile_one(original, mpopt);
        end
        values = median(samples, 1);
        for index = 1:length(labels)
            fprintf('matpower_stage,%s,%s,%.6f\n', case_name, labels{index}, values(index));
        end
    end
end

function timing = profile_one(original, mpopt)
    define_constants;
    total_start = tic;
    stage_start = tic;
    mpc = original;
    if size(mpc.branch, 2) < QT
        mpc.branch = [mpc.branch, zeros(size(mpc.branch, 1), QT - size(mpc.branch, 2))];
    end
    mpc = ext2int(mpc, mpopt);
    [baseMVA, bus, gen, branch] = deal(mpc.baseMVA, mpc.bus, mpc.gen, mpc.branch);
    [ref, pv, pq] = bustypes(bus, gen);
    on = find(gen(:, GEN_STATUS) > 0);
    gbus = gen(on, GEN_BUS);
    V = bus(:, VM) .* exp(1j * pi / 180 * bus(:, VA));
    vcb = ones(size(V));
    vcb(pq) = 0;
    k = find(vcb(gbus));
    V(gbus(k)) = gen(on(k), VG) ./ abs(V(gbus(k))) .* V(gbus(k));
    prepare_ms = toc(stage_start) * 1e3;

    stage_start = tic;
    [Ybus, Yf, Yt] = makeYbus(baseMVA, bus, branch);
    make_ybus_ms = toc(stage_start) * 1e3;
    Sbus = @(Vm) makeSbus(baseMVA, bus, gen, mpopt, Vm);
    stage_start = tic;
    initial_sbus = Sbus(abs(V));
    make_sbus_ms = toc(stage_start) * 1e3;

    Va = angle(V);
    Vm = abs(V);
    npv = length(pv);
    npq = length(pq);
    j1 = 1;         j2 = npv;
    j3 = j2 + 1;    j4 = j2 + npq;
    j5 = j4 + 1;    j6 = j4 + npq;
    pv_pq = [pv; pq];
    stage_start = tic;
    mis = V .* conj(Ybus * V) - initial_sbus;
    F = [real(mis(pv_pq)); imag(mis(pq))];
    normF = norm(F, inf);
    initial_mismatch_ms = toc(stage_start) * 1e3;

    derivative_ms = 0;
    select_ms = 0;
    blockcat_ms = 0;
    solve_ms = 0;
    update_ms = 0;
    mismatch_ms = 0;
    converged = normF < mpopt.pf.tol;
    iteration = 0;
    while ~converged && iteration < mpopt.pf.nr.max_it
        iteration = iteration + 1;
        stage_start = tic;
        [dSbus_dVa, dSbus_dVm] = dSbus_dV(Ybus, V);
        [~, neg_dSd_dVm] = Sbus(Vm);
        dSbus_dVm = dSbus_dVm - neg_dSd_dVm;
        derivative_ms = derivative_ms + toc(stage_start) * 1e3;

        stage_start = tic;
        j11 = real(dSbus_dVa(pv_pq, pv_pq));
        j12 = real(dSbus_dVm(pv_pq, pq));
        j21 = imag(dSbus_dVa(pq, pv_pq));
        j22 = imag(dSbus_dVm(pq, pq));
        select_ms = select_ms + toc(stage_start) * 1e3;
        stage_start = tic;
        J = [j11, j12; j21, j22];
        blockcat_ms = blockcat_ms + toc(stage_start) * 1e3;
        stage_start = tic;
        dx = mplinsolve(J, -F, 'LU3');
        solve_ms = solve_ms + toc(stage_start) * 1e3;

        stage_start = tic;
        if npv
            Va(pv) = Va(pv) + dx(j1:j2);
        end
        if npq
            Va(pq) = Va(pq) + dx(j3:j4);
            Vm(pq) = Vm(pq) + dx(j5:j6);
        end
        V = Vm .* exp(1j * Va);
        Vm = abs(V);
        Va = angle(V);
        update_ms = update_ms + toc(stage_start) * 1e3;

        stage_start = tic;
        mis = V .* conj(Ybus * V) - Sbus(Vm);
        F = [real(mis(pv_pq)); imag(mis(pq))];
        normF = norm(F, inf);
        converged = normF < mpopt.pf.tol;
        mismatch_ms = mismatch_ms + toc(stage_start) * 1e3;
    end

    stage_start = tic;
    [bus, gen, branch] = pfsoln(baseMVA, bus, gen, branch, ...
        Ybus, Yf, Yt, V, ref, pv, pq, mpopt);
    pfsoln_ms = toc(stage_start) * 1e3;
    stage_start = tic;
    mpc.bus = bus;
    mpc.gen = gen;
    mpc.branch = branch;
    int2ext(mpc);
    int2ext_ms = toc(stage_start) * 1e3;
    total_ms = toc(total_start) * 1e3;

    timing = [total_ms, prepare_ms, make_ybus_ms, make_sbus_ms, initial_mismatch_ms, ...
        derivative_ms, select_ms, blockcat_ms, solve_ms, update_ms, mismatch_ms, ...
        pfsoln_ms, int2ext_ms, iteration];
end
