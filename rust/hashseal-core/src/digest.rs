use crate::error::{Error, Result};
use std::fmt;
use std::str::FromStr;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(rename_all = "lowercase"))]
pub enum Algorithm {
    Blake3,
    Sha256,
}

impl Algorithm {
    pub fn as_str(self) -> &'static str {
        match self {
            Algorithm::Blake3 => "blake3",
            Algorithm::Sha256 => "sha256",
        }
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Algorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "blake3" => Ok(Algorithm::Blake3),
            "sha256" => Ok(Algorithm::Sha256),
            other => Err(Error::UnsupportedAlgorithm(other.to_string())),
        }
    }
}

/// Algorithm-qualified digest, e.g. `blake3:deadbeef…`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Digest {
    pub algorithm: Algorithm,
    pub hex: String,
}

impl Digest {
    pub fn compute(algorithm: Algorithm, data: &[u8]) -> Self {
        let hex = match algorithm {
            Algorithm::Blake3 => blake3::hash(data).to_hex().to_string(),
            Algorithm::Sha256 => pure_sha256_hex(data),
        };
        Self { algorithm, hex }
    }

    pub fn qualified(&self) -> String {
        format!("{}:{}", self.algorithm.as_str(), self.hex)
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.qualified())
    }
}

impl FromStr for Digest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().trim_matches('"');
        let (alg, hex) = s
            .split_once(':')
            .ok_or_else(|| Error::InvalidFormat(format!("expected alg:hex, got {s}")))?;
        let algorithm = Algorithm::from_str(alg)?;
        let hex = hex.trim().to_ascii_lowercase();
        if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidFormat(format!("invalid hex in digest: {s}")));
        }
        Ok(Self { algorithm, hex })
    }
}

fn pure_sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let bit_len = (data.len() as u64).saturating_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = String::with_capacity(64);
    for word in h {
        out.push_str(&format!("{word:08x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_roundtrip_parse() {
        let d = Digest::compute(Algorithm::Blake3, b"hello");
        let parsed: Digest = d.qualified().parse().unwrap();
        assert_eq!(parsed, d);
    }

    #[test]
    fn sha256_empty() {
        let d = Digest::compute(Algorithm::Sha256, b"");
        assert_eq!(
            d.hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
