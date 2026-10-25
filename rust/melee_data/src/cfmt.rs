//! printf `%.Ng` as the Windows UCRT prints it (the C build's trace files and log lines).

/// `%.{prec}g` of a double.
pub fn g(v: f64, prec: usize) -> String {
    let p = if prec == 0 { 1 } else { prec };
    if v.is_nan() {
        let bits = v.to_bits();
        let neg = bits >> 63 != 0;
        let quiet = bits & (1 << 51) != 0;
        let payload = bits & ((1 << 51) - 1);
        // UCRT: "nan", "-nan(ind)" for the x86 default NaN, "nan(snan)" for a signalling one
        return if !quiet {
            format!("{}nan(snan)", if neg { "-" } else { "" })
        } else if neg && payload == 0 {
            "-nan(ind)".to_string()
        } else {
            format!("{}nan", if neg { "-" } else { "" })
        };
    }
    if v.is_infinite() {
        return if v < 0.0 { "-inf".into() } else { "inf".into() };
    }
    let neg = v.is_sign_negative();
    let a = v.abs();
    let e = format!("{:.*e}", p - 1, a);
    let (mant, exp) = e.split_once('e').unwrap();
    let x: i32 = exp.parse().unwrap();
    let body = if x < -4 || x >= p as i32 {
        let m = strip(mant);
        format!("{}e{}{:02}", m, if x < 0 { '-' } else { '+' }, x.abs())
    } else {
        let dec = (p as i32 - 1 - x).max(0) as usize;
        strip(&format!("{:.*}", dec, a))
    };
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

fn strip(s: &str) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::g;
    #[test]
    fn basics() {
        assert_eq!(g(0.0, 17), "0");
        assert_eq!(g(-0.0, 17), "-0");
        assert_eq!(g(1.0, 17), "1");
        assert_eq!(g(0.1, 17), "0.10000000000000001");
        assert_eq!(g(1e-5, 9), "1e-05");
        assert_eq!(g(1e20, 17), "1e+20");
        assert_eq!(g(123456789012345678.0, 17), "1.2345678901234568e+17");
        assert_eq!(g(100.0, 9), "100");
    }
}
