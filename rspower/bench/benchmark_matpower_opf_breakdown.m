function benchmark_matpower_opf_breakdown(ipopt_directory, cases, warmups, samples, native_timing)
% Compare callback stages with Rust, using MATPOWER7.1/IPOPT.
% Instrument temporary copies; leave the installed MATPOWER files untouched.
if nargin<1, ipopt_directory=''; end
if nargin<2, cases={'case300','case1354pegase','case2736sp'}; end
if nargin<3, warmups=20; end
if nargin<4, samples=21; end
if nargin<5, native_timing=false; end
shadow=tempname; mkdir(shadow);
original_path=path;
cleanup=onCleanup(@() restore_files(original_path,shadow));

source=fileread(which('nlps_ipopt'));
for item={'objective','gradient','constraints','jacobian','hessian'}
    name=item{1};
    stage=name;
    if strcmp(name,'objective') || strcmp(name,'gradient'), stage='f_df'; end
    if strcmp(name,'constraints') || strcmp(name,'jacobian'), stage='g_jacobian'; end
    args='x,d';
    if strcmp(name,'hessian'), args='x,sigma,lambda,d'; end
    pattern=['(?m)^(funcs\.' name '\s*=\s*)@' name ';'];
    assert(numel(regexp(source,pattern))==1,'MATPOWER callback layout changed');
    replacement=['$1@(' args ') opf_bench_call(''' stage ''',@' name ',' args ');'];
    source=regexprep(source,pattern,replacement);
end
source=replace_once(source,'[x, info] = ipopt_auxdata(x0,funcs,options);', ...
    '[x, info] = opf_bench_call(''ipopt'',@ipopt_auxdata,x0,funcs,options);');
source=replace_once(source,'[x, info] = ipopt(x0,funcs,options);', ...
    '[x, info] = opf_bench_call(''ipopt'',@ipopt,x0,funcs,options);');
write_source(fullfile(shadow,'nlps_ipopt.m'),source);

source=fileread(which('nlp_hessfcn'));
source=replace_once(source,'[f, df, d2f] = nlp_costfcn(om, x);', ...
    '[f, df, d2f] = opf_bench_call(''cost_hessian'',@nlp_costfcn,om,x);');
source=replace_once(source,'d2G = om.eval_nln_constraint_hess(x, lambda.eqnonlin, 1);', ...
    'd2G = opf_bench_call(''power_hessian'',@() om.eval_nln_constraint_hess(x,lambda.eqnonlin,1));');
source=replace_once(source,'d2H = om.eval_nln_constraint_hess(x, lambda.ineqnonlin, 0);', ...
    'd2H = opf_bench_call(''flow_hessian'',@() om.eval_nln_constraint_hess(x,lambda.ineqnonlin,0));');
write_source(fullfile(shadow,'nlp_hessfcn.m'),source);

% Export the local helper to the same temporary directory as the copied functions.
source=fileread([mfilename('fullpath') '.m']);
start=regexp(source,'(?m)^function varargout = opf_bench_call\(');
assert(numel(start)==1);
write_source(fullfile(shadow,'opf_bench_call.m'),source(start:end));
addpath(shadow,'-begin');
clear nlps_ipopt nlp_hessfcn opf_bench_call;
diagnostic=1+double(native_timing);
benchmark_matpower_opf(ipopt_directory,cases,warmups,samples,{'IPOPT'},diagnostic);
end

function source=replace_once(source,before,after)
assert(numel(strfind(source,before))==1,'MATPOWER7.1 instrumentation location changed');
source=strrep(source,before,after);
end

function write_source(name,source)
file=fopen(name,'w'); assert(file~=-1);
cleanup=onCleanup(@() fclose(file));
fprintf(file,'%s',source);
end

function restore_files(original_path,shadow)
path(original_path);
clear nlps_ipopt nlp_hessfcn opf_bench_call;
if exist(shadow,'dir'), rmdir(shadow,'s'); end
end

function varargout = opf_bench_call(stage,fun,varargin)
global OPF_BENCH_PROFILE;
is_solver=strcmp(stage,'ipopt');
if ~is_solver && ~OPF_BENCH_PROFILE.in_solver
    [varargout{1:nargout}]=fun(varargin{:});
    return;
end
if is_solver
    OPF_BENCH_PROFILE.in_solver=true;
    if OPF_BENCH_PROFILE.native_timing
        options=varargin{3};
        options.ipopt.print_level=5;
        options.ipopt.print_timing_statistics='yes';
        varargin{3}=options;
    end
end
started=tic;
try
    [varargout{1:nargout}]=fun(varargin{:});
catch exception
    if is_solver, OPF_BENCH_PROFILE.in_solver=false; end
    rethrow(exception);
end
elapsed=toc(started);
if is_solver, OPF_BENCH_PROFILE.in_solver=false; end
if ~isfield(OPF_BENCH_PROFILE.times,stage)
    OPF_BENCH_PROFILE.times.(stage)=0;
    OPF_BENCH_PROFILE.calls.(stage)=0;
end
OPF_BENCH_PROFILE.times.(stage)=OPF_BENCH_PROFILE.times.(stage)+elapsed;
OPF_BENCH_PROFILE.calls.(stage)=OPF_BENCH_PROFILE.calls.(stage)+1;
end
