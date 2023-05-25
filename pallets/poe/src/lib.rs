#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, ensure_signed};
  use sp_std::prelude::*;


	#[pallet::config] // 模块配置
	pub trait Config: frame_system::Config {
    #[pallet::constant]
		type MaxClaimLength: Get<u32>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}


	#[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

  #[pallet::storage]
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    ClaimCreated(T::AccountId, Vec<u8>),
    ClaimRevoked(T::AccountId, Vec<u8>),
  }

  #[pallet::error]
  pub enum Error<T> {
    ProofAlreadyExist,
    ClaimTooLong,
    ClaimNotExist,
    NotClaimOwner,
  }


  #[pallet::call]
  impl<T: Config> Pallet<T> {

    // 创建存证
    #[pallet::weight(0)]
    #[pallet::call_index(0)]
    pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult { // origin 交易的发送方，claim 存证
      // 校验交易的发送方
      let sender = ensure_signed(origin)?;
      // 校验存证是否超过我们允许的最大长度
      // 将claim转换成BoundedVec类型
      let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;

      // 校验要创建的存证现在还不存在，如果存在则返回ProofAlreadyExist错误
      ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

      // 插入存证
      Proofs::<T>::insert(&bounded_claim, (sender.clone(), frame_system::Pallet::<T>::block_number()));

      // 触发事件: 存证被创建
      Self::deposit_event(Event::ClaimCreated(sender, claim));

      Ok(())
    }

    // 销毁存证
    #[pallet::weight(1)]
    pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
      let sender = ensure_signed(origin)?;
      let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
      let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?; // 获取到这个claim的owner

      // 校验交易的发送方和claim的owner一致
      ensure!(owner == sender, Error::<T>::NotClaimOwner);

      Proofs::<T>::remove(&bounded_claim);
      Self::deposit_event(Event::ClaimRevoked(sender, claim));
      Ok(().into())
    }

    // 转移存证
    #[pallet::weight(2)]
    pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResult {
      // 验证发送方
      let sender = ensure_signed(origin)?;

      let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;

      let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;
      ensure!(owner == sender, Error::<T>::NotClaimOwner);

      Proofs::<T>::insert(bounded_claim, (dest, frame_system::Pallet::<T>::block_number()));
      Ok(().into())
    }
  }
}
