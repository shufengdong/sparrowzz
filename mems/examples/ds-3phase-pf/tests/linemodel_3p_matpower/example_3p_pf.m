% 测试ieee13配电网算例的单段导线
clc;
mpopt = mpoption('verbose',2);
mpc = loadcase('t_case3p_a_1');
run_pf(mpc,mpopt,'mpx',mp.xt_3p);


% 以下为手工计算结果
r = [ 0.3465  0.1560  0.1580 ; 0.1560  0.3375  0.1535 ; 0.1580 0.1535  0.3414 ];
x= [1.0179 0.5017 0.4236; 0.5017 1.0478 0.3849; 0.4236 0.3849 1.0348];
len = 2000/5280;
y = inv(r+1j*x)/len;
v1 = [complexd(7.1996,0.00)  ;  complexd(7.1996,-120.00)  ;  complexd(7.1996,120.00)].*1000;
v2 = [complexd(7.1769,-0.08) ;   complexd(7.1396,-120.37) ;   complexd(7.1500,119.25)].*1000;
i = y*(v2-v1);
abs(i)
rad2deg(angle(i))

pd = mpc.load3p(1,4:6);
pf = mpc.load3p(1,7:9);
qd = pd .* tan(acos(pf));

