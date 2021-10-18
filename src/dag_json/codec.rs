use crate::Ipld;
use alloc::{
  borrow::ToOwned,
  string::String,
};
use bytecursor::ByteCursor;
use core::convert::TryFrom;
use serde::{
  de,
  de::Error as SerdeError,
  ser,
  Deserialize,
  Serialize,
  Serializer,
};
use serde_json::Error;
use sp_cid::Cid;
use alloc::{
  collections::btree_map::BTreeMap,
  vec::Vec,
};
use core::fmt;

const SPECIAL_KEY: &str = "/";

pub fn encode(ipld: &Ipld, writer: &mut ByteCursor) -> Result<(), Error> {
  let ipld_json = serde_json::to_string(&ipld).unwrap();
  writer.write(ipld_json.as_bytes()).unwrap();
  Ok(())
}

pub fn decode(r: &mut ByteCursor) -> Result<Ipld, Error> {
  let mut de = serde_json::Deserializer::from_slice(r.get_ref());
  deserialize(&mut de)
}

impl Serialize for Ipld {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer {
    match &self {
      Ipld::Null => serializer.serialize_none(),
      Ipld::Bool(bool) => serializer.serialize_bool(*bool),
      Ipld::Integer(i128) => serializer.serialize_i128(*i128),
      Ipld::Float(f64) => serializer.serialize_f64(*f64),
      Ipld::String(string) => serializer.serialize_str(string),
      Ipld::Bytes(bytes) => {
        let value = base64::encode(bytes);
        let mut inner_map = BTreeMap::new();
        inner_map.insert(String::from("bytes"), value);
        let mut map = BTreeMap::new();
        map.insert(SPECIAL_KEY, inner_map);

        serializer.collect_map(map)
      }
      Ipld::List(list) => {
        let wrapped = list.iter().map(|ipld| Wrapper(ipld));
        serializer.collect_seq(wrapped)
      }
      Ipld::StringMap(map) => {
        let wrapped = map.iter().map(|(key, ipld)| (key, Wrapper(ipld)));
        serializer.collect_map(wrapped)
      }
      Ipld::Link(link) => {
        let value = base64::encode(link.to_bytes());
        let mut map = BTreeMap::new();
        map.insert(SPECIAL_KEY, value);

        serializer.collect_map(map)
      }
    }
  }
}

fn serialize<S: ser::Serializer>(
  ipld: &Ipld,
  ser: S,
) -> Result<S::Ok, S::Error> {
  match &ipld {
    Ipld::Null => ser.serialize_none(),
    Ipld::Bool(bool) => ser.serialize_bool(*bool),
    Ipld::Integer(i128) => ser.serialize_i128(*i128),
    Ipld::Float(f64) => ser.serialize_f64(*f64),
    Ipld::String(string) => ser.serialize_str(string),
    Ipld::Bytes(bytes) => {
      let value = base64::encode(bytes);
      let mut inner_map = BTreeMap::new();
      inner_map.insert(String::from("bytes"), value);
      let mut map = BTreeMap::new();
      map.insert(SPECIAL_KEY, inner_map);

      ser.collect_map(map)
    }
    Ipld::List(list) => {
      let wrapped = list.iter().map(|ipld| Wrapper(ipld));
      ser.collect_seq(wrapped)
    }
    Ipld::StringMap(map) => {
      let wrapped = map.iter().map(|(key, ipld)| (key, Wrapper(ipld)));
      ser.collect_map(wrapped)
    }
    Ipld::Link(link) => {
      let value = base64::encode(link.to_bytes());
      let mut map = BTreeMap::new();
      map.insert(SPECIAL_KEY, value);

      ser.collect_map(map)
    }
  }
}

fn deserialize<'de, D: de::Deserializer<'de>>(
  deserializer: D,
) -> Result<Ipld, D::Error> {
  // Sadly such a PhantomData hack is needed
  deserializer.deserialize_any(JsonVisitor)
}

// Needed for `collect_seq` and `collect_map` in Seserializer
struct Wrapper<'a>(&'a Ipld);

impl<'a> Serialize for Wrapper<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: ser::Serializer {
    serialize(self.0, serializer)
  }
}

// serde deserializer visitor that is used by Deseraliazer to decode
// json into IPLD.
struct JsonVisitor;
impl<'de> de::Visitor<'de> for JsonVisitor {
  type Value = Ipld;

  fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt.write_str("any valid JSON value")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where E: de::Error {
    self.visit_string(String::from(value))
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::String(value))
  }

  fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
  where E: de::Error {
    self.visit_byte_buf(v.to_owned())
  }

  fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::Bytes(v))
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::Integer(v.into()))
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::Integer(v.into()))
  }

  fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::Integer(v))
  }

  fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::Bool(v))
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
  where E: de::Error {
    self.visit_unit()
  }

  fn visit_unit<E>(self) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::Null)
  }

  fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
  where V: de::SeqAccess<'de> {
    let mut vec: Vec<WrapperOwned> = Vec::new();

    while let Some(elem) = visitor.next_element()? {
      vec.push(elem);
    }

    let unwrapped = vec.into_iter().map(|WrapperOwned(ipld)| ipld).collect();
    Ok(Ipld::List(unwrapped))
  }

  fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
  where V: de::MapAccess<'de> {
    let mut values: Vec<(String, WrapperOwned)> = Vec::new();

    while let Some((key, value)) = visitor.next_entry()? {
      values.push((key, value));
    }

    // JSON Object represents IPLD Link if it is `{ "/": "...." }` therefor
    // we valiadet if that is the case here.
    if let Some((key, WrapperOwned(Ipld::String(value)))) = values.first() {
      if key == SPECIAL_KEY && values.len() == 1 {
        let link = base64::decode(&value).map_err(SerdeError::custom)?;
        let cid = Cid::try_from(link).map_err(SerdeError::custom)?;
        return Ok(Ipld::Link(cid));
      }
    }

    if let Some((first_key, WrapperOwned(Ipld::StringMap(map)))) =
      values.first()
    {
      if let Some((key, Ipld::String(value))) = map.first_key_value() {
        if first_key == SPECIAL_KEY && key == "bytes" && values.len() == 1 {
          let bytes = base64::decode(value).map_err(SerdeError::custom)?;
          return Ok(Ipld::Bytes(bytes));
        }
      }
    }

    let unwrapped = values
      .into_iter()
      .map(|(key, WrapperOwned(value))| (key, value))
      .collect();
    Ok(Ipld::StringMap(unwrapped))
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where E: de::Error {
    Ok(Ipld::Float(v))
  }
}

// Needed for `visit_seq` and `visit_map` in Deserializer
/// We cannot directly implement `serde::Deserializer` for `Ipld` as it is a
/// remote type. Instead wrap it into a newtype struct and implement
/// `serde::Deserialize` for that one. All the deserializer does is calling the
/// `deserialize()` function we defined which returns an unwrapped `Ipld`
/// instance. Wrap that `Ipld` instance in `Wrapper` and return it.
/// Users of this wrapper will then unwrap it again so that they can return the
/// expected `Ipld` instance.
struct WrapperOwned(Ipld);

impl<'de> Deserialize<'de> for WrapperOwned {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: de::Deserializer<'de> {
    let deserialized = deserialize(deserializer);
    // Better version of Ok(Wrapper(deserialized.unwrap()))
    deserialized.map(Self)
  }
}
