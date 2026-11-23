-- @name: regression pikachu thunder self-hit
-- @gameplay: true
-- @rollback_safe: false
-- Pikachu's down-B under an open sky, so the bolt comes down on Pikachu (ftPk_SpecialLw_SetState_Unk0 writes the
-- hit flag while the thunder's item pointer is stored beside it), grounded and in the air; then dashes left / right
-- while the thunder ends, so a Turn overlaps ftPk_SpecialLw_SetState_Unk1 (mvshadow sites #1 / #2).
local FRAMES = 1800
local n = 0
local seq = {
  {90, 0, 0, 0},
  {4, 'B', 0, -127}, {150, 0, 0, 0},                              -- grounded down-B, the bolt hits
  {4, 'B', 0, -127}, {40, 0, 0, 0}, {6, 0, -100, 0}, {6, 0, 100, 0}, {6, 0, -100, 0}, {6, 0, 100, 0}, {80, 0, 0, 0},
  {3, 'X', 0, 0}, {10, 0, 0, 0}, {4, 'B', 0, -127}, {150, 0, 0, 0},   -- aerial down-B
  {4, 'B', 0, -127}, {12, 0, 0, 0}, {4, 0, -100, 0}, {4, 0, 100, 0}, {4, 0, -100, 0}, {4, 0, 100, 0}, {4, 0, -100, 0}, {100, 0, 0, 0},
}
local total = 0
for _, p in ipairs(seq) do total = total + p[1] end
local function at(frame)
  frame = frame % total
  for _, p in ipairs(seq) do
    if frame < p[1] then return {buttons = p[2], x = p[3], y = p[4]} end
    frame = frame - p[1]
  end
end
function on_frame_pre()
    local m = gd.match()
    if not (m and m.active) then return end
    n = n + 1
    gd.input(1, at(n), 2)
    if n == FRAMES then
        gd.log("COVERAGE quit at frame " .. n)
        gd.quit()
    end
end
