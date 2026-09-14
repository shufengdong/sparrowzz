function benchmark_blockcat_select()
%BENCHMARK_BLOCKCAT_SELECT  测量 MATLAB 的块拼接、水平/垂直拼接和二维索引。
%   尺寸和稀疏结构与 TensorEval 基准保持一致，用于验证 PF 中
%   select/blockcat 内核。每项预热 10 轮，再取 31 个批次
%   平均耗时的中位数。关键结果记录在 PF_PERFORMANCE.md。
    n = 13659;
    n1 = 6830;
    n2 = n - n1;

    A = ones(512, 512);
    B = 2 * ones(512, 512);
    C = 3 * ones(512, 512);
    D = 4 * ones(512, 512);

    SA = deterministic_sparse(n1, n1, 6, 0);
    SB = deterministic_sparse(n1, n2, 6, 1);
    SC = deterministic_sparse(n2, n1, 6, 2);
    SD = deterministic_sparse(n2, n2, 6, 3);

    dense = reshape(0:(1024 * 1024 - 1), 1024, 1024);
    sparse_value = deterministic_sparse(n, n, 12, 4);
    dense_indices = index_groups(1024);
    sparse_indices = index_groups(n);
    dense_chunks = cell(1, 8);
    complex_chunks = cell(1, 8);
    sparse_chunks = cell(1, 8);
    for chunk = 1:8
        dense_chunks{chunk} = chunk * ones(256, 64);
        complex_chunks{chunk} = complex(chunk * ones(256, 64), 0.5 * ones(256, 64));
        sparse_chunks{chunk} = deterministic_sparse(256, 64, 6, chunk - 1);
    end

    dense_block_us = median_time_us(@() [A, B; C, D], 20, 31);
    sparse_block_us = median_time_us(@() [SA, SB; SC, SD], 20, 31);
    dense_select_us = median_time_us(@() dense(dense_indices, dense_indices), 20, 31);
    sparse_select_us = median_time_us(@() sparse_value(sparse_indices, sparse_indices), 10, 31);
    dense_horzcat_us = median_time_us(@() horzcat(dense_chunks{:}), 50, 31);
    dense_vertcat_us = median_time_us(@() vertcat(dense_chunks{:}), 50, 31);
    complex_horzcat_us = median_time_us(@() horzcat(complex_chunks{:}), 20, 31);
    complex_vertcat_us = median_time_us(@() vertcat(complex_chunks{:}), 20, 31);
    sparse_horzcat_us = median_time_us(@() horzcat(sparse_chunks{:}), 50, 31);
    sparse_vertcat_us = median_time_us(@() vertcat(sparse_chunks{:}), 50, 31);

    fprintf('matlab,dense_blockcat_1024,%.3f\n', dense_block_us);
    fprintf('matlab,sparse_blockcat_13659,%.3f\n', sparse_block_us);
    fprintf('matlab,dense_select_1024_75pct,%.3f\n', dense_select_us);
    fprintf('matlab,sparse_select_13659_75pct,%.3f\n', sparse_select_us);
    fprintf('matlab,dense_horzcat_8x256x64,%.3f\n', dense_horzcat_us);
    fprintf('matlab,dense_vertcat_8x256x64,%.3f\n', dense_vertcat_us);
    fprintf('matlab,complex_horzcat_8x256x64,%.3f\n', complex_horzcat_us);
    fprintf('matlab,complex_vertcat_8x256x64,%.3f\n', complex_vertcat_us);
    fprintf('matlab,sparse_horzcat_8x256x64,%.3f\n', sparse_horzcat_us);
    fprintf('matlab,sparse_vertcat_8x256x64,%.3f\n', sparse_vertcat_us);
end

function value = deterministic_sparse(rows, columns, entries_per_row, phase)
    row = repelem((0:(rows - 1))', entries_per_row, 1);
    offset = repmat((0:(entries_per_row - 1))', rows, 1);
    column = mod(row * 17 + offset * 7919 + phase, columns);
    data = row + column + phase + 1;
    value = sparse(row + 1, column + 1, data, rows, columns);
end

function indices = index_groups(size_)
    indices = [];
    for remainder = 0:2
        indices = [indices, (remainder + 1):4:size_]; %#ok<AGROW>
    end
end

function result = median_time_us(operation, iterations, samples)
    for index = 1:10
        operation();
    end
    elapsed = zeros(samples, 1);
    for sample = 1:samples
        tic;
        for iteration = 1:iterations
            operation();
        end
        elapsed(sample) = toc * 1e6 / iterations;
    end
    result = median(elapsed);
end
