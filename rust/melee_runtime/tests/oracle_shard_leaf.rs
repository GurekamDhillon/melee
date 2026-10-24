//! Direct C-oracle coverage for the helper definitions omitted by the earlier public-API oracle.
use melee_runtime::tu::{gekko_fp as gk, mtx_pc as mtx};

#[test]
#[should_panic]
fn invalid_position_matrix_index_keeps_the_c_assertion() {
    melee_runtime::tu::util::HSD_Index2PosNrmMtx(10);
}

#[test]
fn helpers_match_shipping_c() {
    let mut counts = [0usize; 4];
    for (line, row) in include_str!("data/oracle_shard_leaf.txt").lines().enumerate() {
        let mut words = row.split_whitespace();
        let tag = words.next().unwrap();
        let mut hex = || u64::from_str_radix(words.next().unwrap(), 16).unwrap();
        match tag {
            "bits" => {
                let bits = hex();
                let value = f64::from_bits(bits);
                assert_eq!(gk::gekko_bits_to_double(bits).to_bits(), hex(), "bits-to-double row {line}");
                assert_eq!(gk::gekko_double_to_bits(value), hex(), "double-to-bits row {line}");
                assert_eq!(gk::gekko_make_quiet(value).to_bits(), hex(), "quiet row {line}");
                assert_eq!(mtx::gekko_force25(value).to_bits(), hex(), "force25 row {line}");
                let negative = hex() as i32;
                assert_eq!(gk::gekko_infinity(negative).to_bits(), hex(), "infinity row {line}");
                assert_eq!(gk::gekko_nan().to_bits(), hex(), "nan row {line}");
                counts[0] += 1;
            }
            "trig" => {
                let a = f32::from_bits(hex() as u32);
                let c = f32::from_bits(hex() as u32);
                let b = f32::from_bits(hex() as u32);
                let x = f32::from_bits(hex() as u32);
                let ysq = f32::from_bits(hex() as u32);
                assert_eq!(gk::gk_fmadds(a, c, b).to_bits(), hex() as u32, "fmadds row {line}");
                assert_eq!(mtx::mtx_madd(a, c, b).to_bits(), hex() as u32, "mtx_madd row {line}");
                let (reduced, quadrant) = gk::gk_sincos_reduce(x);
                assert_eq!(reduced.to_bits(), hex() as u32, "reduce row {line}");
                assert_eq!(quadrant as u32, hex() as u32, "quadrant row {line}");
                assert_eq!(gk::gk_sincos_even(ysq).to_bits(), hex() as u32, "even row {line}");
                assert_eq!(gk::gk_sincos_odd_head(ysq).to_bits(), hex() as u32, "odd row {line}");
                counts[1] += 1;
            }
            "copy" => {
                let mut src = [[0.0; 4]; 3];
                for value in src.iter_mut().flatten() { *value = f32::from_bits(hex() as u32); }
                for value in mtx::PSMTXCopy(&src).iter().flatten() {
                    assert_eq!(value.to_bits(), hex() as u32, "copy row {line}");
                }
                for value in mtx::PSMTXIdentity().iter().flatten() {
                    assert_eq!(value.to_bits(), hex() as u32, "identity row {line}");
                }
                counts[2] += 1;
            }
            "rot" => {
                let mut matrix = [[0.0; 4]; 3];
                for value in matrix.iter_mut().flatten() { *value = f32::from_bits(hex() as u32); }
                let axis = hex() as u8;
                let sin = f32::from_bits(hex() as u32);
                let cos = f32::from_bits(hex() as u32);
                mtx::mtx_rot_trig(&mut matrix, axis, sin, cos);
                for value in matrix.iter().flatten() {
                    assert_eq!(value.to_bits(), hex() as u32, "rotation row {line}");
                }
                counts[3] += 1;
            }
            _ => panic!("unexpected row {line}: {tag}"),
        }
        assert!(words.next().is_none(), "unconsumed values at row {line}");
    }
    assert_eq!(counts, [2048; 4]);
}
