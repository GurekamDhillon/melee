-- One capability: owned retail run launch/restart/cancel controls.
local status='Press C: Classic Mario Normal, 3 stocks, NG+ enabled; B: end'
function on_key_down(key)
 if key=='C' then
  local ok,why=gd.start_1p{mode='classic',fighter='mario',difficulty=2,stocks=3,loop=true}
  status=ok and 'Classic started; complete normally to loop' or tostring(why)
 elseif key=='B' then gd.end_1p();status='Run ended' end
end
function on_1p_complete(e) status='Completed NG+'..e.loop..'; starting next playthrough' end
function on_1p_game_over() status='Game over: run ended' end
function on_unload() gd.loop_1p(false);gd.end_1p() end
function on_draw() gd.kit.text(status,24,28,14,'bone') end
