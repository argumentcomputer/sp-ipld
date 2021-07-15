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
    Ok(codec::encode(self, w).map_err(|x| x.to_string())?)
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
    0x71,
    Code::Blake2b256
      .digest(DagJsonCodec.encode(x).unwrap().into_inner().as_ref()),
  )
}
