-- @name: Six full Geno fighters / heap 3 load regression
-- @gameplay: true
-- @rollback_safe: false
-- Pair with geno_six_full_heap3.py, which asserts lbDvd's after-load heap
-- measurements. The native --test suite has no game boot/frame driver.
-- Mount the stress6 geno-falcon-1..6 fixtures, then launch mode=workbench
-- with MELEE_WB_HEADLESS=1 (or mode=vs for the rendered CPU smoke).
-- See .omo/evidence/report-heap3.md for exact run.sh commands.
gd.run(function()
    local ok, why = pcall(function()
        assert(gd.wait_until(function()
            local m = gd.match()
            return m and m.active
        end, 9000), 'six-fighter match never became active')
        assert(not gd.match().netplay, 'an offline fixture is required')
        -- Workbench pauses at its first ready fighter, before tasks run.
        -- Resume that frame so this coroutine can reach the loaded-match probe.
        gd.resume()
        assert(gd.wait_until(function()
            local m = gd.match()
            return m and m.active and m.frame >= 60
        end, 9000), 'six-fighter match did not advance')
        local match = gd.match()
        assert(match.stage_external == 121, 'expected ACE external stage 121, got ' ..
               tostring(match.stage_external))
        gd.log(string.format('HEAP3_SIX_FULL stage_ext=%d stage_internal=%d',
                             match.stage_external, match.stage))
        local kinds = {}
        for port = 1, 6 do
            local p = gd.player(port)
            assert(p and p.cpu, 'missing CPU in port ' .. port)
            assert(p.char_name == 'Geno Falcon ' .. port,
                   'wrong fighter in port ' .. port .. ': ' .. tostring(p.char_name))
            assert(not kinds[p.kind], 'fighters must use six distinct Geno definitions')
            kinds[p.kind] = true
            gd.log(string.format('HEAP3_SIX_FULL player=%d kind=%d name=%s',
                                 port, p.kind, p.char_name))
        end
        -- Exercise normal CPU play too; do not capture disc-derived images.
        gd.wait(600)
        assert(gd.match().active and gd.match().stage_external == 121,
               'heavy-stage match ended or changed during CPU play')
        for port = 1, 6 do
            assert(gd.player(port), 'fighter disappeared from port ' .. port)
        end
    end)
    gd.log('TEST geno_six_full_heap3 players ' ..
           (ok and 'PASS' or ('FAIL ' .. tostring(why))))
    gd.quit()
end)
