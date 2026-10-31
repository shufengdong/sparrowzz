function benchmark_matpower_pf(cases,warmups,samples)
%BENCHMARK_MATPOWER_PF Same numeric .txt input and Newton settings as rspower.
if nargin<1 || isempty(cases)
    cases={'case5','case9','case14','case30','case39','case57','case118','case118zh','case300','case1354pegase','case2736sp','case2737sop','case13659pegase'};
end
if nargin<2, warmups=20; end
if nargin<3, samples=21; end
options=mpoption('verbose',0,'out.all',0,'pf.alg','NR','pf.tol',1e-6,'pf.nr.max_it',100,'pf.enforce_q_lims',0);
for k=1:numel(cases)
    name=cases{k};input=read_rspower_case(name);
    for i=1:warmups
        result=runpf(input,options); assert(result.success,'%s warmup did not converge',name);
    end
    for i=1:samples
        started=tic;result=runpf(input,options);elapsed=toc(started)*1000;
        assert(result.success,'%s sample did not converge',name);
        if i==1
            balance=power_balance(result);assert(balance<=1.01e-6,'%s balance %.12g',name,balance);
            fprintf('MATPOWER_PF_SIZE,%s,%d,%d,%d,%.12g\n',name,sum(result.bus(:,2)~=4),sum(result.gen(:,8)>0),sum(result.branch(:,11)>0),balance);
        end
        fprintf('MATPOWER_PF_SAMPLE,%s,%d,%.9f,%d\n',name,i-1,elapsed,result.iterations);
    end
end
end
function residual=power_balance(result)
bus=result.bus;gen=result.gen;branch=result.branch;
[~,g]=ismember(gen(:,1),bus(:,1));[~,f]=ismember(branch(:,1),bus(:,1));[~,t]=ismember(branch(:,2),bus(:,1));
on=gen(:,8)>0;active=branch(:,11)>0;n=size(bus,1);
p=-bus(:,3)-bus(:,5).*bus(:,8).^2+accumarray(g(on),gen(on,2),[n,1]);
q=-bus(:,4)+bus(:,6).*bus(:,8).^2+accumarray(g(on),gen(on,3),[n,1]);
p=p-accumarray(f(active),branch(active,14),[n,1])-accumarray(t(active),branch(active,16),[n,1]);
q=q-accumarray(f(active),branch(active,15),[n,1])-accumarray(t(active),branch(active,17),[n,1]);
p(bus(:,2)==4)=0;q(bus(:,2)==4)=0;residual=max(abs([p;q]))/result.baseMVA;
end
