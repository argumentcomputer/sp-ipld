use crate::{
  codec::{
    Codec,
    Decode,
    Encode,
    UnsupportedCodec,
  },
  ipld::Ipld,
};

use sp_cid::Cid;
use sp_multihash::{
  Code,
  MultihashDigest,
};

use sp_std::convert::TryFrom;

pub mod decode;
pub mod encode;

/// A struct representing the dag-cbor IPLD codec.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DagCborCodec;

impl Codec for DagCborCodec {}

impl From<DagCborCodec> for u64 {
  fn from(_: DagCborCodec) -> Self { 0x71 }
}

impl TryFrom<u64> for DagCborCodec {
  type Error = UnsupportedCodec;

  fn try_from(_: u64) -> core::result::Result<Self, Self::Error> { Ok(Self) }
}

/// A trait representing the capability to both decode and encode
/// the type using the dag-cbor codec
pub trait DagCbor: Encode<DagCborCodec> + Decode<DagCborCodec> {}

impl<T: Encode<DagCborCodec> + Decode<DagCborCodec>> DagCbor for T {}

/// Returns the corresponding dag-json v1 Cid 
/// to the passed IPLD
/// # Panics
/// Panics if x could not be encoded into a dag-cbor bytecursor
pub fn cid(x: &Ipld) -> Cid {
  Cid::new_v1(
    0x71,
    Code::Blake2b256
      .digest(DagCborCodec.encode(x).unwrap().into_inner().as_ref()),
  )
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::ipld::*;
  use bytecursor::ByteCursor;
  use quickcheck::{
    quickcheck,
    Arbitrary,
    Gen,
  };

  use sp_std::collections::btree_map::BTreeMap;

  fn encode_decode_id<T: DagCbor + PartialEq<T> + Clone>(value: T) -> bool {
    let mut bc = ByteCursor::new(Vec::new());
    match Encode::encode(&value, DagCborCodec, &mut bc) {
      Ok(()) => {
        bc.set_position(0);
        match Decode::decode(DagCborCodec, &mut bc) {
          Ok(new_value) => return value == new_value,
          Err(e) => println!("Error occurred during decoding: {}", e),
        }
      }
      Err(e) => println!("Error occurred during encoding: {}", e),
    }
    false
  }

  #[quickcheck]
  pub fn edid_null() -> bool { encode_decode_id(Ipld::Null) }

  #[quickcheck]
  pub fn edid_bool(x: bool) -> bool { encode_decode_id(Ipld::Bool(x)) }

  #[quickcheck]
  pub fn edid_integer(x: i64) -> bool {
    encode_decode_id(Ipld::Integer(x as i128))
  }

  #[quickcheck]
  pub fn edid_bytes(x: Vec<u8>) -> bool { encode_decode_id(Ipld::Bytes(x)) }

  #[quickcheck]
  pub fn edid_string(x: String) -> bool { encode_decode_id(Ipld::String(x)) }

  // fails on `Vec<Float(inf)>`
  #[quickcheck]
  pub fn edid_list(x: Vec<Ipld>) -> bool { encode_decode_id(Ipld::List(x)) }

  #[quickcheck]
  pub fn edid_string_map(x: BTreeMap<String, Ipld>) -> bool {
    encode_decode_id(Ipld::StringMap(x))
  }

  #[derive(Debug, Clone)]
  pub struct ACid(pub Cid);

  impl Arbitrary for ACid {
    fn arbitrary(g: &mut Gen) -> Self {
      ACid(crate::ipld::tests::arbitrary_cid(g))
    }
  }

  #[quickcheck]
  pub fn edid_link(x: ACid) -> bool { encode_decode_id(Ipld::Link(x.0)) }
}
