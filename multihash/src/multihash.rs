use crate::hasher::{Digest, Size};
use crate::Error;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::fmt::Debug;
use generic_array::{ArrayLength, GenericArray};

use bytecursor::ByteCursor;
use sp_std::vec::Vec;
use unsigned_varint::{decode, encode as varint_encode};

/// Trait that implements hashing.
///
/// It is usually implemented by a custom code table enum that derives the [`Multihash` derive].
///
/// [`Multihash` derive]: crate::derive
pub trait MultihashDigest:
    TryFrom<u64> + Into<u64> + Send + Sync + Unpin + Copy + Eq + Debug + 'static
{
    /// The maximum size a hash will allocate.
    type AllocSize: Size;

    /// Calculate the hash of some input data.
    ///
    /// # Example
    ///
    /// ```
    /// // `Code` implements `MultihashDigest`
    /// use multihash::{Code, MultihashDigest};
    ///
    /// let hash = Code::Sha3_256.digest(b"Hello world!");
    /// println!("{:02x?}", hash);
    /// ```
    fn digest(&self, input: &[u8]) -> Multihash<Self::AllocSize>;

    /// Create a multihash from an existing [`Digest`].
    ///
    /// # Example
    ///
    /// ```
    /// use multihash::{Code, MultihashDigest, Sha3_256, StatefulHasher};
    ///
    /// let mut hasher = Sha3_256::default();
    /// hasher.update(b"Hello world!");
    /// let hash = Code::multihash_from_digest(&hasher.finalize());
    /// println!("{:02x?}", hash);
    /// ```
    #[allow(clippy::needless_lifetimes)]
    fn multihash_from_digest<'a, S, D>(digest: &'a D) -> Multihash<Self::AllocSize>
    where
        S: Size,
        D: Digest<S>,
        Self: From<&'a D>;
}

/// A Multihash instance that only supports the basic functionality and no hashing.
///
/// With this Multihash implementation you can operate on Multihashes in a generic way, but
/// no hasher implementation is associated with the code.
///
/// # Example
///
/// ```
/// use multihash::Multihash;
///
/// const Sha3_256: u64 = 0x16;
/// let digest_bytes = [
///     0x16, 0x20, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8, 0x9e,
///     0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c, 0x3b, 0xfb,
///     0xf2, 0x4e, 0x39, 0x38,
/// ];
/// let mh = Multihash::from_bytes(&digest_bytes).unwrap();
/// assert_eq!(mh.code(), Sha3_256);
/// assert_eq!(mh.size(), 32);
/// assert_eq!(mh.digest(), &digest_bytes[2..]);
/// ```
#[cfg_attr(feature = "serde-codec", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde-codec", derive(serde::Serialize))]
#[cfg_attr(feature = "serde-codec", serde(bound = "S: Size"))]
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Multihash<S: Size> {
    /// The code of the Multihash.
    code: u64,
    /// The actual size of the digest in bytes (not the allocated size).
    size: u8,
    /// The digest.
    digest: GenericArray<u8, S>,
}

impl<S: Size> Copy for Multihash<S> where <S as ArrayLength<u8>>::ArrayType: Copy {}

impl<S: Size> Multihash<S> {
    /// Wraps the digest in a multihash.
    pub fn wrap(code: u64, input_digest: &[u8]) -> Result<Self, Error> {
        if input_digest.len() > S::to_usize() {
            return Err(Error::InvalidSize(input_digest.len() as _));
        }
        let size = input_digest.len();
        let mut digest = GenericArray::default();
        digest[..size].copy_from_slice(input_digest);
        Ok(Self {
            code,
            size: size as u8,
            digest,
        })
    }

    /// Returns the code of the multihash.
    pub fn code(&self) -> u64 {
        self.code
    }

    /// Returns the size of the digest.
    pub fn size(&self) -> u8 {
        self.size
    }

    /// Returns the digest.
    pub fn digest(&self) -> &[u8] {
        &self.digest[..self.size as usize]
    }

    /// Reads a multihash from a byte stream.
    pub fn read(r: &mut ByteCursor) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let (code, size, digest) = match read_multihash(r) {
            Ok((c, s, d)) => (c, s, d),
            Err(e) => return Err(e),
        };
        Ok(Self { code, size, digest })
    }

    /// Parses a multihash from a bytes.
    ///
    /// You need to make sure the passed in bytes have the correct length. The digest length
    /// needs to match the `size` value of the multihash.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: Sized,
    {
      let mut r = ByteCursor::new(bytes.to_vec());
      let result = match Self::read(&mut r) {
        Ok(r) => r,
        Err(_) => return Err(Error::Varint(decode::Error::Overflow)),
      };
      // There were more bytes supplied than read
      if bytes.len() >= r.position() as usize + 1 {
        return Err(Error::InvalidSize(r.get_ref().len().try_into().expect(
          "Currently the maximum size is 255, therefore always fits into usize",
        )));
      }
      
      Ok(result)
    }

    /// Writes a multihash to a byte stream.
    pub fn write(&self, w: &mut ByteCursor) -> Result<(), Error> {
        write_multihash(w, self.code(), self.size(), self.digest())
    }

    /// Returns the bytes of a multihash.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = ByteCursor::new(Vec::with_capacity(self.size().into()));
        self.write(&mut bytes)
            .expect("writing to a vec should never fail");

        bytes.into_inner()
    }
}

// Don't hash the whole allocated space, but just the actual digest
#[allow(clippy::derive_hash_xor_eq)]
impl<S: Size> core::hash::Hash for Multihash<S> {
    fn hash<T: core::hash::Hasher>(&self, state: &mut T) {
        self.code.hash(state);
        self.digest().hash(state);
    }
}

impl<S: Size> From<Multihash<S>> for Vec<u8> {
    fn from(multihash: Multihash<S>) -> Self {
        multihash.to_bytes()
    }
}

#[cfg(feature = "scale-codec")]
impl parity_scale_codec::Encode for Multihash<crate::U32> {
    fn encode_to<EncOut: parity_scale_codec::Output + ?Sized>(&self, dest: &mut EncOut) {
        let mut digest = [0; 32];
        digest.copy_from_slice(&self.digest);
        self.code.encode_to(dest);
        self.size.encode_to(dest);
        digest.encode_to(dest);
    }
}

#[cfg(feature = "scale-codec")]
impl parity_scale_codec::EncodeLike for Multihash<crate::U32> {}

#[cfg(feature = "scale-codec")]
impl parity_scale_codec::Decode for Multihash<crate::U32> {
    fn decode<DecIn: parity_scale_codec::Input>(
        input: &mut DecIn,
    ) -> Result<Self, parity_scale_codec::Error> {
        Ok(Multihash {
            code: parity_scale_codec::Decode::decode(input)?,
            size: parity_scale_codec::Decode::decode(input)?,
            digest: {
                let digest = <[u8; 32]>::decode(input)?;
                GenericArray::clone_from_slice(&digest)
            },
        })
    }
}

#[cfg(feature = "scale-codec")]
impl parity_scale_codec::Encode for Multihash<crate::U64> {
    fn encode_to<EncOut: parity_scale_codec::Output + ?Sized>(&self, dest: &mut EncOut) {
        let mut digest = [0; 64];
        digest.copy_from_slice(&self.digest);
        self.code.encode_to(dest);
        self.size.encode_to(dest);
        digest.encode_to(dest);
    }
}

#[cfg(feature = "scale-codec")]
impl parity_scale_codec::EncodeLike for Multihash<crate::U64> {}

#[cfg(feature = "scale-codec")]
impl parity_scale_codec::Decode for Multihash<crate::U64> {
    fn decode<DecIn: parity_scale_codec::Input>(
        input: &mut DecIn,
    ) -> Result<Self, parity_scale_codec::Error> {
        Ok(Multihash {
            code: parity_scale_codec::Decode::decode(input)?,
            size: parity_scale_codec::Decode::decode(input)?,
            digest: {
                let digest = <[u8; 64]>::decode(input)?;
                GenericArray::clone_from_slice(&digest)
            },
        })
    }
}

/// Writes the multihash to a byte stream.
pub fn write_multihash(
    w: &mut ByteCursor,
    code: u64,
    size: u8,
    digest: &[u8],
) -> Result<(), Error> {
    let mut code_buf = varint_encode::u64_buffer();
    let code = varint_encode::u64(code, &mut code_buf);

    let mut size_buf = varint_encode::u8_buffer();
    let size = varint_encode::u8(size, &mut size_buf);

    match w.write_all(code) {
        Ok(_) => (),
        Err(_) => return Err(Error::Varint(decode::Error::Overflow)),
    };
    match w.write_all(size) {
        Ok(_) => (),
        Err(_) => return Err(Error::Varint(decode::Error::Overflow)),
    };
    match w.write_all(digest) {
        Ok(_) => (),
        Err(_) => return Err(Error::Varint(decode::Error::Overflow)),
    };
    w.set_position(0);
    Ok(())
}

pub fn read_u64(r: &mut ByteCursor) -> Result<u64, Error> {
    let mut b = varint_encode::u64_buffer();
    for i in 0..b.len() {
        let n = r.read(&mut b[i..(i + 1)]);
        if n == 0 {
            return Err(Error::Varint(decode::Error::Overflow));
        }
        if decode::is_last(b[i]) {
          match decode::u64(&b[..=i]) {
            Ok(d) => return Ok(d.0),
            Err(_) => return Err(Error::Varint(decode::Error::Overflow)),
          };
            //return Ok(decode::u64(&b[..=i]).unwrap().0);
        }
    }
    Err(Error::Varint(decode::Error::Overflow))
}

/// Reads a multihash from a byte stream that contains a full multihash (code, size and the digest)
///
/// Returns the code, size and the digest. The size is the actual size and not the
/// maximum/allocated size of the digest.
///
/// Currently the maximum size for a digest is 255 bytes.
pub fn read_multihash<S>(r: &mut ByteCursor) -> Result<(u64, u8, GenericArray<u8, S>), Error>
where
    S: Size,
{
    let code = match read_u64(r) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    let size = match read_u64(r) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    if size > S::to_u64() || size > u8::MAX as u64 {
        return Err(Error::InvalidSize(size));
    }

    let mut digest = GenericArray::default();

    match r.read_exact(&mut digest[..size as usize]) {
        Ok(_) => (),
        Err(_) => return Err(Error::Varint(decode::Error::Overflow)),
    }
    Ok((code, size as u8, digest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multihash_impl::Code;

    #[test]
    fn roundtrip() {
        let hash = Code::Sha2_256.digest(b"hello world");
        let mut buf = ByteCursor::new([0u8; 35].to_vec());
        hash.write(&mut buf).unwrap();
        buf.set_position(0);
        let hash2 = Multihash::read(&mut buf).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    #[cfg(feature = "scale-codec")]
    fn test_scale() {
        use parity_scale_codec::{Decode, Encode};

        let mh = Multihash::<crate::U32>::default();
        let bytes = mh.encode();
        let mh2: Multihash<crate::U32> = Decode::decode(&mut &bytes[..]).unwrap();
        assert_eq!(mh, mh2);
    }

    #[test]
    #[cfg(feature = "serde-codec")]
    fn test_serde() {
        let mh = Multihash::<crate::U32>::default();
        let bytes = serde_json::to_string(&mh).unwrap();
        let mh2 = serde_json::from_str(&bytes).unwrap();
        assert_eq!(mh, mh2);
    }
}
