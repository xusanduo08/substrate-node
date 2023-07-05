#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod weights;
pub use weights::*;

use sp_runtime:: { traits::Zero, offchain:: { storage::StorageValueRef, storage::MutateStorageError } };

use serde::{ self, Deserialize, Deserializer};

use frame_system::{
  offchain::{
    AppCrypto, CreateSignedTransaction, SendSignedTransaction,
    Signer,
  },
};
use frame_support::{inherent::Vec, pallet_prelude::*};
use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
#[derive(Debug, Decode, Encode)]
struct IndexingData {
  name: Vec<u8>,
  number: u64
}
// type IndexingData = (Vec<u8>, u64);
const OFFCHAIN_STORAGE_KEY: &[u8] = b"ocw-demo::storage::Tx";
pub mod crypto {
  use super::KEY_TYPE;
  use sp_core::sr25519::Signature as Sr25519Signature;
  use sp_runtime::{
    app_crypto::{app_crypto, sr25519},
    traits::Verify,
    MultiSignature, MultiSigner,
  };
  app_crypto!(sr25519, KEY_TYPE);

  pub struct OcwAuthId;

  impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
    type RuntimeAppPublic = Public;
    type GenericSignature = sp_core::sr25519::Signature;
    type GenericPublic = sp_core::sr25519::Public;
  }

  impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
    for OcwAuthId
    {
      type RuntimeAppPublic = Public;
      type GenericSignature = sp_core::sr25519::Signature;
      type GenericPublic = sp_core::sr25519::Public;
    }
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::{pallet_prelude::*, ensure_signed};
  use sp_runtime::offchain::http::{self, Request};
  use sp_core::offchain::Duration;
  use sp_std::vec;

  #[derive(Deserialize, Encode, Decode)]
  pub struct GithubInfo {
    #[serde(deserialize_with="de_string_to_bytes")]
    login: Vec<u8>,
    #[serde(deserialize_with="de_string_to_bytes")]
    blog: Vec<u8>,
    public_repos: u32,
  }

  pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>{
      let s: &str = Deserialize::deserialize(de)?;
      Ok(s.as_bytes().to_vec())
  }

  use core::{ convert::TryInto, fmt };
  impl fmt::Debug for GithubInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(
        f,
        "{{ login: {}, blog: {}, public_repos: {} }}",
        sp_std::str::from_utf8(&self.login).map_err(|_| fmt::Error)?,
        sp_std::str::from_utf8(&self.blog).map_err(|_| fmt::Error)?,
        &self.public_repos
        )
    }
  }

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
    
    #[pallet::call_index(2)]
    #[pallet::weight(0)]
    pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {
      let _who = ensure_signed(origin)?;
      log::info!("OCW ==> in submit_data call: {:?}", payload);
      Ok(().into())
    }

    #[pallet::call_index(3)]
    #[pallet::weight(0)]
    pub fn extrinsics(origin: OriginFor<T>, number: u64) -> DispatchResultWithPostInfo {
      let who = ensure_signed(origin)?;
      let key = Self::derive_key(frame_system::Module::<T>::block_number());
      let data = IndexingData{ name: b"submit_number_unsigned".to_vec(), number };
      sp_io::offchain_index::set(OFFCHAIN_STORAGE_KEY, &data.encode()); // 向offchain DB storage中写入数据
      log::info!("====write to offchain storage");
      Ok(().into())
    }
	}

  impl<T: Config> Pallet<T> {
    #[deny(clippy::clone_double_ref)]
    pub fn derive_key(block_number: T::BlockNumber) -> Vec<u8> {
      block_number.using_encoded(|encoded_bn| {
        b"node-template::storage::"
          .iter()
          .chain(encoded_bn)
          .copied()
          .collect::<Vec<u8>>()
      })
    }

    pub fn fetch_github_info() -> Result<GithubInfo, http::Error>{
      let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
      let request = http::Request::get("https://api.github.com/orgs/substrate-developer-hub");
      let pending = request
        .add_header("User-Agent", "Substrate-Offchain-Worker")
        .deadline(deadline).send().map_err(|_| http::Error::IoError)?;

      let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
      if response.code != 200 {
        log::warn!("Unexpected status code: {}", response.code);
        return Err(http::Error::Unknown)
      }

      let body = response.body().collect::<Vec<u8>>();
      let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
        log::warn!("No UTF8 body");
        http::Error::Unknown
      })?;

      let gh_info: GithubInfo = serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;
      Ok(gh_info)
    }
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn offchain_worker(block_number: T::BlockNumber) {
      let key = Self::derive_key(block_number);
      let storage_ref = StorageValueRef::persistent(OFFCHAIN_STORAGE_KEY);

      if let Ok(Some(data)) = storage_ref.get::<IndexingData>() {
        log::info!("local storage data: {:?}, {:?}", sp_std::str::from_utf8(&data.name).unwrap_or("error"), data.number);
      } else {
        log::info!("Error reading from local storage.");
      }
    }
  }
}
