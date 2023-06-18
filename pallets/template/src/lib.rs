#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use sp_runtime:: { traits::Zero, offchain:: { storage::StorageValueRef, storage::MutateStorageError } };

use serde::{ self, Deserialize, Deserializer};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
  use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*};
use sp_runtime::offchain::http::{self, Request};
  use sp_core::offchain::Duration;

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
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

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
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
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
    // 奇数写入数据，偶数读取数据
    // fn offchain_worker(block_number: T::BlockNumber) {
    //   log::info!("OCW==> Hello world from offchain workers!: {:?}", block_number);
      
    //   if block_number % 2u32.into() != Zero::zero() {
      
    //     let key = Self::derive_key(block_number);
    //     let val_ref = StorageValueRef::persistent(&key);

    //     let random_slice = sp_io::offchain::random_seed();

    //     let time_stamp_u64 = sp_io::offchain::timestamp().unix_millis();

    //     let value = (random_slice, time_stamp_u64);

    //     struct StateError;
    //     // 原子化修改
    //     let res = val_ref.mutate(|val: Result<Option<([u8; 32], u64)>, sp_runtime::offchain::storage::StorageRetrievalError>| -> Result<_, StateError> {
    //       match val {
    //         Ok(Some(_)) => Ok(value),
    //         _ => Ok(value),
    //       }
    //     });

    //     match res {
    //       Ok(value) => log::info!("OCW ==> in odd block, mutate value successful: {:?}", value),
    //       Err(MutateStorageError::ValueFunctionFailed(_)) => (),
    //       Err(MutateStorageError::ConcurrentModification(_)) => (),
    //     }
    //   } else {
    //     // block_number是逐个增加的，-1就能获取到上个block_numbers
    //     let key = Self::derive_key(block_number - 1u32.into());
    //     let mut val_ref = StorageValueRef::persistent(&key);

    //     if let Ok(Some(value)) = val_ref.get::<([u8; 32], u64)>() {
    //       log::info!("OCW ==> in even block, value read: {:?}", value);
    //       val_ref.clear();
    //     }
    //   }
    //   log::info!("OCW==> Leave from offchain workers!: {:?}", block_number);
    // }

    fn offchain_worker(block_number: T::BlockNumber) {
      log::info!("OCW ==> Hello World from offchain workers!: {:?}", block_number);
      if let Ok(info) = Self::fetch_github_info() {
        log::info!("OCW ==> GithubInfo: {:?}", info);
      } else {
        log::info!("OCW ==> Error while fetch github info!");
      }

      log::info!("OCW ==> Leave from offchian workers!: {:?}", block_number);
    }
  }
}
