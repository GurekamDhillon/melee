
fn bswap32(v: u32, le: bool) -> u32 {
  if (le) {
    return v;
  }
  return ((v & 0x000000FFu) << 24u) |
         ((v & 0x0000FF00u) << 8u) |
         ((v & 0x00FF0000u) >> 8u) |
         ((v & 0xFF000000u) >> 24u);
}

fn bswap16(v: u32, le: bool) -> u32 {
  return select(((v & 0xFFu) << 8u) | (v >> 8u), v, le);
}

fn load_word(p: ptr<storage, array<u32>>, word_idx: u32) -> u32 {
  // This guard is not expected to handle routine out-of-bounds accesses.
  // It appears to discourage some Adreno drivers/optimizers from storage buffer
  // optimizations that can cause visual artifacts, including vertex explosions
  // in Dusklight.
  if (word_idx < arrayLength(p)) {
    return p[word_idx];
  }
  return 0u;
}

fn load_u8(p: ptr<storage, array<u32>>, byte_off: u32) -> u32 {
  let word = load_word(p, byte_off / 4u);
  let shift = (byte_off & 3u) * 8u;
  return (word >> shift) & 0xFFu;
}

fn load_u32_raw(p: ptr<storage, array<u32>>, byte_off: u32) -> u32 {
  let word_idx = byte_off >> 2u;
  let sub = byte_off & 3u;
  let lo = load_word(p, word_idx);
  if (sub == 0u) {
    return lo;
  }
  let hi = load_word(p, word_idx + 1u);
  let shift = sub * 8u;
  return (lo >> shift) | (hi << (32u - shift));
}

fn load_u16(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> u32 {
  let word_idx = byte_off >> 2u;
  let sub = byte_off & 3u;
  let word = load_word(p, word_idx);
  if (sub <= 2u) {
    return bswap16(extractBits(word, sub * 8u, 16u), le);
  }
  let next = load_word(p, word_idx + 1u);
  let raw = extractBits(word, 24u, 8u) | (extractBits(next, 0u, 8u) << 8u);
  return bswap16(raw, le);
}

fn load_u24(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> u32 {
  let raw = load_u32_raw(p, byte_off) & 0x00FFFFFFu;
  if (le) {
    return raw;
  }
  return ((raw & 0x0000FFu) << 16u) |
         (raw & 0x00FF00u) |
         ((raw & 0xFF0000u) >> 16u);
}

fn load_u32(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> u32 {
  return bswap32(load_u32_raw(p, byte_off), le);
}

fn load_f32(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> f32 {
  return bitcast<f32>(load_u32(p, byte_off, le));
}

fn raw_fetch_u8_1(p: ptr<storage, array<u32>>, byte_off: u32) -> u32 {
  return load_u8(p, byte_off);
}

fn raw_fetch_u8_2(p: ptr<storage, array<u32>>, byte_off: u32) -> vec2u {
  let word_idx = byte_off >> 2u;
  let sub = byte_off & 3u;
  let word = load_word(p, word_idx);
  if (sub <= 2u) {
    let shift = sub * 8u;
    return vec2u(
      extractBits(word, shift + 0u, 8u),
      extractBits(word, shift + 8u, 8u),
    );
  }
  let next = load_word(p, word_idx + 1u);
  return vec2u(
    extractBits(word, 24u, 8u),
    extractBits(next, 0u, 8u),
  );
}

fn raw_fetch_u8_3(p: ptr<storage, array<u32>>, byte_off: u32) -> vec3u {
  let raw = load_u32_raw(p, byte_off);
  return vec3u(
    extractBits(raw, 0u, 8u),
    extractBits(raw, 8u, 8u),
    extractBits(raw, 16u, 8u),
  );
}

fn raw_fetch_u8_4(p: ptr<storage, array<u32>>, byte_off: u32) -> vec4u {
  let raw = load_u32_raw(p, byte_off);
  return vec4u(
    extractBits(raw, 0u, 8u),
    extractBits(raw, 8u, 8u),
    extractBits(raw, 16u, 8u),
    extractBits(raw, 24u, 8u),
  );
}

fn raw_fetch_u16_1(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> u32 {
  return load_u16(p, byte_off, le);
}

fn raw_fetch_u16_2(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec2u {
  return vec2u(
    load_u16(p, byte_off + 0u, le),
    load_u16(p, byte_off + 2u, le),
  );
}

fn raw_fetch_u16_3(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec3u {
  return vec3u(
    load_u16(p, byte_off + 0u, le),
    load_u16(p, byte_off + 2u, le),
    load_u16(p, byte_off + 4u, le),
  );
}

fn raw_fetch_u16_4(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4u {
  return vec4u(
    load_u16(p, byte_off + 0u, le),
    load_u16(p, byte_off + 2u, le),
    load_u16(p, byte_off + 4u, le),
    load_u16(p, byte_off + 6u, le),
  );
}

fn raw_fetch_f32_1(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> f32 {
  return load_f32(p, byte_off, le);
}

fn raw_fetch_f32_2(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec2f {
  return vec2f(
    load_f32(p, byte_off + 0u, le),
    load_f32(p, byte_off + 4u, le),
  );
}

fn raw_fetch_f32_3(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec3f {
  return vec3f(
    load_f32(p, byte_off + 0u, le),
    load_f32(p, byte_off + 4u, le),
    load_f32(p, byte_off + 8u, le),
  );
}

fn raw_fetch_f32_4(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  return vec4f(
    load_f32(p, byte_off + 0u, le),
    load_f32(p, byte_off + 4u, le),
    load_f32(p, byte_off + 8u, le),
    load_f32(p, byte_off + 12u, le),
  );
}

fn fetch_u8_1(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> f32 {
  let v = raw_fetch_u8_1(p, byte_off);
  return f32(v) / f32(1u << frac);
}

fn fetch_s8_1(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> f32 {
  let v = (bitcast<i32>(raw_fetch_u8_1(p, byte_off)) << 24) >> 24;
  return f32(v) / f32(1u << frac);
}

fn fetch_u8_2(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec2f {
  let v = raw_fetch_u8_2(p, byte_off);
  return vec2f(v) / f32(1u << frac);
}

fn fetch_s8_2(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec2f {
  let v = (bitcast<vec2i>(raw_fetch_u8_2(p, byte_off)) << vec2u(24u)) >> vec2u(24u);
  return vec2f(v) / f32(1u << frac);
}

fn fetch_u8_3(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec3f {
  let v = raw_fetch_u8_3(p, byte_off);
  return vec3f(v) / f32(1u << frac);
}

fn fetch_s8_3(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec3f {
  let v = (bitcast<vec3i>(raw_fetch_u8_3(p, byte_off)) << vec3u(24u)) >> vec3u(24u);
  return vec3f(v) / f32(1u << frac);
}

fn fetch_u8_4(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec4f {
  let v = raw_fetch_u8_4(p, byte_off);
  return vec4f(v) / f32(1u << frac);
}

fn fetch_s8_4(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec4f {
  let v = (bitcast<vec4i>(raw_fetch_u8_4(p, byte_off)) << vec4u(24u)) >> vec4u(24u);
  return vec4f(v) / f32(1u << frac);
}

fn fetch_u16_1(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> f32 {
  let v = raw_fetch_u16_1(p, byte_off, le);
  return f32(v) / f32(1u << frac);
}

fn fetch_s16_1(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> f32 {
  let v = bitcast<i32>(raw_fetch_u16_1(p, byte_off, le) << 16u) >> 16;
  return f32(v) / f32(1u << frac);
}

fn fetch_u16_2(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec2f {
  let v = raw_fetch_u16_2(p, byte_off, le);
  return vec2f(v) / f32(1u << frac);
}

fn fetch_s16_2(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec2f {
  let v = (bitcast<vec2i>(raw_fetch_u16_2(p, byte_off, le)) << vec2u(16u)) >> vec2u(16u);
  return vec2f(v) / f32(1u << frac);
}

fn fetch_u16_3(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec3f {
  let v = raw_fetch_u16_3(p, byte_off, le);
  return vec3f(v) / f32(1u << frac);
}

fn fetch_s16_3(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec3f {
  let v = (bitcast<vec3i>(raw_fetch_u16_3(p, byte_off, le)) << vec3u(16u)) >> vec3u(16u);
  return vec3f(v) / f32(1u << frac);
}

fn fetch_u16_4(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec4f {
  let v = raw_fetch_u16_4(p, byte_off, le);
  return vec4f(v) / f32(1u << frac);
}

fn fetch_s16_4(p: ptr<storage, array<u32>>, byte_off: u32, frac: u32, le: bool) -> vec4f {
  let v = (bitcast<vec4i>(raw_fetch_u16_4(p, byte_off, le)) << vec4u(16u)) >> vec4u(16u);
  return vec4f(v) / f32(1u << frac);
}

fn fetch_f32_1(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> f32 {
  return raw_fetch_f32_1(p, byte_off, le);
}

fn fetch_f32_2(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec2f {
  return raw_fetch_f32_2(p, byte_off, le);
}

fn fetch_f32_3(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec3f {
  return raw_fetch_f32_3(p, byte_off, le);
}

fn fetch_f32_4(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  return raw_fetch_f32_4(p, byte_off, le);
}

fn fetch_rgb565(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  let v = load_u16(p, byte_off, le);
  return vec4f(
    f32((v >> 11u) & 0x1Fu) / f32(0x1Fu),
    f32((v >>  5u) & 0x3Fu) / f32(0x3Fu),
    f32((v >>  0u) & 0x1Fu) / f32(0x1Fu),
    1.0,
  );
}

fn fetch_rgb8(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  let v = raw_fetch_u8_3(p, byte_off);
  return vec4f(f32(v.x), f32(v.y), f32(v.z), 255.0) / 255.0;
}

fn fetch_rgbx8(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  let v = raw_fetch_u8_4(p, byte_off);
  return vec4f(f32(v.x), f32(v.y), f32(v.z), 255.0) / 255.0;
}

fn fetch_rgba4(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  let v = load_u16(p, byte_off, le);
  return vec4f(
    f32((v >> 12u) & 0x0Fu) / f32(0x0Fu),
    f32((v >>  8u) & 0x0Fu) / f32(0x0Fu),
    f32((v >>  4u) & 0x0Fu) / f32(0x0Fu),
    f32((v >>  0u) & 0x0Fu) / f32(0x0Fu),
  );
}

fn fetch_rgba6(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  let v = load_u24(p, byte_off, le);
  return vec4f(
    f32((v >> 18u) & 0x3Fu) / f32(0x3Fu),
    f32((v >> 12u) & 0x3Fu) / f32(0x3Fu),
    f32((v >>  6u) & 0x3Fu) / f32(0x3Fu),
    f32((v >>  0u) & 0x3Fu) / f32(0x3Fu),
  );
}

fn fetch_rgba8(p: ptr<storage, array<u32>>, byte_off: u32, le: bool) -> vec4f {
  let v = raw_fetch_u8_4(p, byte_off);
  return vec4f(v) / 255.0;
}

fn tev_overflow_f32(in: f32) -> f32 {
  let byte_space = in * 255.0;
  return (byte_space - floor(byte_space / 256.0) * 256.0) / 255.0;
}

fn tev_overflow_vec3f(in: vec3f) -> vec3f {
  let byte_space = in * 255.0;
  return (byte_space - floor(byte_space / 256.0) * 256.0) / 255.0;
}

fn tev_overflow_vec4f(in: vec4f) -> vec4f {
  let byte_space = in * 255.0;
  return (byte_space - floor(byte_space / 256.0) * 256.0) / 255.0;
}

