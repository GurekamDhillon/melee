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
  -- Three polynomial steps share one modulo and one C byte lookup. Even with
  -- double-only Lua, h*131^3 + byte terms stays below 2^53 (under 4.83e15),
  -- so batching preserves the exact historical integer hash. Four would not.
  local h, length = 7, #text
  local batched = length - length % 3
  for i = 1, batched, 3 do
    local a, b, c = text:byte(i, i + 2)
    h = (h * 2248091 + a * 17161 + b * 131 + c) % 2147483647
  end
  for i = batched + 1, length do h = (h * 131 + text:byte(i)) % 2147483647 end
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

-- `opts` may carry expected schema versions. Singular opts.manifest_schema /
-- opts.progress_version accept one exact version; plural opts.manifest_schemas /
-- opts.progress_versions accept a set of supported versions. A section of an
-- unsupported version is refused with a "preserved" reason rather than returned
-- as if interchangeable, so a future inner schema is never silently adopted.
-- A legacy manifest with no schema_version field is treated as its `version`.
--
-- Returns `record`, or nil, reason, preserve. `preserve` is true only when the
-- refusal is caused by a version newer than this decoder supports (outer
-- envelope, inner manifest/progress); callers must keep those bytes. Malformed
-- or otherwise corrupt data returns preserve == false so it is not treated as
-- an unsupported future record. Restore/codec exceptions are contained here, so
-- a direct caller never sees a raw error for nested malformed data.
function Checkpoint.decode(text, core, codec, opts)
  if type(text) ~= 'string' then return nil, 'invalid checkpoint text' end
  if #text > Checkpoint.max_body + 128 then return nil, 'checkpoint too large' end
  local tag = text:match('^(TBD%d+) ')
  if not tag then return nil, 'missing checkpoint header' end
  if tag ~= Checkpoint.tag then
    local version = tonumber(tag:sub(4))
    -- Older envelopes (TBD1/TBD2) are legitimate legacy input for the caller;
    -- only a newer-than-supported envelope must be preserved.
    local future = version ~= nil and version > Checkpoint.version
    return nil, 'unsupported checkpoint version ' .. tag:sub(4) .. '; preserved', future
  end
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
  local profile_ok, profile = pcall(core.restore, body:sub(1, plen))
  if not profile_ok or type(profile) ~= 'table' or profile.type ~= 'profile' then return nil, 'invalid profile section' end
  local run = nil
  if rlen > 0 then
    local run_ok, decoded_run = pcall(core.restore, body:sub(plen + 1, plen + rlen))
    if not run_ok or type(decoded_run) ~= 'table' or decoded_run.type ~= 'run' or decoded_run.owner ~= profile.id then
      return nil, 'invalid run section'
    end
    run = decoded_run
  end
  opts = opts or {}
  local manifest = nil
  if mlen > 0 then
    local ok, decoded = pcall(codec.decode, body:sub(plen + rlen + 1, plen + rlen + mlen))
    if not ok or type(decoded) ~= 'table' then return nil, 'invalid manifest section' end
    -- v1 manifests predate schema_version; their `version` is the schema.
    local schema = decoded.schema_version
    if schema == nil then schema = decoded.version end
    local accepted = true
    if opts.manifest_schemas ~= nil then accepted = opts.manifest_schemas[schema] == true
    elseif opts.manifest_schema ~= nil then accepted = schema == opts.manifest_schema end
    if not accepted then
      return nil, string.format('unsupported manifest schema %s; preserved', tostring(schema)), true
    end
    manifest = decoded
  end
  local roster = xlen > 0 and body:sub(plen + rlen + mlen + 1, plen + rlen + mlen + xlen) or nil
  local progress = nil
  if glen > 0 then
    local ok, decoded = pcall(codec.decode, body:sub(plen + rlen + mlen + xlen + 1))
    if not ok or type(decoded) ~= 'table' then return nil, 'invalid progress section' end
    local accepted = true
    if opts.progress_versions ~= nil then accepted = opts.progress_versions[decoded.version] == true
    elseif opts.progress_version ~= nil then accepted = decoded.version == opts.progress_version end
    if not accepted then
      return nil, string.format('unsupported progress version %s; preserved', tostring(decoded.version)), true
    end
    progress = decoded
  end
  return {generation = generation, profile = profile, run = run, manifest = manifest,
    roster = roster, progress = progress}
end

return Checkpoint
