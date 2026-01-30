use crate::gf256::Gf256;

#[derive(Debug)]
pub enum HeccError {
    InvalidParams,
    InvalidShardIndex,
    DuplicateIndex,
    NotEnoughShards,
}

#[derive(Debug, Clone, Copy)]
pub struct HeccParams {
    pub k: usize,
    pub t: usize,
    pub n: usize,
}

impl HeccParams {
    pub fn validate(&self) -> Result<(), HeccError> {
        if self.k == 0 || self.t == 0 || self.n < self.k + self.t || self.n > 255 {
            return Err(HeccError::InvalidParams);
        }
        Ok(())
    }
}

/// HECC.Enc per MCP whitepaper (Alg. 2).
/// - `m` is a length-K message vector in F.
/// - `r` is a length-T randomness vector in F.
/// - `n` is total number of shreds N, with N >= K + T and N <= 255.
pub fn hecc_encode(m: &[u8], r: &[u8], n: usize) -> Result<Vec<u8>, HeccError> {
    let k = m.len();
    let t = r.len();
    if k == 0 || t == 0 || n < k + t || n > 255 {
        return Err(HeccError::InvalidParams);
    }

    let gf = Gf256::default();
    let mut coeffs = Vec::with_capacity(k + t);
    coeffs.extend_from_slice(m);
    coeffs.extend_from_slice(r);

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let x = (i + 1) as u8;
        let y = poly_eval(&gf, &coeffs, x);
        out.push(y);
    }
    Ok(out)
}

/// HECC.Dec per MCP whitepaper (Alg. 2).
/// Input shards are (value, index) where index is 1-based in [1, N].
pub fn hecc_decode(
    shards: &[(u8, usize)],
    k: usize,
    t: usize,
) -> Result<(Vec<u8>, Vec<u8>), HeccError> {
    if k == 0 || t == 0 {
        return Err(HeccError::InvalidParams);
    }
    let need = k + t;
    if shards.len() < need {
        return Err(HeccError::NotEnoughShards);
    }

    let mut unique = Vec::with_capacity(need);
    for &(v, idx) in shards {
        if idx == 0 || idx > 255 {
            return Err(HeccError::InvalidShardIndex);
        }
        if unique.iter().any(|&(_, i)| i == idx) {
            return Err(HeccError::DuplicateIndex);
        }
        unique.push((v, idx));
        if unique.len() == need {
            break;
        }
    }
    if unique.len() < need {
        return Err(HeccError::NotEnoughShards);
    }

    let gf = Gf256::default();
    let xs: Vec<u8> = unique.iter().map(|&(_, idx)| idx as u8).collect();
    let ys: Vec<u8> = unique.iter().map(|&(v, _)| v).collect();

    let coeffs = lagrange_interpolate(&gf, &xs, &ys, need);
    let m = coeffs[..k].to_vec();
    let r = coeffs[k..k + t].to_vec();
    Ok((m, r))
}

fn poly_eval(gf: &Gf256, coeffs: &[u8], x: u8) -> u8 {
    let mut y = 0u8;
    let mut x_pow = 1u8;
    for &c in coeffs {
        if c != 0 {
            y ^= gf.mul(c, x_pow);
        }
        x_pow = gf.mul(x_pow, x);
    }
    y
}

fn lagrange_interpolate(gf: &Gf256, xs: &[u8], ys: &[u8], degree: usize) -> Vec<u8> {
    let mut poly = vec![0u8; degree];
    for j in 0..degree {
        let mut basis = vec![1u8];
        let mut denom = 1u8;
        for m in 0..degree {
            if m == j {
                continue;
            }
            let xj = xs[j];
            let xm = xs[m];
            let diff = gf.sub(xj, xm);
            denom = gf.mul(denom, diff);
            basis = poly_mul(gf, &basis, &[gf.sub(0, xm), 1]);
        }
        let scale = gf.div(ys[j], denom);
        for (i, &b) in basis.iter().enumerate() {
            if i < poly.len() {
                poly[i] ^= gf.mul(scale, b);
            }
        }
    }
    poly
}

fn poly_mul(gf: &Gf256, a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; a.len() + b.len() - 1];
    for (i, &ai) in a.iter().enumerate() {
        if ai == 0 {
            continue;
        }
        for (j, &bj) in b.iter().enumerate() {
            if bj == 0 {
                continue;
            }
            out[i + j] ^= gf.mul(ai, bj);
        }
    }
    out
}
