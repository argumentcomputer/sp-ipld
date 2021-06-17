//! # cid
//!
//! Implementation of [cid](https://github.com/ipld/cid) in Rust.

#![cfg_attr(not(feature = "std"), no_std)]

mod cid;
mod error;
mod version;

#[cfg(any(test, feature = "arb"))]
mod arb;

pub use self::cid::Cid as CidGeneric;
pub use self::error::{Error, Result};
pub use self::version::Version;

pub use multibase;
pub use sp_multihash;

extern crate alloc;
use bytecursor::ByteCursor;
use unsigned_varint::{encode as varint_encode, decode};

  ///Reader function from unsigned_varint 
  pub fn varint_read_u64(r: &mut ByteCursor) -> Result<u64> {
    let mut buf: [u8;10] = [0;10];
    let slice = r.get_ref();
    for i in 0..10 {
      buf[i] = slice[i];
    }
    let b = varint_encode::u64(0, &mut buf); 
    for i in 0..b.len() {
      r.read(&mut (b[i..i+1]).to_vec());
      if decode::is_last(b[i]) {
        return Ok(decode::u64(&b[..=i]).unwrap().0)
      }
    }
    Err(Error::VarIntDecodeError)
  }

/// A Cid that contains a multihash with an allocated size of 512 bits.
///
/// This is the same digest size the default multihash code table has.
///
/// If you need a CID that is generic over its digest size, use [`CidGeneric`] instead.
pub type Cid = CidGeneric<sp_multihash::U64>;
