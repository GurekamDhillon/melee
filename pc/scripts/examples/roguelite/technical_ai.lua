-- Lifecycle adapter for the original native Fox/Falco technical assist.
-- No combat decisions are translated into Lua; all pulses use native input.
local A={}
function A.configure(port,skill,seed)
 if not gd.cpu_technical then return false,'Native technical assist unavailable' end
 if type(skill)~='number' or skill%1~=0 or skill<0 or skill>3 or
  type(seed)~='number' or seed%1~=0 or seed<1 or seed>2147483647 then return false,'Invalid technical assist settings' end
 local ok,enabled=pcall(gd.cpu_technical,port,skill,seed)
 if not ok or not enabled then return false,'Technical assist refused; native CPU baseline retained' end
 return true,skill==0 and 'Technical assist disabled' or
  ('Original technical assist '..skill..' / '..(skill==1 and 6 or skill==2 and 4 or 2)..' frame observation delay')
end
function A.clear(port)
 if not gd.cpu_technical then return false end
 local ok,result=pcall(gd.cpu_technical,port,0,1);return ok and result==true
end
function A.status(port)
 if not gd.cpu_technical then return {enabled=false,policy='Native CPU baseline'} end
 local ok,state=pcall(gd.cpu_technical,port)
 return ok and type(state)=='table' and state or {enabled=false,policy='Native CPU baseline'}
end
return A
