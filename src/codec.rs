use bytecursor::ByteCursor;
use sp_cid::Cid;

use alloc::string::String;
use sp_std::{
  convert::TryFrom,
  ops::Deref,
  vec::Vec,
};

pub struct UnsupportedCodec(pub u64);

pub enum Error {
  UnsupportedCodec(u64),
}

pub trait Codec:
  Copy
  + Unpin
  + Send
  + Sync
  + 'static
  + Sized
  + TryFrom<u64, Error = UnsupportedCodec>
  + Into<u64> {
  /// # Errors
  ///
  /// Will return `Err` if there was a problem encoding the object into a
  /// `ByteCursor`
  fn encode<T: Encode<Self> + ?Sized>(
    &self,
    obj: &T,
  ) -> Result<ByteCursor, String> {
    let mut buf = ByteCursor::new(Vec::with_capacity(u16::MAX as usize));
    obj.encode(*self, &mut buf)?;
    Ok(buf)
  }

  /// # Errors
  ///
  /// Will return `Err` if there was a problem decoding the `ByteCursor` into an
  /// object
  fn decode<T: Decode<Self>>(
    &self,
    mut bytes: ByteCursor,
  ) -> Result<T, String> {
    T::decode(*self, &mut bytes)
  }

  /// Extends `set` with any cids the type encoded in the bytecursor
  /// refers to.
  ///
  /// # Errors
  ///
  /// Returns `Err` if there were any errors decoding the bytecursor.
  fn references<T: References<Self>, E: Extend<Cid>>(
    &self,
    mut bytes: ByteCursor,
    set: &mut E,
  ) -> Result<(), String> {
    T::references(*self, &mut bytes, set)
  }
}

/// A trait to represent the ability to encode with
/// the codec `C` for the type.
pub trait Encode<C: Codec> {
  /// Encodes `Self` using codec `C` into the mutable bytecursor
  /// `w`. Returns `Ok` if the encoding process succeeded.
  ///
  /// # Errors
  ///
  /// Will return `Err` if there was a problem during encoding
  fn encode(&self, c: C, w: &mut ByteCursor) -> Result<(), String>;
}

impl<C: Codec, T: Encode<C>> Encode<C> for &T {
  fn encode(&self, c: C, w: &mut ByteCursor) -> Result<(), String> {
    self.deref().encode(c, w)
  }
}

/// A trait representing the ability to decode with 
/// the codec `C` for the type.
pub trait Decode<C: Codec>: Sized {
  /// Decodes the bytes in `r` using the codec `C` into
  /// `Self`. Returns `ok` if the bytes represented a valid 
  /// value of the type.
  ///
  /// # Errors
  ///
  /// Will return `Err` if there was a problem during decoding
  fn decode(c: C, r: &mut ByteCursor) -> Result<Self, String>;
}

/// A trait representing the ability to count cid references in the 
/// encoding of the type with the codec `C`
pub trait References<C: Codec>: Sized {
  /// Extends `set` with any Cid references found in the encoding 
  /// of the type in `r` with the codec `C`
  ///
  /// # Errors
  ///
  /// Will return `Err` if `r` did not contain a valid encoding of the
  /// type with codec `C`.
  fn references<E: Extend<Cid>>(
    c: C,
    r: &mut ByteCursor,
    set: &mut E,
  ) -> Result<(), String>;
}

/// A trait for codecs representing the ability to skip values.
pub trait SkipOne: Codec {
  /// Skips a single value of the encoded type using the given codec in `r`.
  ///
  /// # Errors
  ///
  /// Will return `Err` if there was a problem during skipping
  fn skip(&self, r: &mut ByteCursor) -> Result<(), String>;
}
