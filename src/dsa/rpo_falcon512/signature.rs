use super::{
    math::{decompress_signature, pub_key_from_bytes, FalconFelt, FastFft, Polynomial},
    ByteReader, ByteWriter, Deserializable, DeserializationError, Felt, NonceBytes, NonceElements,
    PublicKeyBytes, Rpo256, Serializable, SignatureBytes, Word, MODULUS, N, SIG_HEADER_LEN,
    SIG_L2_BOUND, SIG_NONCE_LEN, ZERO,
};
use crate::utils::string::*;
use num::Zero;

// FALCON SIGNATURE
// ================================================================================================

/// An RPO Falcon512 signature over a message.
///
/// The signature is a pair of polynomials (s1, s2) in (Z_p\[x\]/(phi))^2, where:
/// - p := 12289
/// - phi := x^512 + 1
/// - s1 = c - s2 * h
/// - h is a polynomial representing the public key and c is a polynomial that is the hash-to-point
///   of the message being signed.
///
/// The signature  verifies if and only if:
/// 1. s1 = c - s2 * h
/// 2. |s1|^2 + |s2|^2 <= SIG_L2_BOUND
///
/// where |.| is the norm.
///
/// [Signature] also includes the extended public key which is serialized as:
/// 1. 1 byte representing the log2(512) i.e., 9.
/// 2. 896 bytes for the public key. This is decoded into the `h` polynomial above.
///
/// The actual signature is serialized as:
/// 1. A header byte specifying the algorithm used to encode the coefficients of the `s2` polynomial
///    together with the degree of the irreducible polynomial phi.
///    The general format of this byte is 0b0cc1nnnn where:
///     a. cc is either 01 when the compressed encoding algorithm is used and 10 when the
///     uncompressed algorithm is used.
///     b. nnnn is log2(N) where N is the degree of the irreducible polynomial phi.
///    The current implementation works always with cc equal to 0b01 and nnnn equal to 0b1001 and
///    thus the header byte is always equal to 0b00111001.
/// 2. 40 bytes for the nonce.
/// 3. 625 bytes encoding the `s2` polynomial above.
///
/// The total size of the signature (including the extended public key) is 1563 bytes.
#[derive(Debug, Clone)]
pub struct Signature {
    pub(super) pk: PublicKeyBytes,
    pub(super) sig: SignatureBytes,

    // Cached polynomial decoding for public key and signature
    pub(super) pk_poly: Polynomial<FalconFelt>,
    pub(super) sig_poly: Polynomial<FalconFelt>,
}

impl Signature {
    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the public key polynomial h.
    pub fn pub_key_poly(&self) -> &Polynomial<FalconFelt> {
        &self.pk_poly
    }

    /// Returns the nonce component of the signature represented as field elements.
    ///
    /// Nonce bytes are converted to field elements by taking consecutive 5 byte chunks
    /// of the nonce and interpreting them as field elements.
    pub fn nonce(&self) -> NonceBytes {
        // we assume that the signature was constructed with a valid signature, and thus
        // expect() is OK here.
        self.sig[SIG_HEADER_LEN..SIG_HEADER_LEN + SIG_NONCE_LEN]
            .try_into()
            .expect("invalid signature")
    }

    // Returns the polynomial representation of the signature in Z_p[x]/(phi).
    pub fn sig_poly(&self) -> &Polynomial<FalconFelt> {
        &self.sig_poly
    }

    // HASH-TO-POINT
    // --------------------------------------------------------------------------------------------

    /// Returns a polynomial in Z_p\[x\]/(phi) representing the hash of the provided message.
    pub fn hash_to_point(&self, message: Word) -> Polynomial<FalconFelt> {
        let nonce = &self.nonce();
        hash_to_point(message, nonce)
    }

    // SIGNATURE VERIFICATION
    // --------------------------------------------------------------------------------------------
    /// Returns true if this signature is a valid signature for the specified message generated
    /// against key pair matching the specified public key commitment.
    pub fn verify(&self, message: Word, pubkey_com: Word) -> bool {
        let h: Polynomial<Felt> = self.pub_key_poly().into();
        let h_digest: Word = Rpo256::hash_elements(&h.coefficients).into();
        if h_digest != pubkey_com {
            return false;
        }
        let c = hash_to_point(message, &self.nonce());

        let s2 = &self.sig_poly;
        let s2_ntt = s2.fft();
        let h_ntt = self.pk_poly.fft();
        let c_ntt = c.fft();

        // s1 = c - s2 * pk.h;
        let s1_ntt = c_ntt - s2_ntt.hadamard_mul(&h_ntt);
        let s1 = s1_ntt.ifft();

        let length_squared_s1 = s1.norm_squared();
        let length_squared_s2 = s2.norm_squared();
        let length_squared = length_squared_s1 + length_squared_s2;
        (length_squared as u64) < SIG_L2_BOUND
    }
}

// SERIALIZATION / DESERIALIZATION
// ================================================================================================

impl Serializable for Signature {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_bytes(&self.pk);
        target.write_bytes(&self.sig);
    }
}

impl Deserializable for Signature {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let pk: PublicKeyBytes = source.read_array()?;
        let sig: SignatureBytes = source.read_array()?;

        // make sure public key and signature can be decoded correctly
        let pk_polynomial = pub_key_from_bytes(&pk)
            .map_err(|err| DeserializationError::InvalidValue(err.to_string()))?;
        let sig_polynomial = if let Ok(poly) = decompress_signature(&sig) {
            poly
        } else {
            return Err(DeserializationError::InvalidValue(
                "Invalid signature encoding".to_string(),
            ));
        };

        Ok(Self {
            pk,
            sig,
            pk_poly: pk_polynomial,
            sig_poly: sig_polynomial,
        })
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Returns a polynomial in Z_p[x]/(phi) representing the hash of the provided message and
/// nonce.
pub fn hash_to_point(message: Word, nonce: &NonceBytes) -> Polynomial<FalconFelt> {
    let mut state = [ZERO; Rpo256::STATE_WIDTH];

    // absorb the nonce into the state
    let nonce = decode_nonce(nonce);
    for (&n, s) in nonce.iter().zip(state[Rpo256::RATE_RANGE].iter_mut()) {
        *s = n;
    }
    Rpo256::apply_permutation(&mut state);

    // absorb message into the state
    for (&m, s) in message.iter().zip(state[Rpo256::RATE_RANGE].iter_mut()) {
        *s = m;
    }

    // squeeze the coefficients of the polynomial
    let mut i = 0;
    let mut res = [FalconFelt::zero(); N];
    for _ in 0..64 {
        Rpo256::apply_permutation(&mut state);
        for a in &state[Rpo256::RATE_RANGE] {
            res[i] = FalconFelt::new((a.as_int() % MODULUS as u64) as i16);
            i += 1;
        }
    }

    Polynomial::new(res.to_vec())
}

/// Converts byte representation of the nonce into field element representation.
pub fn decode_nonce(nonce: &NonceBytes) -> NonceElements {
    let mut buffer = [0_u8; 8];
    let mut result = [ZERO; 8];
    for (i, bytes) in nonce.chunks(5).enumerate() {
        buffer[..5].copy_from_slice(bytes);
        // we can safely (without overflow) create a new Felt from u64 value here since this value
        // contains at most 5 bytes
        result[i] = Felt::new(u64::from_le_bytes(buffer));
    }

    result
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{super::SecretKey, *};

    #[test]
    fn test_serialization_round_trip() {
        let key = SecretKey::new();
        let signature = key.sign(Word::default()).unwrap();
        let serialized = signature.to_bytes();
        let deserialized = Signature::read_from_bytes(&serialized).unwrap();
        assert_eq!(signature.sig_poly(), deserialized.sig_poly());
        assert_eq!(signature.pub_key_poly(), deserialized.pub_key_poly());
    }
}
