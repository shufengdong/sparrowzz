function input = read_rspower_case(name)
path = fullfile(fileparts(mfilename('fullpath')),'..','data',[name '.txt']);
source = regexprep(fileread(path),'//[^\r\n]*','');
source = regexprep(source,'\]\s*,\s*\[',';');
source = regexprep(source,'\[\s*\[','[');
source = regexprep(source,'\]\s*\]',']');
eval(source);
if ~exist('gencost','var'), gencost=[]; end
input = struct('version','2','baseMVA',baseMVA,'bus',bus,'gen',gen,'branch',branch,'gencost',gencost);
end
