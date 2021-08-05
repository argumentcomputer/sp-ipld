use alloc::{
  borrow::ToOwned,
  string::String,
  vec,
};
use sp_cid::Cid;
use sp_std::{
  self,
  boxed::Box,
  collections::btree_map::BTreeMap,
  vec::Vec,
};

/// IPLD data format
#[derive(Clone, PartialEq)]
pub enum Ipld {
  /// Represents the absence of a value or the value undefined.
  Null,
  /// Represents a boolean value.
  Bool(bool),
  /// Represents an integer.
  Integer(i128),
  /// Represents a floating point value.
  Float(f64),
  /// Represents an UTF-8 string.
  String(String),
  /// Represents a sequence of bytes.
  Bytes(Vec<u8>),
  /// Represents a list.
  List(Vec<Ipld>),
  /// Represents a map of strings.
  StringMap(BTreeMap<String, Ipld>),
  /// Represents a link to an Ipld node.
  Link(Cid),
}

impl sp_std::fmt::Debug for Ipld {
  fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
    use Ipld::*;
    match self {
      Null => write!(f, "null"),
      Bool(b) => write!(f, "{:?}", b),
      Integer(i) => write!(f, "{:?}", i),
      Float(i) => write!(f, "{:?}", i),
      String(s) => write!(f, "{:?}", s),
      Bytes(b) => write!(f, "{:?}", b),
      List(l) => write!(f, "{:?}", l),
      StringMap(m) => write!(f, "{:?}", m),
      Link(cid) => write!(f, "{}", cid),
    }
  }
}

impl Ipld {
  /// Returns an iterator.
  pub fn iter(&self) -> IpldIter<'_> {
    IpldIter { stack: vec![Box::new(vec![self].into_iter())] }
  }

  /// Returns the references to other blocks.
  pub fn references<E: Extend<Cid>>(&self, set: &mut E) {
    for ipld in self.iter() {
      if let Ipld::Link(cid) = ipld {
        set.extend(sp_std::iter::once(cid.to_owned()));
      }
    }
  }
}

impl<'a> Iterator for IpldIter<'a> {
  type Item = &'a Ipld;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(iter) = self.stack.last_mut() {
        if let Some(ipld) = iter.next() {
          match ipld {
            Ipld::List(list) => {
              self.stack.push(Box::new(list.iter()));
            }
            Ipld::StringMap(map) => {
              self.stack.push(Box::new(map.values()));
            }
            #[cfg(feature = "unleashed")]
            Ipld::IntegerMap(map) => {
              self.stack.push(Box::new(map.values()));
            }
            #[cfg(feature = "unleashed")]
            Ipld::Tag(_, ipld) => {
              self.stack.push(Box::new(ipld.iter()));
            }
            _ => {}
          }
          return Some(ipld);
        }
        else {
          self.stack.pop();
        }
      }
      else {
        return None;
      }
    }
  }
}

/// Ipld iterator.
pub struct IpldIter<'a> {
  stack: Vec<Box<dyn Iterator<Item = &'a Ipld> + 'a>>,
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::rand::Rng;
  use alloc::vec;
  use quickcheck::{
    Arbitrary,
    Gen,
  };
  use sp_multihash::{
    Code,
    MultihashDigest,
  };
  use sp_std::boxed::Box;

  pub(crate) fn arbitrary_cid(g: &mut Gen) -> Cid {
    let mut bytes: [u8; 32] = [0; 32];
    for x in bytes.iter_mut() {
      *x = Arbitrary::arbitrary(g);
    }
    Cid::new_v1(0x55, Code::Blake2b256.digest(&bytes))
  }

  fn frequency<T, F: Fn(&mut Gen) -> T>(g: &mut Gen, gens: Vec<(i64, F)>) -> T {
    if gens.iter().any(|(v, _)| *v < 0) {
      panic!("Negative weight");
    }
    let sum: i64 = gens.iter().map(|x| x.0).sum();
    let mut rng = rand::thread_rng();
    let mut weight: i64 = rng.gen_range(1..=sum);
    for gen in gens {
      if weight - gen.0 <= 0 {
        return gen.1(g);
      }
      else {
        weight -= gen.0;
      }
    }
    panic!("Calculation error for weight = {}", weight);
  }

  fn arbitrary_null() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |_: &mut Gen| Ipld::Null)
  }

  fn arbitrary_bool() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |g: &mut Gen| Ipld::Bool(Arbitrary::arbitrary(g)))
  }

  fn arbitrary_link() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |g: &mut Gen| Ipld::Link(arbitrary_cid(g)))
  }

  pub fn arbitrary_i128() -> Box<dyn Fn(&mut Gen) -> i128> {
    Box::new(move |g: &mut Gen| i64::arbitrary(g) as i128)
  }

  pub fn arbitrary_integer() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |g: &mut Gen| Ipld::Integer(arbitrary_i128()(g)))
  }

  fn arbitrary_string() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |g: &mut Gen| Ipld::String(Arbitrary::arbitrary(g)))
  }

  fn arbitrary_bytes() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |g: &mut Gen| Ipld::Bytes(Arbitrary::arbitrary(g)))
  }

  pub fn arbitrary_list() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |g: &mut Gen| {
      let mut rng = rand::thread_rng();
      let size = rng.gen_range(0..5);
      Ipld::List((0..size).map(|_| Arbitrary::arbitrary(g)).collect())
    })
  }

  pub fn arbitrary_stringmap() -> Box<dyn Fn(&mut Gen) -> Ipld> {
    Box::new(move |g: &mut Gen| {
      let mut rng = rand::thread_rng();
      let size = rng.gen_range(0..5);
      Ipld::StringMap((0..size).map(|_| Arbitrary::arbitrary(g)).collect())
    })
  }

  impl Arbitrary for Ipld {
    fn arbitrary(g: &mut Gen) -> Self {
      frequency(g, vec![
        (100, arbitrary_null()),
        (100, arbitrary_bool()),
        (100, arbitrary_link()),
        (100, arbitrary_integer()),
        (100, arbitrary_string()),
        (100, arbitrary_bytes()),
        (30, arbitrary_list()),
        (30, arbitrary_stringmap()),
      ])
    }
  }
}
