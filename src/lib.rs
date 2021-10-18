#![feature(map_first_last)]
#![deny(
  clippy::missing_errors_doc,
  clippy::missing_panics_doc,
  clippy::missing_safety_doc
)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[macro_use]
extern crate alloc;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
#[cfg(test)]
extern crate rand;

pub mod codec;
#[cfg(feature = "dag-cbor")]
pub mod dag_cbor;
#[cfg(feature = "dag-json")]
pub mod dag_json;
pub mod ipld;

pub use codec::*;
pub use ipld::*;

#[cfg(test)]
pub mod tests {
  use super::*;
  #[cfg(feature = "dag-cbor")]
  use super::{
    dag_cbor,
    dag_cbor::DagCborCodec,
  };
  use bytecursor::ByteCursor;
  use quickcheck::quickcheck;
  use reqwest::multipart;
  use tokio::runtime::Runtime;

  #[cfg(feature = "dag-cbor")]
  async fn dag_put_cbor(dag: Ipld) -> Result<String, reqwest::Error> {
    let host = "http://127.0.0.1:5001";
    let url = format!(
      "{}{}?{}",
      host,
      "/api/v0/dag/put",
      "format=dag-cbor&pin=true&input-enc=cbor&hash=blake2b-256"
    );
    let cbor = DagCborCodec.encode(&dag).unwrap().into_inner();
    let client = reqwest::Client::new();
    let form =
      multipart::Form::new().part("file", multipart::Part::bytes(cbor));
    let response: serde_json::Value =
      client.post(url).multipart(form).send().await?.json().await?;
    println!("PUT response: {:?}", response);

    let ipfs_cid: String = response["Cid"]["/"].as_str().unwrap().to_string();
    let local_cid: String = dag_cbor::cid(&dag).to_string();

    if ipfs_cid == local_cid {
      Ok(ipfs_cid)
    }
    else {
      panic!("CIDs are different {} != {}", ipfs_cid, local_cid);
    }
  }

  // #[cfg(feature = "dag-json")]
  // pub async fn dag_put_json(dag: Ipld) -> Result<String, reqwest::Error> {
  // let host = "http://127.0.0.1:5001";
  // let url = format!(
  // "{}{}?{}",
  // host,
  // "/api/v0/dag/put",
  // "format=dag-cbor&pin=true&input-enc=json&hash=blake2b-256"
  // );
  // let cbor = DagJsonCodec.encode(&dag).unwrap().into_inner();
  // let client = reqwest::Client::new();
  // let form =
  // multipart::Form::new().part("file", multipart::Part::bytes(cbor));
  // let response: serde_json::Value =
  // client.post(url).multipart(form).send().await?.json().await?;
  // println!("response: {:?}", response);
  //
  // let ipfs_cid = response["Cid"]["/"].as_str().unwrap().to_string();
  //
  // assert_eq!(ipfs_cid, dag_cbor::cid(&dag).to_string());
  //
  // Ok(ipfs_cid)
  // }

  #[cfg(feature = "dag-cbor")]
  async fn dag_get_cbor(cid: String) -> Result<Ipld, reqwest::Error> {
    let host = "http://127.0.0.1:5001";
    let url =
      format!("{}{}?arg={}", host, "/api/v0/block/get", cid.to_string());
    let client = reqwest::Client::new();
    let response = client.post(url).send().await?.bytes().await?;
    let response = response.to_vec();
    println!("GET response: {:?}", response);
    let ipld = DagCborCodec
      .decode(ByteCursor::new(response))
      .expect("invalid ipld cbor.");
    println!("ipld: {:?}", ipld);

    Ok(ipld)
  }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model
  // #[cfg(feature = "dag-json")]
  // pub async fn dag_get_json(cid: String) -> Result<Ipld, reqwest::Error> {
  // let host = "http://127.0.0.1:5001";
  // let url =
  // format!("{}{}?arg={}", host, "/api/v0/dag/get", cid.to_string());
  // let client = reqwest::Client::new();
  // let response = client.post(url).send().await?.bytes().await?;
  // let response = response.to_vec();
  // println!("response: {:?}", response);
  // let ipld = DagJsonCodec
  // .decode(ByteCursor::new(response))
  // .expect("invalid ipld json.");
  // println!("ipld: {:?}", ipld);
  //
  // Ok(ipld)
  // }

  #[cfg(feature = "dag-cbor")]
  async fn async_ipld_ipfs_cbor(ipld: Ipld) -> bool {
    match dag_put_cbor(ipld.clone()).await {
      Ok(cid) => match dag_get_cbor(cid.clone()).await {
        Ok(new_ipld) => {
          if ipld.clone() == new_ipld {
            true
          }
          else {
            println!("Cid: {}", cid);
            println!("Encoded ipld: {:?}", ipld);
            println!("Decoded ipld: {:?}", new_ipld);
            false
          }
        }
        Err(e) => {
          println!("Error during `dag_get`: {}", e);
          false
        }
      },
      Err(e) => {
        println!("Error during `dag_put`: {}", e);
        false
      }
    }
  }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model
  // #[cfg(feature = "dag-json")]
  // async fn async_ipld_ipfs_json(ipld: Ipld) -> bool {
  // match dag_put_json(ipld.clone()).await {
  // Ok(cid) => match dag_get_json(cid.clone()).await {
  // Ok(new_ipld) => {
  // if ipld.clone() == new_ipld {
  // true
  // }
  // else {
  // println!("Cid: {}", cid);
  // println!("Encoded ipld: {:?}", ipld);
  // println!("Decoded ipld: {:?}", new_ipld);
  // false
  // }
  // }
  // Err(e) => {
  // println!("Error during `dag_get`: {}", e);
  // false
  // }
  // },
  // Err(e) => {
  // println!("Error during `dag_put`: {}", e);
  // false
  // }
  // }
  // }

  #[cfg(feature = "dag-cbor")]
  fn ipld_ipfs_cbor(ipld: Ipld) -> bool {
    match Runtime::new() {
      Ok(runtime) => runtime.block_on(async_ipld_ipfs_cbor(ipld)),
      Err(e) => {
        println!("Error creating runtime: {}", e);
        false
      }
    }
  }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model
  // #[cfg(feature = "dag-json")]
  // fn ipld_ipfs_json(ipld: Ipld) -> bool {
  // match Runtime::new() {
  // Ok(runtime) => runtime.block_on(async_ipld_ipfs_json(ipld)),
  // Err(e) => {
  // println!("Error creating runtime: {}", e);
  // false
  // }
  // }
  // }

  #[cfg(feature = "dag-cbor")]
  #[ignore]
  #[quickcheck]
  fn null_ipfs_cbor() -> bool { ipld_ipfs_cbor(Ipld::Null) }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model
  // #[cfg(feature = "dag-json")]
  // #[ignore]
  // #[quickcheck]
  // fn null_ipfs_json() -> bool { ipld_ipfs_json(Ipld::Null) }

  #[cfg(feature = "dag-cbor")]
  #[ignore]
  #[quickcheck]
  fn bool_ipfs_cbor(b: bool) -> bool { ipld_ipfs_cbor(Ipld::Bool(b)) }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model
  // #[cfg(feature = "dag-json")]
  // #[ignore]
  // #[quickcheck]
  // fn bool_ipfs_json(b: bool) -> bool { ipld_ipfs_json(Ipld::Bool(b)) }

  #[cfg(feature = "dag-cbor")]
  #[ignore]
  #[quickcheck]
  fn string_ipfs_cbor(x: String) -> bool { ipld_ipfs_cbor(Ipld::String(x)) }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model
  // #[cfg(feature = "dag-json")]
  // #[ignore]
  // #[quickcheck]
  // fn string_ipfs_json(x: String) -> bool { ipld_ipfs_json(Ipld::String(x)) }

  #[cfg(feature = "dag-cbor")]
  #[ignore]
  #[test]
  fn integers_ipfs_cbor() {
    assert!(ipld_ipfs_cbor(Ipld::Integer(0i128)));
    assert!(ipld_ipfs_cbor(Ipld::Integer(1i128)));
    assert!(ipld_ipfs_cbor(Ipld::Integer(i128::from(i64::MAX))));
    assert!(ipld_ipfs_cbor(Ipld::Integer(i128::from(i64::MIN))));
  }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model, so this test fails
  // #[cfg(feature = "dag-json")]
  // #[ignore]
  // #[test]
  // fn integers_ipfs_json() {
  // assert!(ipld_ipfs_json(Ipld::Integer(0i128)));
  // assert!(ipld_ipfs_json(Ipld::Integer(1i128)));
  // assert!(ipld_ipfs_json(Ipld::Integer(i128::from(i64::MAX))));
  // assert!(ipld_ipfs_json(Ipld::Integer(i128::from(i64::MIN))));
  // }

  #[cfg(feature = "dag-cbor")]
  #[ignore]
  #[quickcheck]
  fn integer_ipfs_cbor(x: i64) -> bool {
    ipld_ipfs_cbor(Ipld::Integer(i128::from(x)))
  }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model, so this test fails
  // #[cfg(feature = "dag-json")]
  // #[ignore]
  // #[quickcheck]
  // fn integer_ipfs_json(x: i64) -> bool {
  // ipld_ipfs_json(Ipld::Integer(i128::from(x))) }

  #[cfg(feature = "dag-cbor")]
  #[ignore]
  #[quickcheck]
  fn ipfs_cbor(x: Ipld) -> bool { ipld_ipfs_cbor(x) }

  // "input-enc=json" coerces numbers to floats and does not obey the ipld data
  // model, so this test fails
  // #[cfg(feature = "dag-json")]
  // #[ignore]
  // #[quickcheck]
  // fn ipfs_json(x: Ipld) -> bool { ipld_ipfs_json(x) }
}
