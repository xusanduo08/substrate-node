#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

// token质押、transfer
// 创建kitty时需要质押一定数量的token

#[frame_support::pallet]
pub mod pallet {
	pub use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, ExistenceRequirement, StorageVersion, Randomness};
	use frame_support::Blake2_128Concat;
	use frame_support::PalletId;
	pub use frame_system::pallet_prelude::*;
	use frame_system::{ensure_signed, pallet_prelude::OriginFor};
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::AccountIdConversion;

  use crate::migrations;

	pub type KittyId = u32;
	#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	pub struct Kitty{
    pub dna: [u8; 16],
    pub name: [u8; 8],
  }

  const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	// 注意这里balance的type的定义
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config] // 模块配置
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>; // kitty的价格

		type Currency: Currency<Self::AccountId>;

		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
  #[pallet::storage_version(STORAGE_VERSION)] // 给pallet增加version属性
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_on_sale)]
	pub type KittiesOnSale<T> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

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
		NotOwner,      // 不是所有者
		AlreadyOnSale, // 在售
		NotOnSale,     // 没有在售
		AlreadyOwned,  // 已经拥有
		NotOwned,      // 没有所有者
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event )]
	pub enum Event<T: Config> {
		KittyCreated { sender: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransfered { sender: T::AccountId, to: T::AccountId, kitty_id: KittyId },
		KittyOnSale { sender: T::AccountId, kitty_id: KittyId },
	}

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_runtime_upgrade() -> Weight {
      migrations::v2::migrate::<T>();
      Weight::zero()
    }
  }

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[pallet::call_index(0)]
		pub fn create(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty{ dna: Self::random_value(&sender), name };

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&sender, price)?; // 质押price数量的token
			T::Currency::transfer(
				&sender,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

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
      name: [u8; 8]
		) -> DispatchResult {
			// 繁殖
			let sender = ensure_signed(origin)?;
			// 要求两个kittyid是不一样的
			ensure!(kitty_id1 != kitty_id2, Error::<T>::SameKittyId);

			// 确定是合法的kittyId
			ensure!(Kitties::<T>::contains_key(kitty_id1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id2), Error::<T>::InvalidKittyId);

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&sender, price)?;
			T::Currency::transfer(
				&sender,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			let kitty_id = Self::get_next_id()?; // 生成新kitty的id
			let kitty1 = Self::kitties(kitty_id1).ok_or(Error::<T>::KittyNotExist)?;
			let kitty2 = Self::kitties(kitty_id2).ok_or(Error::<T>::KittyNotExist)?;

			let selector = Self::random_value(&sender);
			let mut data = [0u8; 16];
			for i in 0..kitty1.dna.len() {
				data[i] = (kitty1.dna[i] & selector[i]) | (kitty2.dna[i] & selector[i]);
			}

			let kitty = Kitty{ dna: data, name };
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

		#[pallet::weight(4)]
		#[pallet::call_index(4)]
		pub fn sale(sender: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let sender = ensure_signed(sender)?;
			Self::kitties(kitty_id).ok_or(Error::<T>::KittyNotExist)?;
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;

			ensure!(sender == owner, Error::<T>::NotOwner);

			ensure!(Self::kitties_on_sale(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			KittiesOnSale::<T>::insert(kitty_id, ());
			Self::deposit_event(Event::KittyOnSale { sender, kitty_id });
			Ok(())
		}

		#[pallet::weight(5)]
		#[pallet::call_index(5)]
		pub fn buy(sender: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			// 购买
			// 验证签名
			let sender = ensure_signed(sender)?;
			// 验证存在这个kitty
			Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			// 验证sender不是owner
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::NotOwned)?; // 没有owner
			ensure!(sender != owner, Error::<T>::AlreadyOwned);
			// 验证kitty是否在售卖
			Self::kitties_on_sale(kitty_id).ok_or(Error::<T>::NotOnSale)?;
			// 转移sender price数量的token到owner
			let price = T::KittyPrice::get();
			// T::Currency::reserve(&sender, price)?;
			// T::Currency::unreserve(&owner, price);
			T::Currency::transfer(&sender, &owner, price, ExistenceRequirement::KeepAlive)?;
			// 更新kittyOwner数据
			<KittyOwner<T>>::insert(kitty_id, &sender);
			// 删除kitty on sale中的数据
			<KittiesOnSale<T>>::remove(kitty_id);
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

		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
