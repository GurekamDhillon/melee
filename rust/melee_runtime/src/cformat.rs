//! printf for the subset of conversions the game's formats use (`%d %i %u %x %X %o %c %s %f %e %g %%`
//! with flags, width and precision), over the variadic arguments given as a slice. Moved here from
//! `melee_world::tu::textlib` (DevText_Printf) so that `OSReport` (`tu::os_error`) shares it.
#![allow(clippy::missing_safety_doc)]

/// One variadic argument of `DevText_Printf`.
#[derive(Clone, Copy, Debug)]
pub enum PrintfArg {
    /// `int` / `unsigned` (also every promoted narrow integer)
    Int(i32),
    /// `double` (a promoted `float`)
    Dbl(f64),
    /// `char*` (NUL-terminated or NULL)
    Str(*const i8),
}

/// printf for the subset the game's formats use; returns the formatted bytes.
pub unsafe fn c_format(fmt: *const i8, args: &[PrintfArg]) -> std::vec::Vec<u8> {
    let mut out = std::vec::Vec::new();
    let mut p = fmt as *const u8;
    let mut ai = 0usize;
    while *p != 0 {
        if *p != b'%' {
            out.push(*p);
            p = p.add(1);
            continue;
        }
        p = p.add(1);
        let (mut left, mut plus, mut space, mut alt, mut zero) = (false, false, false, false, false);
        loop {
            match *p {
                b'-' => left = true,
                b'+' => plus = true,
                b' ' => space = true,
                b'#' => alt = true,
                b'0' => zero = true,
                _ => break,
            }
            p = p.add(1);
        }
        let mut width = 0usize;
        while (*p as char).is_ascii_digit() {
            width = width * 10 + (*p - b'0') as usize;
            p = p.add(1);
        }
        let mut prec: Option<usize> = None;
        if *p == b'.' {
            p = p.add(1);
            let mut n = 0usize;
            while (*p as char).is_ascii_digit() {
                n = n * 10 + (*p - b'0') as usize;
                p = p.add(1);
            }
            prec = Some(n);
        }
        while matches!(*p, b'h' | b'l' | b'L' | b'z' | b't' | b'j') {
            p = p.add(1);
        }
        let conv = *p;
        if conv == 0 {
            break;
        }
        p = p.add(1);
        let mut body: std::vec::Vec<u8>;
        let mut sign = String::new();
        let mut numeric = false;
        match conv {
            b'%' => body = b"%".to_vec(),
            b'c' => {
                body = std::vec![match args.get(ai) { Some(PrintfArg::Int(v)) => *v as u8, _ => 0 }];
                ai += 1;
            }
            b's' => {
                let s = match args.get(ai) { Some(PrintfArg::Str(s)) => *s, _ => core::ptr::null() };
                ai += 1;
                body = std::vec::Vec::new();
                if s.is_null() {
                    body.extend_from_slice(b"(null)");
                } else {
                    let mut q = s as *const u8;
                    while *q != 0 && prec.map_or(true, |n| body.len() < n) {
                        body.push(*q);
                        q = q.add(1);
                    }
                }
            }
            b'd' | b'i' => {
                let v = match args.get(ai) { Some(PrintfArg::Int(v)) => *v, _ => 0 };
                ai += 1;
                numeric = true;
                if v < 0 {
                    sign.push('-');
                } else if plus {
                    sign.push('+');
                } else if space {
                    sign.push(' ');
                }
                let mut d = (v as i64).unsigned_abs().to_string().into_bytes();
                if let Some(n) = prec {
                    while d.len() < n {
                        d.insert(0, b'0');
                    }
                    if n == 0 && v == 0 {
                        d.clear();
                    }
                }
                body = d;
            }
            b'u' | b'x' | b'X' | b'o' => {
                let v = match args.get(ai) { Some(PrintfArg::Int(v)) => *v as u32, _ => 0 };
                ai += 1;
                numeric = true;
                let mut d = match conv {
                    b'u' => format!("{}", v),
                    b'x' => format!("{:x}", v),
                    b'X' => format!("{:X}", v),
                    _ => format!("{:o}", v),
                }
                .into_bytes();
                if let Some(n) = prec {
                    while d.len() < n {
                        d.insert(0, b'0');
                    }
                    if n == 0 && v == 0 {
                        d.clear();
                    }
                }
                if alt && v != 0 {
                    match conv {
                        b'x' => sign.push_str("0x"),
                        b'X' => sign.push_str("0X"),
                        b'o' if d.first() != Some(&b'0') => d.insert(0, b'0'),
                        _ => {}
                    }
                }
                body = d;
            }
            b'f' | b'F' => {
                let v = match args.get(ai) { Some(PrintfArg::Dbl(v)) => *v, _ => 0.0 };
                ai += 1;
                numeric = !(v.is_nan() || v.is_infinite());
                if v.is_sign_negative() {
                    sign.push('-');
                } else if plus {
                    sign.push('+');
                } else if space {
                    sign.push(' ');
                }
                body = if v.is_nan() {
                    b"nan".to_vec()
                } else if v.is_infinite() {
                    b"inf".to_vec()
                } else {
                    let mut s = format!("{:.*}", prec.unwrap_or(6), v.abs()).into_bytes();
                    if alt && prec == Some(0) {
                        s.push(b'.');
                    }
                    s
                };
            }
            _ => {
                body = std::vec![b'%', conv];
            }
        }
        let len = sign.len() + body.len();
        let pad = width.saturating_sub(len);
        if left {
            out.extend_from_slice(sign.as_bytes());
            out.extend_from_slice(&body);
            out.extend(core::iter::repeat(b' ').take(pad));
        } else if zero && numeric && !(prec.is_some() && matches!(conv, b'd' | b'i' | b'u' | b'x' | b'X' | b'o')) {
            out.extend_from_slice(sign.as_bytes());
            out.extend(core::iter::repeat(b'0').take(pad));
            out.extend_from_slice(&body);
        } else {
            out.extend(core::iter::repeat(b' ').take(pad));
            out.extend_from_slice(sign.as_bytes());
            out.extend_from_slice(&body);
        }
    }
    out
}
