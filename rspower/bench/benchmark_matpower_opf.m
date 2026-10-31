function benchmark_matpower_opf(ipopt_directory, cases, warmups, samples, solvers, diagnostic)
%BENCHMARK_MATPOWER_OPF  Current generic OPF comparison; CSV samples like PF.
% Read the same rspower/data/*.txt as Rust. Exclude input I/O and MATLAB startup.
% Run with -singleCompThread, CPU affinity and the six thread variables set to1.
% DIAGNOSTIC is reserved for benchmark_matpower_opf_breakdown.
if nargin < 1, ipopt_directory = ''; end
if nargin < 2, cases = {'case9','case30','case57','case145','case300','case1354pegase','case2736sp'}; end
if nargin < 3, warmups = 20; end
if nargin < 4, samples = 21; end
if nargin < 6, diagnostic = 0; end
assert(warmups>=0 && warmups==fix(warmups) && samples>0 && samples==fix(samples));
if ~isempty(ipopt_directory)
    addpath(ipopt_directory, '-begin');
    rehash;
    have_feature('all','clear_cache');
end
maxNumCompThreads(1);
fprintf('OPF_ENV,matlab,%s\n', version);
fprintf('OPF_ENV,matpower,%s\n', mpver);
% Resolve a newly added MEX before MATPOWER's exist()-based detection.
fprintf('OPF_ENV,ipopt_path,%s\n',which('ipopt'));
available_solvers = {'MIPS'};
if have_feature('ipopt')
    available_solvers{end+1} = 'IPOPT';
    fprintf('OPF_ENV,ipopt,%s\n', have_feature('ipopt', 'vstr'));
else
    fprintf('OPF_UNAVAILABLE,IPOPT,no callable IPOPT MEX\n');
end
if nargin < 5, solvers = available_solvers; end
assert(all(ismember(solvers,available_solvers)), 'Requested solver is unavailable');
clear mpoption;
for name = cases
    original = read_rspower_case(name{1});
    internal = ext2int(original);
    nb=size(internal.bus,1); ng=size(internal.gen,1); nl=size(internal.branch,1);
    fprintf('MATPOWER_SIZE,%s,%d,%d,%d,%d\n',name{1},nb,ng,nl,2*nb+2*ng);
    for solver = solvers
      try
        options = mpoption('verbose',0,'out.all',0,'opf.ac.solver',solver{1}, ...
            'opf.start',2,'opf.use_vg',0,'opf.v_cartesian',0,'opf.flow_lim','S', ...
            'opf.violation',1e-9,'mips.feastol',1e-9,'mips.gradtol',1e-9, ...
            'mips.comptol',1e-9,'mips.costtol',1e-9,'mips.max_it',300);
        if strcmp(solver{1}, 'IPOPT')
            % Same primary tolerances and adaptive strategy as Rust/IPOPT.
            settings = struct('tol',1e-9,'constr_viol_tol',1e-9,'max_iter',300, ...
                'linear_solver','mumps','mu_strategy','adaptive','bound_relax_factor',0,'acceptable_iter',0, ...
                'dual_inf_tol',1,'compl_inf_tol',1e-4,'acceptable_tol',1e-6, ...
                'acceptable_constr_viol_tol',1e-2,'acceptable_compl_inf_tol',1e-2);
            options = mpoption(options,'ipopt.opts',settings);
        end
        % Warm MATLAB's JIT before collecting the 21 timed samples.
        for sample = 1-warmups:samples
            if diagnostic
                global OPF_BENCH_PROFILE;
                OPF_BENCH_PROFILE=struct('in_solver',false,'native_timing',diagnostic==2,'times',struct(),'calls',struct());
            end
            cpu_started = cputime;
            started = tic;
            result = runopf(original, options);
            elapsed = toc(started);
            cpu_elapsed = cputime-cpu_started;
            assert(result.success, 'OPF failed: %s/%s, iterations=%d', ...
                name{1},solver{1},result.raw.output.iterations);
            assert(isfinite(result.f), 'Nonfinite OPF objective');
            violation = balance_violation(result);
            assert(violation<1e-7, 'OPF power mismatch exceeds common threshold');
            if sample > 0
                fprintf('MATPOWER_OPF,%s,%s,%d,%.9f,%.12f,%d,%.12g,%.9f\n', ...
                    name{1},solver{1},sample,elapsed,result.f,result.raw.output.iterations,violation,cpu_elapsed);
                if diagnostic
                    stages=fieldnames(OPF_BENCH_PROFILE.times);
                    for stage=stages'
                        fprintf('MATPOWER_PHASE,%s,%d,%s,%.9f,%d\n',name{1},sample,stage{1}, ...
                            OPF_BENCH_PROFILE.times.(stage{1}),OPF_BENCH_PROFILE.calls.(stage{1}));
                    end
                    t=OPF_BENCH_PROFILE.times;
                    native=t.ipopt-t.f_df-t.g_jacobian-t.hessian;
                    fprintf('MATPOWER_PHASE,%s,%d,ipopt_and_boundary,%.9f,1\n',name{1},sample,native);
                    fprintf('MATPOWER_PHASE,%s,%d,outside_ipopt,%.9f,1\n',name{1},sample,elapsed-t.ipopt);
                end
            end
        end
      catch exception
        fprintf('MATPOWER_OPF_FAILURE,%s,%s,%s\n',name{1},solver{1},strrep(exception.message,newline,' '));
      end
    end
end
end


function violation = balance_violation(result)
% Independent MW/MVAr balance from reported powers, including bus shunts.
bus = result.bus; generators = result.gen; branch = result.branch;
[~, gen_bus] = ismember(generators(:,1),bus(:,1));
[~, from] = ismember(branch(:,1),bus(:,1));
[~, to] = ismember(branch(:,2),bus(:,1));
n = size(bus,1); online = generators(:,8)>0; active = branch(:,11)>0;
p = -bus(:,3)-bus(:,5).*bus(:,8).^2;
q = -bus(:,4)+bus(:,6).*bus(:,8).^2;
p = p+accumarray(gen_bus(online),generators(online,2),[n 1]);
q = q+accumarray(gen_bus(online),generators(online,3),[n 1]);
p = p-accumarray(from(active),branch(active,14),[n 1])-accumarray(to(active),branch(active,16),[n 1]);
q = q-accumarray(from(active),branch(active,15),[n 1])-accumarray(to(active),branch(active,17),[n 1]);
p(bus(:,2)==4)=0; q(bus(:,2)==4)=0;
violation = max(abs([p;q]))/result.baseMVA;
end
