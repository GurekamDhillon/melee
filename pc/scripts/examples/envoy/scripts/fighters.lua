-- The fighters a run can be played as: the retail roster, and any Geno-defined fighter the mod folder lists in `missions/fighters.txt` (gd.mod_read reads only under missions/)
-- (the engine has no call that lists mounted Geno fighters, so the folder that mounts one says so: one `geno:<name>  <label>`
-- line per fighter, `#` comments). A token the engine would refuse is refused here, plainly, before a scene is launched.
return function(D)
 local F={}
 F.retail={}
 for name in ('fox falco mario luigi drmario peach bowser donkey captain ganondorf link younglink zelda sheik samus yoshi kirby pikachu pichu jigglypuff mewtwo ness marth roy iceclimbers gamewatch'):gmatch('%S+') do F.retail[#F.retail+1]=name end
 F.retail_set={};for _,n in ipairs(F.retail) do F.retail_set[n]=true end
 -- The extra (Geno) fighters: {id='geno:vanilla-hero',label='Vanilla Hero'} in file order.
 function F.parse(text)
  local out,seen={},{}
  for line in tostring(text or ''):gmatch('[^\r\n]+') do
   line=line:gsub('#.*$','')
   local id,label=line:match('^%s*(geno:[%w_%-]+)%s*(.-)%s*$')
   if id and not seen[id] and #id<=80 then seen[id]=true;out[#out+1]={id=id,label=(label~='' and label or id:sub(6))} end
  end
  return out
 end
 function F.extras(g)
  if not (g and g.mod_read) then return {} end
  local ok,text=pcall(g.mod_read,'missions/fighters.txt')
  if not ok or type(text)~='string' then return {} end
  return F.parse(text)
 end
 -- The menu's list: retail names, then the extras as {id,label} rows.
 function F.menu(g)
  local list={};for _,n in ipairs(F.retail) do list[#list+1]=n end
  for _,e in ipairs(F.extras(g)) do list[#list+1]={id=e.id,label=e.label} end
  return list
 end
 -- A token from a command: the id, or nil and a plain reason.
 function F.resolve(g,token)
  token=tostring(token or ''):lower()
  if token=='' then return nil,'no fighter given' end
  if F.retail_set[token] then return token end
  for _,e in ipairs(F.extras(g)) do if e.id:lower()==token then return e.id end end
  if token:match('^geno:') then return nil,("Geno fighter '"..token.."' is not mounted here (add it to this mod's fighters.txt)") end
  return nil,("unknown fighter '"..token.."': use one of "..table.concat(F.retail,', ')..' or a geno:<name> listed in missions/fighters.txt')
 end
 -- A name for a nameplate: the engine's own unless it only says "character 127" (a Geno fighter's number), then the label the
 -- fighter list gives (the only extra fighter's, else a plain "Custom fighter").
 function F.plate(g,engine_name,label)
  local n=tostring(engine_name or '')
  if n=='' or n=='nil' or n:match('^character %d+$') then
   if label then return label end
   local extras=F.extras(g);if #extras==1 then return extras[1].label end
   return 'Custom fighter'
  end
  return n
 end
 return F
end
