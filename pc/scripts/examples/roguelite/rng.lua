-- Named deterministic random streams. Park-Miller minimal standard with the
-- same multiplier/modulus as Core's saved seeds, so a seed recorded for the
-- gene economy reproduces here. Pure: no engine access, no os/io/clock.
--
-- Independent streams are derived from (base seed, stream name). A stream's
-- name is hashed with a small integer polynomial so results do not depend on
-- platform string hashing or table iteration order. Callers must sort any ID
-- list before selection; this module never iterates a table with pairs() to
-- choose an outcome.
local RNG = {version = 1, modulus = 2147483647, multiplier = 48271}
local MOD, MUL = RNG.modulus, RNG.multiplier

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end
local function advance(s) return (s * MUL) % MOD end

function RNG.hash(text)
  assert(type(text) == 'string' and #text > 0 and #text <= 64, 'invalid stream name')
  local h = 7
  for i = 1, #text do h = (h * 131 + text:byte(i)) % MOD end
  return h == 0 and 1 or h
end

-- One named stream for a run. `name` must be a short ASCII identifier.
function RNG.derive(base, name)
  assert(integer(base, 1, MOD - 1), 'invalid base seed')
  assert(type(name) == 'string' and #name > 0 and #name <= 64, 'invalid stream name')
  local s = advance((base + RNG.hash(name)) % MOD)
  return s == 0 and 1 or s
end

function RNG.new(base, name)
  local self = {base = base, name = name, seed = RNG.derive(base, name), count = 0}
  return setmetatable(self, {__index = RNG})
end

-- Raw next value in 1..MOD-1. Exposed for stream continuity checks only.
function RNG:int()
  local s = advance(self.seed)
  self.seed = s
  self.count = self.count + 1
  return s
end

-- Uniform-ish integer in [1, n]. Rejection sampling removes modulo bias while
-- preserving determinism (the discarded draws still advance the stream).
function RNG:below(n)
  assert(integer(n, 1, MOD), 'invalid bound')
  local limit = MOD - (MOD % n)
  local value
  repeat value = self:int() until value <= limit
  return (value - 1) % n + 1
end

function RNG:range(lo, hi)
  assert(integer(lo, -1000000000, 1000000000) and integer(hi, lo, 1000000000), 'invalid range')
  return lo + self:below(hi - lo + 1) - 1
end

function RNG:chance(numerator, denominator)
  assert(integer(numerator, 0, denominator) and integer(denominator, 1, 1000000), 'invalid chance')
  return numerator > 0 and self:below(denominator) <= numerator
end

function RNG:pick(list)
  assert(type(list) == 'table' and #list > 0, 'invalid pick list')
  return list[self:below(#list)]
end

function RNG:shuffle(list)
  assert(type(list) == 'table', 'invalid list')
  local out = {}
  for i = 1, #list do out[i] = list[i] end
  for i = #out, 2, -1 do
    local j = self:below(i)
    out[i], out[j] = out[j], out[i]
  end
  return out
end

-- choices: array of {value=..., weight=positive integer}. A zero total rejects.
function RNG:weighted(choices)
  assert(type(choices) == 'table' and #choices > 0, 'invalid weighted list')
  local total = 0
  for _, choice in ipairs(choices) do
    assert(type(choice) == 'table' and choice.weight ~= nil and integer(choice.weight, 0, 1000000), 'invalid weight')
    total = total + choice.weight
  end
  assert(total > 0, 'zero total weight')
  local roll = self:below(total)
  for _, choice in ipairs(choices) do
    roll = roll - choice.weight
    if roll <= 0 then return choice.value end
  end
  return choices[#choices].value
end

-- Deterministic ascending order of a table's keys, independent of pairs().
function RNG.sorted_keys(t)
  local keys = {}
  for k in pairs(t) do keys[#keys + 1] = k end
  table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
  return keys
end

return RNG
