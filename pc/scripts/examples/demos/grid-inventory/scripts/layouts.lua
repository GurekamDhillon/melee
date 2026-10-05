-- The demo's fake data and its three layouts (BAG, REWARD, FULL SLOTS), shaped like the roguelike layer's drives:
-- a family colour, a rarity, an affix count (pips), flags, and a few plain stat lines. Pure Lua: no gd calls.
-- Source of truth for the embedded copy in main.lua (scripts/embed.py).
local L = {}

L.names = { 'bag4', 'bag5', 'bag6', 'reward2', 'reward3', 'swap' }

local FAMILY = { red = 'Damage dealt', blue = 'Launch dealt', green = 'Speed', yellow = 'Damage taken', white = 'Wild' }
local RARITY_WORD = { common = 'Common', uncommon = 'Uncommon', rare = 'Rare', unique = 'Unique' }

local function st(key, label, value, text, better) return { key = key, label = label, value = value, text = text, better = better } end

-- a drive: stats = { st(...), ... }; the plain lines are derived from them once, here
local function drive(name, colour, rarity, pips, flags, stats, icon)
  local lines = { RARITY_WORD[rarity] .. ' ' .. FAMILY[colour] .. ' drive, ' .. pips .. (pips == 1 and ' affix' or ' affixes') }
  for _, s in ipairs(stats) do lines[#lines + 1] = s.label .. '  ' .. s.text end
  return { name = name, colour = colour, rarity = rarity, pips = pips, flags = flags, stats = stats, lines = lines, icon = icon }
end

local function pool()
  return {
    drive('Ember Core', 'red', 'rare', 3, { 'equipped' }, { st('dmg', 'Damage dealt', 18, '+18%'), st('hit', 'Damage taken', 6, '+6%', 'low') }, { kind = 'model', asset = 'ember_core', spin = true }),
    drive('Gale Coil', 'green', 'uncommon', 2, { 'equipped' }, { st('spd', 'Speed', 12, '+12%') }),
    drive('Tide Lens', 'blue', 'common', 1, { 'equipped', 'merge' }, { st('lau', 'Launch dealt', 8, '+8%') }),
    drive('Sun Plate', 'yellow', 'rare', 2, { 'equipped' }, { st('hit', 'Damage taken', -14, '-14%', 'low'), st('spd', 'Speed', -4, '-4%') }),
    drive('Prism Seed', 'white', 'unique', 4, { 'equipped', 'locked' }, { st('dmg', 'Damage dealt', 9, '+9%'), st('spd', 'Speed', 9, '+9%'), st('lau', 'Launch dealt', 9, '+9%') }),
    drive('Rust Bolt', 'red', 'common', 1, { 'equipped' }, { st('dmg', 'Damage dealt', 7, '+7%') }),
  }
end

local function bag_items()
  return {
    drive('Ash Fang', 'red', 'uncommon', 2, { 'new' }, { st('dmg', 'Damage dealt', 14, '+14%'), st('lau', 'Launch dealt', 3, '+3%') }),
    drive('Moss Reed', 'green', 'common', 1, nil, { st('spd', 'Speed', 5, '+5%') }),
    drive('Deep Lens', 'blue', 'rare', 3, { 'merge' }, { st('lau', 'Launch dealt', 21, '+21%'), st('hit', 'Damage taken', 5, '+5%', 'low') }),
    drive('Tide Lens', 'blue', 'common', 1, { 'merge' }, { st('lau', 'Launch dealt', 8, '+8%') }),
  }
end

local function keystones()
  local function key(name, line1, line2)
    return { name = name, colour = 'gold', rarity = 'unique', pips = 0, icon = 'crown',
             lines = { 'Keystone: one strong rule with a drawback', line1, line2 }, actions = { A = 'Toggle', B = 'Close' } }
  end
  return { key('Glass Cannon', 'Damage dealt  +40%', 'Damage taken  +30%'), key('Featherweight', 'Speed  +25%', 'Launch dealt  -15%') }
end

-- equipped block: `slots` existing slots; the first `filled` hold a drive, the rest are empty slots; the cell after
-- the last slot is nil (no such slot), which is what makes a 5-slot 3x2 block an uneven block
local function equipped_cells(slots, filled, actions)
  local p, cells = pool(), {}
  for i = 1, slots do
    local c = p[i]
    if i <= filled and c then c.actions = actions
    else c = { empty = true, name = 'Empty slot', lines = { 'Pick a bag drive and press A' }, actions = { B = 'Close' } } end
    cells[i] = c
  end
  return cells
end

-- Returns { title, blocks, actions, hint, focus = {block, index}, compare = {block, index, as}, countdown = {s, total} }
function L.spec(name)
  local spec = { actions = { B = 'Close' }, hint = 'L: next layout' }
  if name == 'bag4' or name == 'bag5' or name == 'bag6' then
    local slots = tonumber(name:sub(4))
    local filled = slots == 6 and 5 or 4
    local bag = bag_items()
    for i, c in ipairs(bag) do
      c.actions = { A = 'Equip', Y = 'Discard', B = 'Close' }
      if c.flags and c.flags[1] == 'merge' then c.actions.X = 'Merge' end
    end
    spec.title = 'BAG'
    spec.blocks = {
      { id = 'eq', title = ('EQUIPPED %d/%d'):format(filled, slots), cols = slots == 4 and 2 or 3, rows = 2, band = 1,
        cells = equipped_cells(slots, filled, { A = 'To bag', B = 'Close' }) },
      { id = 'bag', title = 'BAG 4/12', cols = 2, rows = 2, band = 1, cells = bag },
      { id = 'key', title = 'KEYSTONES 2/3', cols = 3, rows = 1, band = 2, cells = keystones() },
    }
    spec.focus = { 'eq', 1 }
  elseif name == 'reward2' or name == 'reward3' then
    local n = tonumber(name:sub(7))
    local offered = {
      drive('Cinder Edge', 'red', 'rare', 3, { 'new' }, { st('dmg', 'Damage dealt', 26, '+26%'), st('hit', 'Damage taken', 8, '+8%', 'low') }),
      drive('Zephyr Wire', 'green', 'uncommon', 2, { 'new' }, { st('spd', 'Speed', 16, '+16%') }),
      drive('Bulwark', 'yellow', 'unique', 4, { 'new' }, { st('hit', 'Damage taken', -22, '-22%', 'low'), st('spd', 'Speed', -2, '-2%') }),
    }
    local cells = {}
    for i = 1, n do offered[i].actions = { A = 'Take + equip', X = 'Take to bag', B = 'Skip' }; cells[i] = offered[i] end
    spec.title = 'STAGE CLEAR'
    spec.blocks = {
      { id = 'offer', title = ('TAKE ONE OF %d'):format(n), cols = n, rows = 1, band = 1, cells = cells },
      { id = 'eq', title = 'EQUIPPED 4/6', cols = 3, rows = 2, band = 2, focusable = false, cells = equipped_cells(6, 4, {}) },
    }
    spec.focus = { 'offer', 1 }
    spec.compare = { 'eq', 1, 'before' }             -- the drive that would be replaced
    spec.countdown = { 30, 30 }
    spec.actions = { B = 'Skip' }
  elseif name == 'swap' then
    local incoming = drive('Cinder Edge', 'red', 'rare', 3, { 'new' }, { st('dmg', 'Damage dealt', 26, '+26%'), st('hit', 'Damage taken', 8, '+8%', 'low') })
    incoming.actions = {}; incoming.nocompare = true
    local keep = { target = true, nocompare = true, name = 'Keep in bag', colour = 'grey', lines = { 'The new drive goes to the bag.', 'Nothing is swapped out.' },
                   actions = { A = 'Keep in bag', B = 'Back' } }
    spec.title = 'SLOTS FULL'
    spec.blocks = {
      { id = 'in', title = 'NEW DRIVE', cols = 1, rows = 1, band = 1, focusable = false, cells = { incoming } },
      { id = 'keep', title = 'OR KEEP', cols = 1, rows = 1, band = 1, cells = { keep } },
      { id = 'eq', title = 'SWAP OUT WHICH?', cols = 3, rows = 2, band = 2, cells = equipped_cells(6, 6, { A = 'Swap out', B = 'Back' }) },
    }
    spec.focus = { 'eq', 1 }
    spec.compare = { 'in', 1, 'after' }              -- focus = what you have, partner = what comes in
    spec.actions = { B = 'Back' }
    spec.countdown = { 15, 15 }
  else error('unknown layout ' .. tostring(name)) end
  return spec
end

-- Apply a layout to a view: blocks, focus, compare, countdown, actions, hint. Returns the spec.
function L.apply(view, name)
  local s = L.spec(name)
  view:set_compare(nil)
  view:set_title(s.title)
  view:set_actions(s.actions)
  view:set_hint(s.hint)
  view.fe = nil
  view:set_blocks(s.blocks)
  if s.focus then view:set_focus(s.focus[1], s.focus[2]) end
  if s.compare then view:set_compare(s.compare[1], s.compare[2], s.compare[3]) end
  view:set_countdown(s.countdown and s.countdown[1] or nil, s.countdown and s.countdown[2] or nil)
  return s
end

return L
