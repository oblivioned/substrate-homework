#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PoeModule {
		Proofs get(fn proofs): map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		ClaimCreated(AccountId, Vec<u8>),
		ClaimRevoked(AccountId, Vec<u8>),
		ClaimTransfer(AccountId, AccountId, Vec<u8>),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		ProofAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		InvalidClaim,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 0]
		// #[weight = FunctionOf(|args:(&Vec<u8>| args.0 * 10, DispatchClass::Normal, Pays::Yes))]
		pub fn create_claim(origin, claim: Vec<u8>) {
			let sender = ensure_signed(origin)?;

			// 长度检测要求claim的长度在[1-10]之间
			ensure!(claim.len() > 0 && claim.len() <= 10, Error::<T>::InvalidClaim);

			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

			Proofs::<T>::insert(&claim, (sender.clone(), frame_system::Module::<T>::block_number()));

			Self::deposit_event(RawEvent::ClaimCreated(sender, claim));
		}

        #[weight = 0]
        fn revoke_claim(origin, proof: Vec<u8>) {

            let sender = ensure_signed(origin)?;

            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::ClaimNotExist);

            let (owner, _) = Proofs::<T>::get(&proof);

            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            Proofs::<T>::remove(&proof);

            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
		}

		#[weight = 0]
		fn transfer_claim(origin, dest: T::AccountId, claim: Vec<u8>) {
			let sender = ensure_signed(origin)?;

			// 1.存证是否存在
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ClaimNotExist);

			// 2.sender是否为存在的拥有者
			let (owner, _block_number) = Proofs::<T>::get(&claim);
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 3.转移存证
			Proofs::<T>::insert(&claim, (dest.clone(), frame_system::Module::<T>::block_number()));

			// 4.抛出事件
			Self::deposit_event(RawEvent::ClaimTransfer(sender, dest, claim));
		}
	}
}
