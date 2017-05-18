// Copyright 2015-2017 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use {error, init, rand};
use untrusted;

/// A key agreement algorithm.
// XXX: This doesn't seem like the best place for this.
pub struct AgreementAlgorithmImpl {
    pub curve: &'static Curve,
    pub ecdh: fn(out: &mut [u8], private_key: &PrivateKey,
                 peer_public_key: untrusted::Input)
                 -> Result<(), error::Unspecified>,
}

pub struct Curve {
    pub public_key_len: usize,
    pub elem_and_scalar_len: usize,

    pub id: CurveID,

    // Precondition: `bytes` is the correct length.
    check_private_key_bytes: fn(bytes: &[u8]) -> Result<(), error::Unspecified>,

    generate_private_key: fn(rng: &rand::SecureRandom)
                             -> Result<PrivateKey, error::Unspecified>,

    public_from_private: fn(public_out: &mut [u8], private_key: &PrivateKey)
                            -> Result<(), error::Unspecified>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CurveID {
    Curve25519,
    P256,
    P384,
}

pub struct KeyPair {
    pub private_key: PrivateKey,
    pub public_key: [u8; PUBLIC_KEY_MAX_LEN],
}

pub struct PrivateKey {
    bytes: [u8; SCALAR_MAX_BYTES],
}

impl<'a> PrivateKey {
    pub fn generate(curve: &Curve, rng: &rand::SecureRandom)
                    -> Result<PrivateKey, error::Unspecified> {
        init::init_once();
        (curve.generate_private_key)(rng)
    }

    pub fn from_bytes(curve: &Curve, bytes: untrusted::Input)
                      -> Result<PrivateKey, error::Unspecified> {
        init::init_once();
        let bytes = bytes.as_slice_less_safe();
        if curve.elem_and_scalar_len != bytes.len() {
            return Err(error::Unspecified);
        }
        (curve.check_private_key_bytes)(bytes)?;
        let mut r = PrivateKey {
            bytes: [0; SCALAR_MAX_BYTES],
        };
        r.bytes[..curve.elem_and_scalar_len].copy_from_slice(bytes);
        Ok(r)
    }

    pub fn bytes(&'a self, curve: &Curve) -> &'a [u8] {
        &self.bytes[..curve.elem_and_scalar_len]
    }

    #[inline(always)]
    pub fn compute_public_key(&self, curve: &Curve, out: &mut [u8])
                              -> Result<(), error::Unspecified> {
        if out.len() != curve.public_key_len {
            return Err(error::Unspecified);
        }
        (curve.public_from_private)(out, self)
    }
}


const ELEM_MAX_BITS: usize = 384;
pub const ELEM_MAX_BYTES: usize = (ELEM_MAX_BITS + 7) / 8;

pub const SCALAR_MAX_BYTES: usize = ELEM_MAX_BYTES;

/// The maximum length, in bytes, of an encoded public key.
pub const PUBLIC_KEY_MAX_LEN: usize = 1 + (2 * ELEM_MAX_BYTES);

/// The maximum length of a PKCS#8 documents generated by *ring* for ECC keys.
///
/// This is NOT the maximum length of a PKCS#8 document that can be consumed by
/// `pkcs8::unwrap_key()`.
///
/// `40` is the length of the P-384 template. It is actually one byte shorter
/// than the P-256 template, but the private key and the public key are much
/// longer.
pub const PKCS8_DOCUMENT_MAX_LEN: usize =
    40 + SCALAR_MAX_BYTES + PUBLIC_KEY_MAX_LEN;

#[path = "curve25519/curve25519.rs"]
pub mod curve25519;

#[path = "suite_b/suite_b.rs"]
pub mod suite_b;
