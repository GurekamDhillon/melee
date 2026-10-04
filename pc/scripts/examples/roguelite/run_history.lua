-- Gate 7 pure, bounded run history.
--
-- A configurable ledger of completed runs. It is deliberately separate from
-- Core's profile.finished map: Core's ledger stores only outcome/export and is
-- validated by the frozen Core codec, while this record carries the richer
-- run summary and serializes through the sandbox-safe Codec (never load()).
-- from_core_finished() is the explicit migration boundary: it reads Core's
-- finished map and produces a versioned history WITHOUT mutating the profile.
--
-- Bounds: max_entries is configurable; every id/string array is capped at
-- max_array (16) with an honest `truncated` flag. Append returns a staged copy
-- and refuses or evicts the oldest entry according to the configured overflow.
local RunHistory = {version = 1, default_max_entries = 512, max_entries = 1024, max_array = 16, max_id = 64}
RunHistory.outcomes = {success = true, failure = true}
RunHistory.overflow_policies = {refuse = true, evict = true}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end
local function valid_id(x) return type(x) == 'string' and #x > 0 and #x <= RunHistory.max_id and not x:find('[%z\1-\31]') end

local function copy(t)
  if type(t) ~= 'table' then return t end
  local out = {}
  for k, v in pairs(t) do out[k] = copy(v) end
  return out
end

local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end

local function sorted_keys(t)
  local out = {}
  for k in pairs(t) do out[#out + 1] = k end
  table.sort(out)
  return out
end

-- Copy a bounded, de-duplicated id array. `truncated` reports that entries were
-- dropped so a summary never silently overstates what the ledger holds.
local function bound_ids(list, truncated)
  local out, seen = {}, {}
  for _, id in ipairs(list or {}) do
    if valid_id(id) and not seen[id] then
      if #out >= RunHistory.max_array then truncated[1] = true break end
      seen[id] = true
      out[#out + 1] = id
    end
  end
  return out
end

local ENTRY_FIELDS = {serial = true, run_id = true, outcome = true, export = true, seed = true,
  stocks = true, frames = true, gene_count = true, currency = true, rewards = true,
  mutations = true, rooms = true, distinct_rewards = true, truncated = true}

function RunHistory.new(owner, opts)
  opts = opts or {}
  if not valid_id(owner) then return nil, 'invalid run history owner' end
  local max_entries = opts.max_entries or RunHistory.default_max_entries
  if not integer(max_entries, 1, RunHistory.max_entries) then return nil, 'invalid max_entries' end
  local overflow = opts.overflow or 'refuse'
  if not RunHistory.overflow_policies[overflow] then return nil, 'invalid overflow policy' end
  return {version = 1, owner = owner, max_entries = max_entries, overflow = overflow,
    next_serial = 1, entries = {}}
end

local function validate_entry(entry)
  if type(entry) ~= 'table' then return false, 'invalid history entry' end
  for field in pairs(entry) do if not ENTRY_FIELDS[field] then return false, 'unknown entry field ' .. tostring(field) end end
  if not integer(entry.serial, 1, 1000000000) then return false, 'invalid entry serial' end
  if not valid_id(entry.run_id) then return false, 'invalid entry run id' end
  if not RunHistory.outcomes[entry.outcome] then return false, 'invalid entry outcome' end
  if entry.export ~= nil and not valid_id(entry.export) then return false, 'invalid entry export' end
  if entry.seed ~= nil and not integer(entry.seed, 1, 2147483646) then return false, 'invalid entry seed' end
  if entry.stocks ~= nil and not integer(entry.stocks, 0, 99) then return false, 'invalid entry stocks' end
  if entry.frames ~= nil and not integer(entry.frames, 0, 1000000000) then return false, 'invalid entry frames' end
  if entry.gene_count ~= nil and not integer(entry.gene_count, 0, 128) then return false, 'invalid entry gene count' end
  if entry.currency ~= nil and not integer(entry.currency, 0, 1000000000) then return false, 'invalid entry currency' end
  for _, field in ipairs({'rewards', 'mutations', 'rooms'}) do
    local list = entry[field]
    if type(list) ~= 'table' or #list > RunHistory.max_array then return false, 'invalid entry ' .. field end
    for _, id in ipairs(list) do if not valid_id(id) then return false, 'invalid id in entry ' .. field end end
  end
  if entry.distinct_rewards ~= nil then
    if not integer(entry.distinct_rewards, 0, RunHistory.max_array) then return false, 'invalid distinct_rewards' end
    if entry.distinct_rewards > #(entry.rewards or {}) then return false, 'distinct_rewards exceeds rewards' end
  end
  if entry.truncated ~= nil and type(entry.truncated) ~= 'boolean' then return false, 'invalid truncated flag' end
  return true
end

-- Build a bounded entry from a live run and a summary table. Fields absent from
-- the summary/run are simply omitted (never fabricated).
function RunHistory.make_entry(run, summary)
  summary = summary or {}
  if type(run) ~= 'table' or not valid_id(run.id) then return nil, 'invalid run' end
  if not RunHistory.outcomes[summary.outcome] then return nil, 'invalid outcome' end
  local entry = {run_id = run.id, outcome = summary.outcome}
  if integer(run.seed, 1, 2147483646) then entry.seed = run.seed end
  if integer(run.frame, 0, 1000000000) then entry.frames = run.frame end
  if integer(run.stocks, 0, 99) then entry.stocks = run.stocks end
  if integer(summary.currency, 0, 1000000000) then entry.currency = summary.currency end
  if type(run.genes) == 'table' then entry.gene_count = count(run.genes) end
  if valid_id(summary.export) then entry.export = summary.export end
  local truncated = {false}
  entry.rewards = bound_ids(summary.rewards, truncated)
  entry.mutations = bound_ids(summary.mutations, truncated)
  entry.rooms = bound_ids(summary.rooms, truncated)
  entry.distinct_rewards = #entry.rewards
  if truncated[1] then entry.truncated = true end
  return entry
end

function RunHistory.append(history, entry)
  local ok, why = RunHistory.validate(history)
  if not ok then return nil, why end
  if type(entry) ~= 'table' then return nil, 'invalid history entry' end
  local staged = copy(history)
  if #staged.entries >= staged.max_entries then
    if staged.overflow == 'evict' then
      table.remove(staged.entries, 1)
    else
      return nil, 'run history full'
    end
  end
  local stored = copy(entry)
  stored.serial = staged.next_serial
  local entry_ok, entry_why = validate_entry(stored)
  if not entry_ok then return nil, entry_why end
  staged.next_serial = staged.next_serial + 1
  staged.entries[#staged.entries + 1] = stored
  return staged
end

local HISTORY_FIELDS = {version = true, owner = true, max_entries = true, overflow = true,
  next_serial = true, entries = true}

function RunHistory.validate(history)
  if type(history) ~= 'table' or history.version ~= 1 then return false, 'unsupported run history version' end
  for field in pairs(history) do if not HISTORY_FIELDS[field] then return false, 'unknown history field ' .. tostring(field) end end
  if not valid_id(history.owner) then return false, 'invalid history owner' end
  if not integer(history.max_entries, 1, RunHistory.max_entries) then return false, 'invalid max_entries' end
  if not RunHistory.overflow_policies[history.overflow] then return false, 'invalid overflow policy' end
  if not integer(history.next_serial, 1, 1000000000) then return false, 'invalid next_serial' end
  if type(history.entries) ~= 'table' or #history.entries > history.max_entries then
    return false, 'history entries exceed bound'
  end
  local last = 0
  for _, entry in ipairs(history.entries) do
    local ok, why = validate_entry(entry)
    if not ok then return false, why end
    if entry.serial <= last then return false, 'history serials not increasing' end
    if entry.serial >= history.next_serial then return false, 'history serial past next_serial' end
    last = entry.serial
  end
  return true
end

function RunHistory.summary(history)
  local out = {total = 0, successes = 0, failures = 0, currency = 0, rewards = {}}
  for _, entry in ipairs(history.entries or {}) do
    out.total = out.total + 1
    if entry.outcome == 'success' then out.successes = out.successes + 1 else out.failures = out.failures + 1 end
    out.currency = out.currency + (entry.currency or 0)
    for _, id in ipairs(entry.rewards or {}) do out.rewards[id] = true end
  end
  out.distinct_rewards = count(out.rewards)
  return out
end

function RunHistory.encode(history, codec)
  if not (type(codec) == 'table' and type(codec.encode) == 'function') then return nil, 'Codec required' end
  local ok, why = RunHistory.validate(history)
  if not ok then return nil, why end
  return codec.encode(copy(history))
end

function RunHistory.decode(text, codec)
  if not (type(codec) == 'table' and type(codec.decode) == 'function') then return nil, 'Codec required' end
  local value, why = codec.decode(text)
  if not value then return nil, why end
  local ok, ok_why = RunHistory.validate(value)
  if not ok then return nil, ok_why end
  return value
end

-- Migration boundary from Core's frozen `profile.finished` ledger. Core stores
-- only {outcome, export}; runs are read in run-serial order and mapped without
-- inventing seed/frames/currency. The profile is never modified.
function RunHistory.from_core_finished(owner, profile, opts)
  if type(profile) ~= 'table' or profile.type ~= 'profile' then return nil, 'invalid profile' end
  if type(profile.finished) ~= 'table' then return nil, 'profile has no finish ledger' end
  local history, why = RunHistory.new(owner, opts)
  if not history then return nil, why end
  local ids = sorted_keys(profile.finished)
  for _, run_id in ipairs(ids) do
    local result = profile.finished[run_id]
    -- A discarded earned gene is absent from collection but remains part of
    -- the historical run outcome. Core records that identity separately.
    local entry = RunHistory.make_entry({id = run_id},
      {outcome = result.outcome, export = result.export or result.discarded_export})
    if not entry then return nil, 'invalid legacy finish record ' .. run_id end
    local next_history, append_why = RunHistory.append(history, entry)
    if not next_history then return nil, append_why end
    history = next_history
  end
  return history
end

return RunHistory
