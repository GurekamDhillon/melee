//! `luaL_check*` / `luaL_opt*` for native `gd.*` functions, with lauxlib's coercions and messages
//! (`bad argument #1 to 'player' (number expected, got nil)`), so a script that errors in the C build
//! errors the same way here.
use mlua::{Error, MultiValue, Result, Table, Value};

pub struct Args {
    name: &'static str,
    v: Vec<Value>,
}

fn typename(v: Option<&Value>) -> &'static str {
    match v {
        None => "no value",
        Some(Value::Nil) => "nil",
        Some(Value::Boolean(_)) => "boolean",
        Some(Value::Integer(_)) | Some(Value::Number(_)) => "number",
        Some(Value::String(_)) => "string",
        Some(Value::Table(_)) => "table",
        Some(Value::Function(_)) => "function",
        Some(Value::Thread(_)) => "thread",
        _ => "userdata",
    }
}

/// `lua_stringtonumber` for the checkers: decimal/hex integers and floats, surrounding whitespace allowed.
pub fn str_to_number(s: &str) -> Option<Value> {
    let t = s.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c'));
    if t.is_empty() {
        return None;
    }
    let (neg, body) = match t.as_bytes()[0] {
        b'-' => (true, &t[1..]),
        b'+' => (false, &t[1..]),
        _ => (false, t),
    };
    if let Some(h) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
        if !h.is_empty() && h.bytes().all(|c| c.is_ascii_hexdigit()) {
            let mut v: u64 = 0;
            for c in h.bytes() {
                v = v.wrapping_mul(16).wrapping_add((c as char).to_digit(16).unwrap() as u64);
            }
            let v = v as i64;
            return Some(Value::Integer(if neg { v.wrapping_neg() } else { v }));
        }
        return None; // hex floats are not accepted by the checkers here
    }
    if !body.is_empty() && body.bytes().all(|c| c.is_ascii_digit()) {
        if let Ok(v) = body.parse::<i64>() {
            return Some(Value::Integer(if neg { -v } else { v }));
        }
    }
    if body.bytes().any(|c| c.is_ascii_alphabetic() && c != b'e' && c != b'E') {
        return None;
    }
    t.parse::<f64>().ok().map(Value::Number)
}

impl Args {
    pub fn new(name: &'static str, args: MultiValue) -> Args {
        Args { name, v: args.into_iter().collect() }
    }
    pub fn get_all(&self) -> Vec<Value> {
        self.v.clone()
    }
    pub fn len(&self) -> usize {
        self.v.len()
    }
    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }
    pub fn err(&self, i: usize, what: &str) -> Error {
        Error::runtime(format!("bad argument #{} to '{}' ({})", i, self.name, what))
    }
    fn type_err(&self, i: usize, expected: &str) -> Error {
        self.err(i, &format!("{} expected, got {}", expected, typename(self.v.get(i - 1))))
    }
    pub fn get(&self, i: usize) -> &Value {
        self.v.get(i - 1).unwrap_or(&Value::Nil)
    }
    pub fn none_or_nil(&self, i: usize) -> bool {
        matches!(self.v.get(i - 1), None | Some(Value::Nil))
    }
    pub fn is_none(&self, i: usize) -> bool {
        self.v.get(i - 1).is_none()
    }
    /// `lua_toboolean`
    pub fn truthy(&self, i: usize) -> bool {
        !matches!(self.v.get(i - 1), None | Some(Value::Nil) | Some(Value::Boolean(false)))
    }
    pub fn is_table(&self, i: usize) -> bool {
        matches!(self.v.get(i - 1), Some(Value::Table(_)))
    }
    pub fn is_function(&self, i: usize) -> bool {
        matches!(self.v.get(i - 1), Some(Value::Function(_)))
    }
    pub fn is_integer(&self, i: usize) -> bool {
        matches!(self.v.get(i - 1), Some(Value::Integer(_)))
    }
    pub fn is_number(&self, i: usize) -> bool {
        match self.v.get(i - 1) {
            Some(Value::Integer(_)) | Some(Value::Number(_)) => true,
            Some(Value::String(s)) => s.to_str().is_ok_and(|s| str_to_number(&s).is_some()),
            _ => false,
        }
    }
    pub fn is_string(&self, i: usize) -> bool {
        matches!(self.v.get(i - 1), Some(Value::String(_)) | Some(Value::Integer(_)) | Some(Value::Number(_)))
    }
    fn to_number(&self, i: usize) -> Option<Value> {
        match self.v.get(i - 1)? {
            v @ (Value::Integer(_) | Value::Number(_)) => Some(v.clone()),
            Value::String(s) => str_to_number(&s.to_str().ok()?),
            _ => None,
        }
    }
    pub fn check_number(&self, i: usize) -> Result<f64> {
        match self.to_number(i) {
            Some(Value::Integer(n)) => Ok(n as f64),
            Some(Value::Number(n)) => Ok(n),
            _ => Err(self.type_err(i, "number")),
        }
    }
    pub fn opt_number(&self, i: usize, d: f64) -> Result<f64> {
        if self.none_or_nil(i) { Ok(d) } else { self.check_number(i) }
    }
    pub fn check_integer(&self, i: usize) -> Result<i64> {
        match self.to_number(i) {
            Some(Value::Integer(n)) => Ok(n),
            Some(Value::Number(f)) => {
                if f.floor() == f && f >= -9.223372036854775808e18 && f < 9.223372036854775808e18 {
                    Ok(f as i64)
                } else {
                    Err(self.err(i, "number has no integer representation"))
                }
            }
            _ => Err(self.type_err(i, "number")),
        }
    }
    pub fn opt_integer(&self, i: usize, d: i64) -> Result<i64> {
        if self.none_or_nil(i) { Ok(d) } else { self.check_integer(i) }
    }
    /// `luaL_checkstring`: strings and numbers (numbers converted as `tostring` would).
    pub fn check_string(&self, i: usize) -> Result<String> {
        match self.v.get(i - 1) {
            Some(Value::String(s)) => Ok(s.to_string_lossy().to_string()),
            Some(Value::Integer(n)) => Ok(n.to_string()),
            Some(Value::Number(n)) => Ok(fmt_number(*n)),
            _ => Err(self.type_err(i, "string")),
        }
    }
    /// bytes of a string argument (data_write bodies may hold any byte)
    pub fn check_bytes(&self, i: usize) -> Result<Vec<u8>> {
        match self.v.get(i - 1) {
            Some(Value::String(s)) => Ok(s.as_bytes().to_vec()),
            Some(Value::Integer(_)) | Some(Value::Number(_)) => Ok(self.check_string(i)?.into_bytes()),
            _ => Err(self.type_err(i, "string")),
        }
    }
    pub fn opt_string(&self, i: usize, d: &str) -> Result<String> {
        if self.none_or_nil(i) { Ok(d.to_string()) } else { self.check_string(i) }
    }
    pub fn check_table(&self, i: usize) -> Result<Table> {
        match self.v.get(i - 1) {
            Some(Value::Table(t)) => Ok(t.clone()),
            _ => Err(self.type_err(i, "table")),
        }
    }
    pub fn check_function(&self, i: usize) -> Result<mlua::Function> {
        match self.v.get(i - 1) {
            Some(Value::Function(f)) => Ok(f.clone()),
            _ => Err(self.type_err(i, "function")),
        }
    }
}

/// `lua_Number2str` ("%.14g") plus the ".0" suffix `tostring` adds to integral floats.
pub fn fmt_number(n: f64) -> String {
    if n.is_nan() {
        return if n.is_sign_negative() { "-nan".into() } else { "nan".into() };
    }
    if n.is_infinite() {
        return if n < 0.0 { "-inf".into() } else { "inf".into() };
    }
    let s = fmt_g(n, 14);
    if s.bytes().all(|c| c == b'-' || c.is_ascii_digit()) { format!("{s}.0") } else { s }
}

/// C `%.<prec>g`
pub fn fmt_g(n: f64, prec: usize) -> String {
    if n == 0.0 {
        return if n.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    let prec = prec.max(1);
    let e = format!("{:.*e}", prec - 1, n);
    let (mant, exp) = e.split_once("e").unwrap();
    let x: i32 = exp.parse().unwrap();
    if x < -4 || x >= prec as i32 {
        let mant = trim_zeros(mant);
        format!("{}e{}{:02}", mant, if x < 0 { '-' } else { '+' }, x.abs())
    } else {
        let decimals = (prec as i32 - 1 - x).max(0) as usize;
        trim_zeros(&format!("{:.*}", decimals, n)).to_string()
    }
}

fn trim_zeros(s: &str) -> &str {
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.') } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g_format_matches_c() {
        assert_eq!(fmt_g(0.1, 14), "0.1");
        assert_eq!(fmt_g(100.0, 14), "100");
        assert_eq!(fmt_g(1e15, 14), "1e+15");
        assert_eq!(fmt_g(123456.789, 4), "1.235e+05");
        assert_eq!(fmt_g(0.0001, 4), "0.0001");
        assert_eq!(fmt_g(0.00001234, 4), "1.234e-05");
        assert_eq!(fmt_g(-2.5, 4), "-2.5");
        assert_eq!(fmt_number(3.0), "3.0");
        assert_eq!(fmt_number(-0.0), "-0.0");
        assert_eq!(fmt_number(1.0 / 3.0), "0.33333333333333");
    }

    #[test]
    fn string_coercion() {
        assert!(matches!(str_to_number(" 12 "), Some(Value::Integer(12))));
        assert!(matches!(str_to_number("0x10"), Some(Value::Integer(16))));
        assert!(matches!(str_to_number("1.5"), Some(Value::Number(_))));
        assert!(str_to_number("abc").is_none() && str_to_number("").is_none());
    }
}
