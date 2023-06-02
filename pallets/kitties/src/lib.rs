#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

#[frame_support::pallet]
pub mod pallet {
	pub use frame_support::pallet_prelude::*;
	use frame_support::Blake2_128Concat;
	pub use frame_system::pallet_prelude::*;

	use frame_support::traits::Randomness;
	use frame_system::{ensure_signed, pallet_prelude::OriginFor};
	use sp_io::hashing::blake2_128;

	pub type KittyId = u32;
	#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::config] // 模块配置
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	#[pallet::error]
	pub enum Error<T> {
		StorageOverflow,
		InvalidKittyId,
		SameKittyId,
		KittyNotExist,
		NotOwner,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event )]
	pub enum Event<T: Config> {
		KittyCreated { sender: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransfered { sender: T::AccountId, to: T::AccountId, kitty_id: KittyId },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[pallet::call_index(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty(Self::random_value(&sender));

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &sender);

			Self::deposit_event(Event::KittyCreated { sender, kitty_id, kitty });
			Ok(())
		}

		#[pallet::weight(1)]
		#[pallet::call_index(1)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id1: KittyId,
			kitty_id2: KittyId,
		) -> DispatchResult {
			// 繁殖
			let sender = ensure_signed(origin)?;
      // 要求两个kittyid是不一样的
			ensure!(kitty_id1 != kitty_id2, Error::<T>::SameKittyId);
			
			// 确定是合法的kittyId
			ensure!(Kitties::<T>::contains_key(kitty_id1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?; // 生成新kitty的id
			let kitty1 = Self::kitties(kitty_id1).ok_or(Error::<T>::KittyNotExist)?;
			let kitty2 = Self::kitties(kitty_id2).ok_or(Error::<T>::KittyNotExist)?;

			let selector = Self::random_value(&sender);
			let mut data = [0u8; 16];
			for i in 0..kitty1.0.len() {
				data[i] = (kitty1.0[i] & selector[i]) | (kitty2.0[i] & selector[i]);
			}

			let kitty = Kitty(data);
			// 将kitty放入kitties中
			Kitties::<T>::insert(kitty_id, &kitty);
			// 更新kittyOwner
			KittyOwner::<T>::insert(kitty_id, &sender);
			// 更新parent信息
			KittyParents::<T>::insert(kitty_id, (kitty_id1, kitty_id2));

			Self::deposit_event(Event::KittyCreated { sender, kitty_id, kitty });

			Ok(())
		}

		#[pallet::weight(2)]
		#[pallet::call_index(2)]
		pub fn transfer(
			sender: OriginFor<T>,
			to: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;
			// kittyid存在
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
			// kitty的owner是当前发起方
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(sender == owner, Error::<T>::NotOwner);

			KittyOwner::<T>::insert(kitty_id, &to);
			Self::deposit_event(Event::KittyTransfered { sender, to, kitty_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id.checked_add(1).ok_or(Error::<T>::InvalidKittyId)?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}
	}
}
