//! GF(2^8) arithmetic with primitive polynomial x^8 + x^4 + x^3 + x^2 + 1 (0x11d).

pub const GF256_SIZE: usize = 256;
pub const GF256_ORDER: usize = 255;
pub const PRIM_POLY_256: u16 = 0x11d;

#[derive(Clone)]
pub struct Gf256 {
    exp: [u8; GF256_ORDER * 2],
    log: [u8; GF256_SIZE],
}

impl Default for Gf256 {
    fn default() -> Self {
        let mut exp = [0u8; GF256_ORDER * 2];
        let mut log = [0u8; GF256_SIZE];

        let mut x: u16 = 1;
        for i in 0..GF256_ORDER {
            exp[i] = x as u8;
            log[x as usize] = i as u8;
            x <<= 1;
            if x & 0x100 != 0 {
                x ^= PRIM_POLY_256;
            }
        }
        for i in GF256_ORDER..(GF256_ORDER * 2) {
            exp[i] = exp[i - GF256_ORDER];
        }

        Gf256 { exp, log }
    }
}

impl Gf256 {
    #[inline]
    pub fn add(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }

    #[inline]
    pub fn sub(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }

    #[inline]
    pub fn mul(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }
        let la = self.log[a as usize] as usize;
        let lb = self.log[b as usize] as usize;
        self.exp[la + lb]
    }

    #[inline]
    pub fn div(&self, a: u8, b: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        if b == 0 {
            return 0;
        }
        let la = self.log[a as usize] as i32;
        let lb = self.log[b as usize] as i32;
        let mut idx = la - lb;
        if idx < 0 {
            idx += GF256_ORDER as i32;
        }
        self.exp[idx as usize]
    }

    #[inline]
    pub fn inv(&self, a: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        let la = self.log[a as usize] as usize;
        self.exp[GF256_ORDER - la]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gf256_mul_inv() {
        let gf = Gf256::default();
        for a in 1u8..=254u8 {
            let inv = gf.inv(a);
            assert_eq!(gf.mul(a, inv), 1);
        }
    }
}
