function benchmark_matpower_remaining_breakdown()
%BENCHMARK_MATPOWER_REMAINING_BREAKDOWN  诊断 makeSbus、pfsoln 和 int2ext 的子阶段。
%   该脚本用于定位索引、稀疏乘法、无功分配和结果写回开销，
%   不生成独立归档 CSV；关键结论记录在 PF_PERFORMANCE.md。
%   每个算例预热 5 次，测量 21 次并输出中位数。
    cases = {'case300', 'case1354pegase', 'case13659pegase'};
    mpopt = mpoption('verbose', 0, 'out.all', 0, 'pf.nr.lin_solver', 'LU3');
    labels = {'make_sbus_load_model', 'make_sbus_online_index', ...
        'make_sbus_connection_matrix', 'make_sbus_generator_values', ...
        'make_sbus_sparse_matvec', 'make_sbus_combine', ...
        'pfsoln_initialize', 'pfsoln_bus_voltage', 'pfsoln_generator_index', ...
        'pfsoln_generator_indices', 'pfsoln_generator_y_select', ...
        'pfsoln_generator_spmv', 'pfsoln_generator_combine', ...
        'pfsoln_initial_q', 'pfsoln_q_distribution', ...
        'pfsoln_slack_generation', 'pfsoln_branch_flow', ...
        'pfsoln_branch_indices', 'pfsoln_branch_y_select', ...
        'pfsoln_branch_spmv', 'pfsoln_branch_combine', ...
        'pfsoln_branch_write', ...
        'int2ext_bus', 'int2ext_gen', 'int2ext_branch'};

    for case_index = 1:length(cases)
        case_name = cases{case_index};
        original = loadcase(case_name);
        internal = ext2int(original, mpopt);
        i2e = internal.order.bus.i2e;
        solved = runpf(original, mpopt);
        solved = ext2int(solved, mpopt);
        define_constants;
        baseMVA = internal.baseMVA;
        bus0 = internal.bus;
        gen0 = internal.gen;
        branch0 = internal.branch;
        if size(branch0, 2) < QT
            branch0(:, end+1:QT) = 0;
        end
        [Ybus, Yf, Yt] = makeYbus(baseMVA, bus0, branch0);
        [ref, ~, ~] = bustypes(bus0, gen0);
        V = solved.bus(:, VM) .* exp(1j * pi / 180 * solved.bus(:, VA));

        for warmup = 1:5
            profile_one(baseMVA, bus0, gen0, branch0, Ybus, Yf, Yt, V, ref, i2e, mpopt);
        end
        samples = zeros(21, length(labels));
        for sample = 1:size(samples, 1)
            samples(sample, :) = profile_one( ...
                baseMVA, bus0, gen0, branch0, Ybus, Yf, Yt, V, ref, i2e, mpopt);
        end
        values = median(samples, 1);
        for index = 1:length(labels)
            fprintf('matpower_detail,%s,%s,%.6f\n', ...
                case_name, labels{index}, values(index));
        end
    end
end

function timing = profile_one(baseMVA, bus0, gen0, branch0, Ybus, Yf, Yt, V, ref, i2e, mpopt)
    define_constants;
    nb = size(bus0, 1);

    started = tic;
    Sd = makeSdzip(baseMVA, bus0, mpopt);
    load_model_ms = toc(started) * 1e3;

    started = tic;
    on_sbus = find(gen0(:, GEN_STATUS) > 0);
    gbus_sbus = gen0(on_sbus, GEN_BUS);
    ngon_sbus = length(on_sbus);
    online_index_ms = toc(started) * 1e3;

    started = tic;
    Cg_sbus = sparse(gbus_sbus, (1:ngon_sbus)', 1, nb, ngon_sbus);
    connection_matrix_ms = toc(started) * 1e3;

    started = tic;
    Sg = gen0(on_sbus, PG) + 1j * gen0(on_sbus, QG);
    generator_values_ms = toc(started) * 1e3;

    started = tic;
    Sbusg = Cg_sbus * Sg;
    sparse_matvec_ms = toc(started) * 1e3;

    started = tic;
    Sbusd = Sd.p + Sd.i + Sd.z;
    Sbus_result = Sbusg / baseMVA - Sbusd;
    combine_ms = toc(started) * 1e3;

    started = tic;
    bus = bus0;
    gen = gen0;
    branch = branch0;
    initialize_ms = toc(started) * 1e3;

    started = tic;
    bus(:, VM) = abs(V);
    bus(:, VA) = angle(V) * 180 / pi;
    bus_voltage_ms = toc(started) * 1e3;

    generator_index_started = tic;
    started = tic;
    on = find(gen(:, GEN_STATUS) > 0 & bus(gen(:, GEN_BUS), BUS_TYPE) ~= PQ);
    off = find(gen(:, GEN_STATUS) <= 0);
    gbus = gen(on, GEN_BUS);
    generator_indices_ms = toc(started) * 1e3;

    started = tic;
    Ybus_gbus = Ybus(gbus, :);
    generator_y_select_ms = toc(started) * 1e3;

    started = tic;
    Ibus = Ybus_gbus * V;
    generator_spmv_ms = toc(started) * 1e3;

    started = tic;
    Sbus = V(gbus) .* conj(Ibus);
    generator_combine_ms = toc(started) * 1e3;
    generator_index_ms = toc(generator_index_started) * 1e3;

    started = tic;
    gen(off, QG) = zeros(length(off), 1);
    [~, Qd_gbus] = total_load(bus(gbus, :), [], 'bus', [], mpopt);
    gen(on, QG) = imag(Sbus) * baseMVA + Qd_gbus;
    initial_q_ms = toc(started) * 1e3;

    started = tic;
    if length(on) > 1
        ngon = length(on);
        Cg = sparse((1:ngon)', gbus, ones(ngon, 1), ngon, nb);
        ngg = Cg * sum(Cg)';
        gen(on, QG) = gen(on, QG) ./ ngg;
        Qmin = gen(on, QMIN);
        Qmax = gen(on, QMAX);
        M = abs(gen(on, QG));
        M(~isinf(Qmax)) = M(~isinf(Qmax)) + abs(Qmax(~isinf(Qmax)));
        M(~isinf(Qmin)) = M(~isinf(Qmin)) + abs(Qmin(~isinf(Qmin)));
        M = Cg * Cg' * M;
        Qmin(Qmin == Inf) = M(Qmin == Inf);
        Qmin(Qmin == -Inf) = -M(Qmin == -Inf);
        Qmax(Qmax == Inf) = M(Qmax == Inf);
        Qmax(Qmax == -Inf) = -M(Qmax == -Inf);
        Cmin = sparse((1:ngon)', gbus, Qmin, ngon, nb);
        Cmax = sparse((1:ngon)', gbus, Qmax, ngon, nb);
        Qg_tot = Cg' * gen(on, QG);
        Qg_min = sum(Cmin)';
        Qg_max = sum(Cmax)';
        gen(on, QG) = Qmin + Cg * ((Qg_tot - Qg_min) ./ ...
            (Qg_max - Qg_min + eps)) .* (Qmax - Qmin);
        ig = find(abs(Cg * (Qg_min - Qg_max)) < 10 * eps);
        if ~isempty(ig)
            ib = find(sum(Cg(ig, :), 1)');
            mis = sparse(ib, 1, (Qg_tot(ib) - Qg_min(ib)) ./ ...
                sum(Cg(:, ib)', 2), nb, 1);
            gen(on(ig), QG) = Qmin(ig) + Cg(ig, :) * mis;
        end
    end
    q_distribution_ms = toc(started) * 1e3;

    started = tic;
    for k = 1:length(ref)
        refgen = find(gbus == ref(k));
        Pd_refk = total_load(bus(ref(k), :), [], 'bus', [], mpopt);
        gen(on(refgen(1)), PG) = real(Sbus(refgen(1))) * baseMVA + Pd_refk;
        if length(refgen) > 1
            gen(on(refgen(1)), PG) = gen(on(refgen(1)), PG) - ...
                sum(gen(on(refgen(2:end)), PG));
        end
    end
    slack_generation_ms = toc(started) * 1e3;

    branch_flow_started = tic;
    started = tic;
    out = find(branch(:, BR_STATUS) == 0);
    br = find(branch(:, BR_STATUS));
    fbus = branch(br, F_BUS);
    tbus = branch(br, T_BUS);
    branch_indices_ms = toc(started) * 1e3;

    started = tic;
    Yf_br = Yf(br, :);
    Yt_br = Yt(br, :);
    branch_y_select_ms = toc(started) * 1e3;

    started = tic;
    If = Yf_br * V;
    It = Yt_br * V;
    branch_spmv_ms = toc(started) * 1e3;

    started = tic;
    Sf = V(fbus) .* conj(If) * baseMVA;
    St = V(tbus) .* conj(It) * baseMVA;
    branch_combine_ms = toc(started) * 1e3;
    branch_flow_ms = toc(branch_flow_started) * 1e3;

    started = tic;
    branch(br, [PF, QF, PT, QT]) = [real(Sf), imag(Sf), real(St), imag(St)];
    branch(out, [PF, QF, PT, QT]) = zeros(length(out), 4);
    branch_write_ms = toc(started) * 1e3;

    started = tic;
    index_bus = bus0(:, BUS_I);
    output_bus = bus0;
    output_bus(:, BUS_I) = i2e(index_bus);
    int2ext_bus_ms = toc(started) * 1e3;

    started = tic;
    index_gen = gen0(:, GEN_BUS);
    output_gen = gen0;
    output_gen(:, GEN_BUS) = i2e(index_gen);
    int2ext_gen_ms = toc(started) * 1e3;

    started = tic;
    index_f = branch0(:, F_BUS);
    index_t = branch0(:, T_BUS);
    output_branch = branch0;
    output_branch(:, [F_BUS, T_BUS]) = [i2e(index_f), i2e(index_t)];
    int2ext_branch_ms = toc(started) * 1e3;

    timing = [load_model_ms, online_index_ms, connection_matrix_ms, ...
        generator_values_ms, sparse_matvec_ms, combine_ms, initialize_ms, ...
        bus_voltage_ms, generator_index_ms, generator_indices_ms, ...
        generator_y_select_ms, generator_spmv_ms, generator_combine_ms, ...
        initial_q_ms, q_distribution_ms, slack_generation_ms, branch_flow_ms, ...
        branch_indices_ms, branch_y_select_ms, branch_spmv_ms, ...
        branch_combine_ms, branch_write_ms, ...
        int2ext_bus_ms, int2ext_gen_ms, int2ext_branch_ms];
end
