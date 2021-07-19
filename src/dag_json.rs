use crate::{
  codec::*,
  Ipld,
  References,
};
use alloc::string::{
  String,
  ToString,
};
use bytecursor::ByteCursor;
use core::convert::TryFrom;
use sp_cid::Cid;
use sp_multihash::{
  Code,
  MultihashDigest,
};

mod codec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DagJsonCodec;

impl Codec for DagJsonCodec {}

impl From<DagJsonCodec> for u64 {
  fn from(_: DagJsonCodec) -> Self { 0x0129 }
}

impl TryFrom<u64> for DagJsonCodec {
  type Error = UnsupportedCodec;

  fn try_from(_: u64) -> core::result::Result<Self, Self::Error> { Ok(Self) }
}

impl Encode<DagJsonCodec> for Ipld {
  fn encode(&self, _: DagJsonCodec, w: &mut ByteCursor) -> Result<(), String> {
    codec::encode(self, w).map_err(|x| x.to_string())
  }
}

impl Decode<DagJsonCodec> for Ipld {
  fn decode(_: DagJsonCodec, r: &mut ByteCursor) -> Result<Self, String> {
    match codec::decode(r) {
      Ok(ipld) => Ok(ipld),
      Err(_) => Err(String::from("Failed to decode JSON")),
    }
  }
}

impl References<DagJsonCodec> for Ipld {
  fn references<E: Extend<Cid>>(
    c: DagJsonCodec,
    r: &mut ByteCursor,
    set: &mut E,
  ) -> Result<(), String> {
    Ipld::decode(c, r)?.references(set);
    Ok(())
  }
}

pub fn cid(x: &Ipld) -> Cid {
  Cid::new_v1(
    0x0129,
    Code::Blake2b256
      .digest(DagJsonCodec.encode(x).unwrap().into_inner().as_ref()),
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

  fn encode_decode_id<
    T: Encode<DagJsonCodec> + Decode<DagJsonCodec> + PartialEq<T> + Clone,
  >(
    value: T,
  ) -> bool {
    let mut bc = ByteCursor::new(Vec::new());
    match Encode::encode(&value, DagJsonCodec, &mut bc) {
      Ok(()) => {
        bc.set_position(0);
        match Decode::decode(DagJsonCodec, &mut bc) {
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
