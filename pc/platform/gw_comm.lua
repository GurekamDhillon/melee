-- gd.comm: a Corneria-style comm callout (portrait window that slides in, a subtitle, a voice line).
-- Compiled into the script prelude (gw_script_comm.inc is generated from this file by
-- pc/platform/gen_comm_inc.py); the engine calls gd.comm_draw() once per draw pass. Single quotes only.
-- State is UI only (draw-pass ticks, no game memory), so savestates and rewinds cannot desync it.
local gd = ... -- gs_build_base passes gd as the first vararg, like the prelude's `local gd, coroutine = ...`
local Comm = {q = {}, cur = nil}
local SLIDE, MAXQ = 16, 16
local WHO = {
  fox    = {name = 'FOX',    rgb = 0xE8892AFF, ink = 0x2A1606FF},
  falco  = {name = 'FALCO',  rgb = 0x3D7BD6FF, ink = 0x081428FF},
  peppy  = {name = 'PEPPY',  rgb = 0x8A8F7AFF, ink = 0x1E2016FF},
  slippy = {name = 'SLIPPY', rgb = 0x5FAE4AFF, ink = 0x0E2008FF},
  custom = {name = 'INCOMING', rgb = 0x9A9AA8FF, ink = 0x14141AFF},
}

function gd.comm(o)
  if type(o) ~= 'table' then error('gd.comm{who=, text=, seconds=, sound=, portrait=, side=}', 2) end
  if gd.match().netplay then error('gd.comm is offline only', 2) end
  local who = o.who or 'custom'
  if not WHO[who] then error('gd.comm: who is fox, falco, peppy, slippy or custom', 2) end
  if #Comm.q >= MAXQ then return false end
  Comm.q[#Comm.q + 1] = {
    who = who, name = o.name or WHO[who].name, text = tostring(o.text or ''),
    frames = math.max(1, math.floor((o.seconds or 3) * 60)),
    sound = o.sound, portrait = o.portrait, side = o.side == 'right' and 'right' or 'left',
  }
  return #Comm.q + (Comm.cur and 1 or 0)
end

function gd.comm_clear()
  Comm.q, Comm.cur = {}, nil
end

-- {shown, queued, who, text, t}: what the window is doing (tests, mods)
function gd.comm_state()
  local c = Comm.cur
  return {shown = c ~= nil, queued = #Comm.q, who = c and c.who, text = c and c.text, t = c and c.t or 0,
          slide = c and c.slide or 0}
end

local function ease(u) u = math.max(0, math.min(1, u)); return 1 - (1 - u) * (1 - u) end

local function safe_rect()
  local r = gd.safe_area and gd.safe_area() -- the widescreen lane's API, when it exists
  if type(r) == 'table' and r.x0 then return r end
  return {x0 = 0, y0 = 0, x1 = 640, y1 = 480}
end

function gd.comm_draw()
  local c = Comm.cur
  if not c then
    c = table.remove(Comm.q, 1)
    if not c then return end
    c.t, Comm.cur = 0, c
    if c.sound then pcall(gd.play_sound, c.sound) end
  end
  c.t = c.t + 1
  local total = SLIDE + c.frames + SLIDE
  if c.t > total then Comm.cur = nil return end
  local s
  if c.t <= SLIDE then s = ease(c.t / SLIDE)
  elseif c.t > SLIDE + c.frames then s = ease((total - c.t) / SLIDE)
  else s = 1 end
  c.slide = s
  local W, H = 330, 92
  local sa = safe_rect()
  local y = sa.y1 - H - 74
  local x_on = c.side == 'right' and (sa.x1 - W - 16) or (sa.x0 + 16)
  local x_off = c.side == 'right' and (sa.x1 + 8) or (sa.x0 - W - 8)
  local x = x_off + (x_on - x_off) * s
  local kit = gd.kit.available() and true or false
  local w = WHO[c.who]
  if kit then gd.kit.panel(x, y, W, H, {piece = 20}) else gd.fill(x, y, W, H, 0x101018E0) gd.box(x, y, W, H, 0xE8E0C8FF) end
  -- portrait: the named kit image, else a colour block with the speaker's initial
  local px, py, ps = x + 10, y + 10, 72
  local drew = false
  if c.portrait and kit then drew = pcall(gd.kit.image, c.portrait, px, py, ps, ps) end
  if not drew then
    gd.fill(px, py, ps, ps, w.rgb)
    local fl = 6 + (c.t % 6 < 3 and 0 or 2) -- static bands: the transmission flicker
    for i = 0, ps - 1, 12 do gd.fill(px, py + ((i + c.t * 2) % ps), ps, 2, 0xFFFFFF30) end
    gd.fill(px, py, ps, fl, 0x00000040)
    gd.text(px + ps / 2 - 10, py + ps / 2 - 14, string.sub(c.name, 1, 1), w.ink, 3)
  end
  -- a blinking REC dot and the speaker name
  if c.t % 30 < 20 then gd.fill(x + W - 22, y + 12, 8, 8, 0xE04040FF) end
  local tx = px + ps + 12
  if kit and pcall(function()
    gd.kit.text(tx, y + 26, c.name, 'label', 'gold')
    gd.kit.text(tx, y + 50, c.text, 'body', 'bone', 'left', {max_w = W - (tx - x) - 12})
  end) then
  else
    gd.text(tx, y + 12, c.name, 0xF0C040FF, 1)
    gd.text(tx, y + 34, c.text, 0xF0E8D0FF, 1)
  end
end
