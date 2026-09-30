-- Pure migration from the legacy TBD1/TBD2 save envelopes to the versioned
-- TBD3 checkpoint. Legacy envelopes stored only profile, run and roster text,
-- never a resolved manifest; the manifest is reconstructed with the frozen v1
-- generator from the run's recorded world seed so the historical route and room
-- IDs are preserved rather than regenerated with a newer generator.
--
-- Dependencies are injected: core (Core.restore), codec (Codec.encode),
-- checkpoint (Checkpoint.encode) and v1 (frozen dungeon_v1 with generate/validate).
local Legacy = {version = 1, max_generation = 1000000000}

function Legacy.parse(text)
  if type(text) ~= 'string' then return nil, 'invalid checkpoint text' end
  local generation, plen, rlen, mlen, body = text:match('^TBD2 (%d+) (%d+) (%d+) (%d+)\n(.*)$')
  local version = 2
  if not generation then
    generation, plen, rlen, body = text:match('^TBD1 (%d+) (%d+) (%d+)\n(.*)$')
    mlen, version = 0, 1
  end
  if not generation then
    local tag = text:match('^(TBD%d+) ')
    if tag then return nil, 'unsupported legacy version ' .. tag:sub(4) end
    return nil, 'not a legacy checkpoint'
  end
  generation, plen, rlen, mlen = tonumber(generation), tonumber(plen), tonumber(rlen), tonumber(mlen)
  if not (generation and plen and rlen and mlen) or generation > Legacy.max_generation then return nil, 'malformed legacy header' end
  if plen < 1 or plen > 262144 or rlen > 262144 or mlen > 192 then return nil, 'legacy section out of range' end
  if #body ~= plen + rlen + mlen then return nil, 'legacy length mismatch' end
  return {version = version, generation = generation, profile_text = body:sub(1, plen),
    run_text = rlen > 0 and body:sub(plen + 1, plen + rlen) or nil,
    roster_text = mlen > 0 and body:sub(plen + rlen + 1) or nil}
end

-- Returns the TBD3 text, or nil, reason. `deps` = {core, codec, checkpoint, v1}.
function Legacy.migrate(text, deps)
  assert(type(deps) == 'table' and deps.core and deps.codec and deps.checkpoint and deps.v1, 'migration dependencies required')
  local parsed, why = Legacy.parse(text)
  if not parsed then return nil, why end
  local profile = deps.core.restore(parsed.profile_text)
  if not profile or profile.type ~= 'profile' then return nil, 'invalid legacy profile' end
  local run = nil
  if parsed.run_text then
    run = deps.core.restore(parsed.run_text)
    if not run or run.type ~= 'run' then return nil, 'invalid legacy run' end
    if run.owner ~= profile.id then return nil, 'legacy run belongs to another profile' end
    local serial = run.id and run.id:match('^run(%d+)$')
    if not serial or tonumber(serial) >= profile.next_run then return nil, 'legacy run counter inconsistent' end
  end
  local manifest_text
  if run then
    local ok, manifest = pcall(deps.v1.generate, run.world_seed)
    if not ok or type(manifest) ~= 'table' then return nil, 'frozen v1 generator could not reproduce the route' end
    local valid, reason = deps.v1.validate(manifest)
    if not valid then return nil, 'reconstructed legacy route is invalid: ' .. tostring(reason) end
    if not manifest.nodes[run.progress and run.progress.room or manifest.start] then return nil, 'legacy progress room absent from reconstructed route' end
    manifest_text = deps.codec.encode(manifest)
    if not manifest_text then return nil, 'reconstructed route did not encode' end
  end
  local ok, encoded = pcall(deps.checkpoint.encode, {
    generation = parsed.generation, profile = parsed.profile_text, run = parsed.run_text,
    manifest = manifest_text, roster = parsed.roster_text,
  })
  if not ok then return nil, 'migration encode failed: ' .. tostring(encoded) end
  return encoded
end

return Legacy
