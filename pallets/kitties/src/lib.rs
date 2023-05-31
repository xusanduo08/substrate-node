#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::Blake2_128Concat;
  pub use frame_support::pallet_prelude::*;
  pub use frame_system::pallet_prelude::*;


  pub type KittyId = u32;
  #[derive(Encode, Decode,Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
  pub struct Kitty(pub [u8; 16]);

  #[pallet::config] // 模块配置
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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
  pub type KittyOwner<T: Config> = StorageMap<_,Blake2_128Concat, KittyId, T::AccountId>;

  #[pallet::error]
  pub enum Error<T> {
    StorageOverflow,
    InvalidKittyId,
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event )]
  pub enum Event<T: Config> {
    KittyCreated{sender: T::AccountId, kitty_id: KittyId, kitty: Kitty}
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::weight(0)]
    #[pallet::call_index(0)]
    pub fn create(origin: OriginFor<T>) -> DispatchResult {
      let sender = ensure_signed(origin)?;
      let kitty_id = Self::get_next_id()?;
      let kitty = Kitty(Default::default());

      Kitties::<T>::insert(kitty_id, &kitty);
      KittyOwner::<T>::insert(kitty_id, &sender);

      Self::deposit_event(Event::KittyCreated{ sender, kitty_id, kitty });
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
  }
}