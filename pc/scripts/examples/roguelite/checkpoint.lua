-- Versioned checkpoint envelope (TBD3). Bundles one validated generation:
-- profile, optional run, resolved run manifest, roster metadata and the
-- separate run-progress record, with a length-delimited body and a cheap
-- integrity checksum. The caller owns A/B file selection and atomic rename;
-- this module only frames and validates.
--
-- Decoding does not regenerate or migrate content: it restores exactly what was
-- written. Core's codec validates profile/run; Codec validates the manifest and
-- progress tables. The caller must still validate route/progress semantics
-- (see route.lua); a valid table is not a valid run.
local Checkpoint = {version = 3, tag = 'TBD3', max_body = 1048576, max_generation = 1000000000}

local function checksum(text)
  -- Polynomial hash mod 2^31-1 keeps every product below 2^53, so results are
  -- identical on any IEEE-754 double platform and do not need bitwise ops.
  local h = 7
  for i = 1, #text do h = (h * 131 + text:byte(i)) % 2147483647 end
  return string.format('%08x', h)
end
Checkpoint.checksum = checksum

local function is_text(v, max) return type(v) == 'string' and #v <= max end

function Checkpoint.encode(fields)
  assert(type(fields) == 'table', 'invalid checkpoint fields')
  local generation = fields.generation
  assert(type(generation) == 'number' and generation % 1 == 0 and generation >= 0 and generation <= Checkpoint.max_generation, 'invalid generation')
  local profile = fields.profile
  assert(is_text(profile, Checkpoint.max_body) and #profile > 0, 'invalid profile text')
  local run = fields.run
  assert(run == nil or is_text(run, Checkpoint.max_body), 'invalid run text')
  local manifest = fields.manifest
  assert(manifest == nil or is_text(manifest, Checkpoint.max_body), 'invalid manifest text')
  local roster = fields.roster
  assert(roster == nil or is_text(roster, Checkpoint.max_body), 'invalid roster text')
  local progress = fields.progress
  assert(progress == nil or is_text(progress, Checkpoint.max_body), 'invalid progress text')
  run, manifest, roster, progress = run or '', manifest or '', roster or '', progress or ''
  local body = profile .. run .. manifest .. roster .. progress
  assert(#body <= Checkpoint.max_body, 'checkpoint body too large')
  local header = string.format('%s %d %d %d %d %d %d %s\n',
    Checkpoint.tag, generation, #profile, #run, #manifest, #roster, #progress, checksum(body))
  return header .. body
end

-- `opts` may carry expected schema versions: opts.progress_version and
-- opts.manifest_schema. A section of the wrong version is refused with a
-- "preserved" reason rather than returned as if interchangeable.
function Checkpoint.decode(text, core, codec, opts)
  if type(text) ~= 'string' then return nil, 'invalid checkpoint text' end
  if #text > Checkpoint.max_body + 128 then return nil, 'checkpoint too large' end
  local tag = text:match('^(TBD%d+) ')
  if not tag then return nil, 'missing checkpoint header' end
  if tag ~= Checkpoint.tag then return nil, 'unsupported checkpoint version ' .. tag:sub(4) .. '; preserved' end
  -- Current form carries a progress section (glen); the earlier roster-only
  -- form is accepted for round-trip compatibility.
  local generation, plen, rlen, mlen, xlen, glen, sum = text:match('^TBD3 (%d+) (%d+) (%d+) (%d+) (%d+) (%d+) (%x+)\n')
  if not generation then
    generation, plen, rlen, mlen, xlen, sum = text:match('^TBD3 (%d+) (%d+) (%d+) (%d+) (%d+) (%x+)\n')
    glen = 0
  end
  generation, plen, rlen, mlen, xlen, glen = tonumber(generation), tonumber(plen), tonumber(rlen), tonumber(mlen), tonumber(xlen), tonumber(glen)
  if not (generation and plen and rlen and mlen and xlen and glen and sum) then return nil, 'malformed checkpoint header' end
  if generation > Checkpoint.max_generation then return nil, 'invalid generation' end
  local _, body_start = text:find('\n', 1, true)
  local body = text:sub(body_start + 1)
  if #body ~= plen + rlen + mlen + xlen + glen then return nil, 'checkpoint length mismatch' end
  if checksum(body) ~= sum then return nil, 'checkpoint checksum mismatch' end
  assert(core and core.restore and codec and codec.decode, 'decoder dependencies required')
  local profile = core.restore(body:sub(1, plen))
  if not profile or profile.type ~= 'profile' then return nil, 'invalid profile section' end
  local run = nil
  if rlen > 0 then
    run = core.restore(body:sub(plen + 1, plen + rlen))
    if not run or run.type ~= 'run' or run.owner ~= profile.id then return nil, 'invalid run section' end
  end
  opts = opts or {}
  local manifest = nil
  if mlen > 0 then
    local ok, decoded = pcall(codec.decode, body:sub(plen + rlen + 1, plen + rlen + mlen))
    if not ok or type(decoded) ~= 'table' then return nil, 'invalid manifest section' end
    if opts.manifest_schema ~= nil and decoded.schema_version ~= opts.manifest_schema then
      return nil, string.format('unsupported manifest schema %s; preserved', tostring(decoded.schema_version))
    end
    manifest = decoded
  end
  local roster = xlen > 0 and body:sub(plen + rlen + mlen + 1, plen + rlen + mlen + xlen) or nil
  local progress = nil
  if glen > 0 then
    local ok, decoded = pcall(codec.decode, body:sub(plen + rlen + mlen + xlen + 1))
    if not ok or type(decoded) ~= 'table' then return nil, 'invalid progress section' end
    if opts.progress_version ~= nil and decoded.version ~= opts.progress_version then
      return nil, string.format('unsupported progress version %s; preserved', tostring(decoded.version))
    end
    progress = decoded
  end
  return {generation = generation, profile = profile, run = run, manifest = manifest,
    roster = roster, progress = progress}
end

return Checkpoint
