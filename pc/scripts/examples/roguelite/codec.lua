-- Bounded JSON data codec for the resolved run manifest and checkpoint
-- metadata. Independent of Core's frozen save codec: this one accepts larger
-- arrays and string-key objects and rejects executable content by construction
-- (it never calls load/loadstring/dofile).
--
-- Rules:
--  * values are nil-free; a table is either a contiguous integer-keyed array
--    (1..n) or a string-keyed object, never mixed; empty tables encode as {}.
--  * object numeric keys are limited to 1..max_object_key and canonicalize to
--    their decimal string; distinct Lua keys that would collide (1 and "1") are
--    rejected before output. Contiguous 1..n integer tables encode as arrays and
--    preserve numeric keys on decode; a non-contiguous integer object encodes
--    with string keys, so {[2]='a'} round-trips as {['2']='a'}.
--  * a table mixing contiguous integers and string keys is an object, not an
--    array, and is subject to the collision rule above.
--  * strings <= max_string bytes; only ASCII controls are escaped.
--  * numbers must be finite; the encoder emits %.17g and the decoder enforces a
--    strict decimal grammar (no leading zeros, no bare exponents).
--  * limits: max_depth, max_nodes, max_bytes, max_fields, max_array.
local Codec = {version = 1, max_depth = 24, max_nodes = 200000, max_bytes = 1048576,
  max_string = 512, max_fields = 2048, max_array = 8192, max_object_key = 64}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end

local function quote(s)
  return '"' .. s:gsub('[%z\1-\31\\"]', function(c) return string.format('\\u%04x', c:byte()) end) .. '"'
end

local function classify(t)
  local array = true
  local count = 0
  for k in pairs(t) do
    count = count + 1
    if not integer(k, 1, Codec.max_array) then array = false end
  end
  if count == 0 then return 'object' end
  if array and count == #t then return 'array' end
  return 'object'
end

local function encode(v, depth, nodes)
  assert(depth <= Codec.max_depth, 'too deep')
  nodes[1] = nodes[1] + 1
  assert(nodes[1] <= Codec.max_nodes, 'too many nodes')
  local kind = type(v)
  if kind == 'string' then
    assert(#v <= Codec.max_string, 'string too long')
    return quote(v)
  elseif kind == 'number' then
    assert(finite(v), 'nonfinite number')
    return string.format('%.17g', v)
  elseif kind == 'boolean' then
    return tostring(v)
  elseif kind == 'table' then
    assert(not getmetatable(v), 'metatable')
    if classify(v) == 'array' then
      return '[' .. (function()
        local parts = {}
        for i = 1, #v do parts[#parts + 1] = encode(v[i], depth + 1, nodes) end
        return table.concat(parts, ',')
      end)() .. ']'
    end
    -- Object keys are canonicalized to strings. Two distinct Lua keys that
    -- canonicalize to the same string (number 1 and string "1") would produce a
    -- duplicate JSON key the decoder rejects, so refuse before writing.
    local keys, canonical = {}, {}
    for k in pairs(v) do
      assert(type(k) == 'string' or integer(k, 1, Codec.max_object_key), 'bad object key')
      local name = tostring(k)
      assert(not canonical[name], 'colliding object keys')
      canonical[name] = true
      keys[#keys + 1] = k
    end
    table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
    assert(#keys <= Codec.max_fields, 'too many fields')
    local parts = {}
    for _, k in ipairs(keys) do
      parts[#parts + 1] = quote(tostring(k)) .. ':' .. encode(v[k], depth + 1, nodes)
    end
    return '{' .. table.concat(parts, ',') .. '}'
  end
  error('unsupported value type ' .. kind)
end

local function decode(text)
  assert(type(text) == 'string' and #text <= Codec.max_bytes, 'invalid codec input size')
  local pos, nodes = 1, 0
  local function ws() local _, b = text:find('^%s*', pos); pos = (b or pos - 1) + 1 end
  local function str()
    assert(text:sub(pos, pos) == '"', 'expected string')
    pos = pos + 1
    local out = {}
    while pos <= #text do
      local ch = text:sub(pos, pos)
      pos = pos + 1
      if ch == '"' then
        local s = table.concat(out)
        assert(#s <= Codec.max_string, 'string too long')
        return s
      elseif ch == '\\' then
        local esc = text:sub(pos, pos)
        pos = pos + 1
        if esc == 'u' then
          local hex = text:sub(pos, pos + 3)
          assert(hex:match('^%x%x%x%x$'), 'bad escape')
          local x = tonumber(hex, 16)
          assert(x <= 127, 'unsupported unicode')
          out[#out + 1] = string.char(x)
          pos = pos + 4
        else
          local map = {['"'] = '"', ['\\'] = '\\', ['/'] = '/', b = '\b', f = '\f', n = '\n', r = '\r', t = '\t'}
          assert(map[esc], 'bad escape')
          out[#out + 1] = map[esc]
        end
      else
        assert(ch:byte() >= 32, 'control character')
        out[#out + 1] = ch
      end
    end
    error('unterminated string')
  end
  local parse
  parse = function(depth)
    nodes = nodes + 1
    assert(nodes <= Codec.max_nodes and depth <= Codec.max_depth, 'too complex')
    ws()
    local ch = text:sub(pos, pos)
    if ch == '[' then
      pos = pos + 1
      ws()
      local out = {}
      if text:sub(pos, pos) == ']' then pos = pos + 1; return out end
      local count = 0
      while true do
        count = count + 1
        assert(count <= Codec.max_array, 'array too long')
        out[count] = parse(depth + 1)
        ws()
        local sep = text:sub(pos, pos)
        pos = pos + 1
        if sep == ']' then return out end
        assert(sep == ',', 'expected comma')
        ws()
      end
    elseif ch == '{' then
      pos = pos + 1
      ws()
      local out = {}
      if text:sub(pos, pos) == '}' then pos = pos + 1; return out end
      local count = 0
      while true do
        ws()
        local k = str()
        assert(out[k] == nil, 'duplicate key')
        ws()
        assert(text:sub(pos, pos) == ':', 'expected colon')
        pos = pos + 1
        out[k] = parse(depth + 1)
        count = count + 1
        assert(count <= Codec.max_fields, 'too many fields')
        ws()
        local sep = text:sub(pos, pos)
        pos = pos + 1
        if sep == '}' then return out end
        assert(sep == ',', 'expected comma')
      end
    elseif ch == '"' then
      return str()
    elseif text:sub(pos, pos + 3) == 'true' then
      pos = pos + 4; return true
    elseif text:sub(pos, pos + 4) == 'false' then
      pos = pos + 5; return false
    else
      local token = text:match('^[%-0-9%.eE+]+', pos)
      assert(token, 'expected value')
      local mantissa, exponent = token:match('^(.-)[eE]([+-]?%d+)$')
      mantissa = mantissa or token
      assert(not mantissa:find('[eE+]') and (mantissa:match('^-?%d+$') or mantissa:match('^-?%d+%.%d+$')), 'bad number')
      local unsigned = mantissa:gsub('^-', '')
      assert(not unsigned:match('^0%d'), 'leading zero')
      local x = tonumber(token)
      assert(finite(x), 'bad number')
      pos = pos + #token
      return x
    end
  end
  local value = parse(0)
  ws()
  assert(pos > #text, 'trailing input')
  return value
end

function Codec.encode(v)
  local ok, text = pcall(function()
    local nodes = {0}
    local result = encode(v, 0, nodes)
    assert(#result <= Codec.max_bytes, 'encoded too large')
    return result
  end)
  if ok then return text end
  return nil, tostring(text)
end

function Codec.decode(text)
  local ok, result = pcall(decode, text)
  if ok then return result end
  return nil, tostring(result)
end

return Codec
