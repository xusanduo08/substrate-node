use crate::*;
use frame_support::{migration::storage_key_iter, Blake2_128Concat};
use frame_support::{
	pallet_prelude::*, storage::StoragePrefixedMap, traits::GetStorageVersion, weights::Weight,
};

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if on_chain_version == 0 {
    return v0_v2::<T>();
	}

  if on_chain_version == 1 {
    return v1_v2::<T>();
  }

	Weight::zero()
}

#[derive(Encode, Decode, Clone, Debug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct v0_Kitty(pub [u8; 16]);

#[derive(Encode, Decode, Clone, Debug, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub struct v1_Kitty{ pub dna: [u8; 16], pub name: [u8; 4] }

pub fn v0_v2<T: Config>() -> Weight {
  let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if on_chain_version != 0 {
		return Weight::zero();
	}

	if current_version != 2 {
		return Weight::zero();
	}
  let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (index, kitty) in storage_key_iter::<KittyId, v0_Kitty, Blake2_128Concat>(module, item).drain() {

		let newKitty = Kitty {
			// 将oldKitty的数据移植到new kitty上
			dna: kitty.0,
			name: *b"abcdefgh",
		};
    Kitties::<T>::insert(index, newKitty);
	}

	Weight::zero()
}

pub fn v1_v2<T: Config>() -> Weight {
  let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if on_chain_version != 1 {
		return Weight::zero();
	}

	if current_version != 2 {
		return Weight::zero();
	}

  let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

  for (index, kitty) in storage_key_iter::<KittyId, v1_Kitty, Blake2_128Concat>(module, item).drain() {

		let newKitty = Kitty {
			dna: kitty.dna,
			name: *b"abcdefgh",
		};
    Kitties::<T>::insert(index, newKitty);
	}

	Weight::zero()
}