local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local G=T.module('genetics')
T.test('full paired DNA and independent copies',function()
  local a,b=G.new('C'),G.new('C');G.validate(a);a.power[1]='S';assert(b.power[1]=='C')
  for _,k in ipairs(G.traits) do assert(#a[k]==2) end
end)
T.test('every grade pair expresses higher at 70 percent boundary',function()
  for _,a in ipairs(G.grades) do for _,b in ipairs(G.grades) do
    local d=G.new();d.power={a,b}
    local hi=G.rank[a]>G.rank[b] and a or b;local lo=hi==a and b or a
    assert(G.express(d,function() return .699999 end).power==hi)
    assert(G.express(d,function() return .7 end).power==lo)
  end end
end)
T.test('each child allele comes from its respective parent without mutation',function()
  local a,b=G.new('E'),G.new('S');a.colour={'red','blue'};b.colour={'green','yellow'}
  for _,r in ipairs({0,.499,.5,.999}) do
    local c=G.blend(a,b,function() return r end)
    local i=r<.5 and 1 or 2
    for _,k in ipairs(G.traits) do assert(c[k][1]==a[k][i] and c[k][2]==b[k][i]) end
    c.power[1]='C';assert(a.power[1]=='E')
  end
end)
T.test('colour dominates normal, two-tone random allele, shiny OR',function()
  local d=G.new();d.colour={'normal','red'};d.two_tone={false,true};d.shiny={false,true}
  local e=G.express(d,function() return .9 end);assert(e.colour=='red' and e.two_tone and e.shiny)
  d.colour={'red','blue'};assert(G.express(d,function() return 0 end).colour=='red')
end)
T.test('invalid DNA and random samples refused',function()
  for _,bad in ipairs({'X',1}) do local d=G.new();d.guard[1]=bad;T.refuses(function() G.validate(d) end) end
  local d=G.new();d.shiny[1]=1;T.refuses(function() G.validate(d) end)
  T.refuses(function() G.express(G.new(),function() return 1 end) end)
end)
T.test('all four parental grade alleles and independent inheritance choices',function()
  for _,a in ipairs(G.grades) do for _,b in ipairs(G.grades) do
    for _,c in ipairs(G.grades) do for _,d in ipairs(G.grades) do
      local left,right=G.new(),G.new();left.power={a,b};right.power={c,d}
      for li=1,2 do for ri=1,2 do
        local call=0;local child=G.blend(left,right,function()
          call=call+1;local choice=call%2==1 and li or ri;return choice==1 and .1 or .9
        end)
        assert(child.power[1]==left.power[li] and child.power[2]==right.power[ri])
      end end
    end end
  end end
end)
T.test('all boolean allele pairs express shiny dominance and two-tone choice',function()
  for _,a in ipairs({false,true}) do for _,b in ipairs({false,true}) do
    local d=G.new();d.shiny={a,b};d.two_tone={a,b}
    for _,n in ipairs({.1,.9}) do
      local e=G.express(d,function() return n end)
      assert(e.shiny==(a or b));assert(e.two_tone==d.two_tone[n<.5 and 1 or 2])
    end
  end end
end)
T.done()
