# How to Use `sp-ipld` within Substrate

1. Clone the [Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template)
2. Navigate to `pallets/template/Cargo.toml` and import `sp-ipld` and `sp-std` as follows:
   ```
   [dependencies.sp-std]
   default-features = false
   version = '3.0.0'
   
   [dependencies.sp-ipld]
   default-features = false
   features = ["dag-cbor"]
   git = 'https://github.com/yatima-inc/sp-ipld'
   version = '0.1'
   ```
3. Navigate to `pallets/template/src/lib.rs` and make the following changes:\
   Below `use frame_system::pallet_prelude::*;` add 
   ```
   use sp_std::vec::Vec;
   use sp_ipld::{dag_cbor, Ipld};
   ```
   Change the following:\
   `pub type Something<T> = StorageValue<_, u32>;` to\
   `pub(super) type Cid<T> = StorageValue<_, Vec<u8>>;`\
   `SomethingStored(u32, T::AccountId),` to
   ```
   CidStored(T::AccountId, Vec<u8>),
   CidRetrieved(T::AccountId, Vec<u8>),
   ```
   Replace the `#[pallet:;call]` section with:
   ```
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn store_ipld(origin: OriginFor<T>, input: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

      //Construct CID from input
      let cid: Vec<u8> = dag_cbor::cid(&Ipld::Integer(input as i128)).to_bytes();

      runtime_print!("Encoded input: {} into dag-cbor CID: {:?}", input, cid);
      runtime_print!("Request sent by: {:?}", who);

			// Insert into storage.
			<Cid<T>>::put(cid.clone());

			// Emit an event.
			Self::deposit_event(Event::CidStored(who, cid));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn retrieve_ipld(origin: OriginFor<T>, cid: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

        // Retrieve from storage
        //let data = <Cid<T>>::get(cid);
      //Retrieve data from CID
        //let data: u32 = match DagCborCodec.decode(ByteCursor::new(cid.clone())).expect("invalid ipld cbor.") {
        //    Ipld::Integer(uint) => uint as u32,
        //    _ => 0 as u32,
        //};
      let data = 5;

      runtime_print!("Decoded data: {} from dag-cbor CID: {:?}", data, cid);
      runtime_print!("Request sent by: {:?}", who);

			// Emit an event.
			Self::deposit_event(Event::CidRetrieved(who, cid));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Cid<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(_) => {
					Ok(())
				},
			}
		}
	}
   ```

4. Build and run the node
   ```
   cargo build --release
   # Run a temporary node in development mode
   ./target/release/node-template --dev -lruntime=debug 
   ```
   Stop the node for now after making sure it works.

5. Clone the [Substrate Front End Template](https://github.com/substrate-developer-hub/substrate-front-end-template.git) in a separate directory.

6. Navigate to `src/TemplateModule.js` and replace it with the following:
   ```
   import React, { useEffect, useState } from 'react';
   import { Form, Input, Grid, Statistic } from 'semantic-ui-react';
   
   import { useSubstrate } from './substrate-lib';
   import { TxButton } from './substrate-lib/components';
   
   export function Main (props) {
     const { api } = useSubstrate();
     const { accountPair } = props;
   
     // The transaction submission status
     const [status, setStatus] = useState('');
   
     // The currently stored values
     const [cid, setCid] = useState(0);
     const [data, setData] = useState(0);
   
     const [currentValue, setCurrentValue] = useState('');
     const [formValue, setFormValue] = useState('');
   
     useEffect(() => {
       let unsubscribe;
         api.query.templateModule.cid(newValue => {
         // The storage value is an Option
         // So we have to check whether it is None first
         // There is also unwrapOr
             if (newValue.isNone) {
                 setCurrentValue('<None>');
             } else {
                 setCurrentValue(newValue.unwrap().toString());
                 let newVal = newValue.unwrap();
                 setCid(newVal[0].toString());
                 setData(newVal[1].toNumber());
             }
         }).then(unsub => {
             unsubscribe = unsub;
       })
             .catch(console.error);
   
         return () => unsubscribe && unsubscribe();
     }, [api.query.templateModule]);
   
     return (
       <Grid.Column width={20} style={{ textAlign: 'center' }}>
         <h1>IPLD Storage</h1>
           <Statistic
             label='Currently Stored Data'
             value={data}
             size='mini'
           />
           <Statistic
             label='Currently Stored CID'
             value={cid}
             size='mini'
           />
         <Form>
           <Form.Field>
             <Input
               label='IPLD Input'
               type='string'
               onChange={(_, { value }) => setFormValue(value)}
             />
           </Form.Field>
           <Form.Field style={{ textAlign: 'center' }}>
             <TxButton
               accountPair={accountPair}
               label='Store IPLD data'
               type='SIGNED-TX'
               setStatus={setStatus}
               attrs={{
                 palletRpc: 'templateModule',
                 callable: 'storeIpld',
                 inputParams: [formValue],
                 paramFields: [true]
               }}
             />
             <TxButton
               accountPair={accountPair}
               label='Get IPLD data'
               type='SIGNED-TX'
               setStatus={setStatus}
               attrs={{
                 palletRpc: 'templateModule',
                 callable: 'retrieveIpld',
                 inputParams: [formValue],
                 paramFields: [true]
               }}
             />
           </Form.Field>
           <div style={{ overflowWrap: 'break-word' }}>{status}</div>
         </Form>
       </Grid.Column>
     );
   }
   
   export default function TemplateModule (props) {
     const { api } = useSubstrate();
     return api.query.templateModule && api.query.templateModule.cid
       ? <Main {...props} />
       : null;
   }
   ```

7. Build and install the front end as detailed in the [Readme](https://github.com/substrate-developer-hub/substrate-front-end-template/blob/master/README.md)\
   Run the node as done in Step 4, then run the front end with `yarn start`\
   In the browser, scroll to the bottom of the page and input an integer, then hit "Store IPLD Data". The encoded CID should show up on screen as well as in the node's stdout.\
   To retrieve this data, enter the "Currently Stored CID" (omitting the "0X") and hit "Retrieve IPLD Data". The integer originally entered should then appear in the event log and in stdout.
   
## Working example of this tutorial
See the Yatima (substrate-node-template)[https://github.com/yatima-inc/substrate-node-template] and
(substrate-front-end-template)[https://github.com/yatima-inc/substrate-front-end-template] repos on the `ipld-tutorial` branch
