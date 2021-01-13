#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, StorageValue, StorageMap, traits::Randomness, sp_std, Parameter};
use frame_system::{ensure_signed};
use sp_runtime::{traits::Bounded, DispatchError};
use sp_io::hashing::blake2_128;
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub struct Kitty(
	pub [u8; 16]
);

pub trait Trait: frame_system::Trait {

	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type Randomness: Randomness<Self::Hash>;

	type KittyIndex : Parameter + AtLeast32BitUnsigned + Default + Bounded + Copy;
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
		pub KittiesCount get(fn kitties_count): T::KittyIndex;
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;

		// 用户对应的Kitty列表
		pub KittiesList get(fn kitty_list): map hasher(blake2_128_concat) T::AccountId => Vec<T::KittyIndex>;
		pub KittiesParent get(fn kitty_parent): map hasher(blake2_128_concat) T::KittyIndex => (T::KittyIndex, T::KittyIndex);
		pub KittiesChildren get(fn kitty_children) : map hasher(blake2_128_concat) (T::KittyIndex, T::KittyIndex) => Vec<T::KittyIndex>;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverflow,
		NotKittyOwner,
		InvalidKittyId,
		RequireDifferentParent,
	}
}

decl_event!(
	pub enum Event<T>
	where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::KittyIndex,
	{
		Created(AccountId, KittyIndex),
		Transfered(AccountId, AccountId, KittyIndex),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;
			let dna = Self::random_value(&sender);
			let kitty = Kitty(dna);

			Self::create_kitty(&sender, kitty_id.clone(), kitty);
			Self::insert_kitty(&sender, kitty_id.clone());
			Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}

		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			////////////////////////////////////////////////////////////////////////////////////////////////
			// Q1.转移前未验证对应的Kitty的原Owner是否是origin地址,修复代码如下
			////////////////////////////////////////////////////////////////////////////////////////////////
			ensure!(<KittyOwners<T>>::get(kitty_id) == Some(sender.clone()), Error::<T>::NotKittyOwner);
			////////////////////////////////////////////////////////////////////////////////////////////////

			Self::remove_kitty(&sender, kitty_id.clone());
			Self::insert_kitty(&to.clone(), kitty_id.clone());
			Self::deposit_event(RawEvent::Transfered(sender, to, kitty_id));
		}

		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;
			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}
	}
}

impl<T:Trait> Module<T> {

	fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
	}

	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
		);

		payload.using_encoded(blake2_128)
	}

	fn create_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		<Kitties<T>>::insert(kitty_id, kitty);
		<KittiesCount<T>>::put(kitty_id + 1.into());
	}

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
		<KittyOwners<T>>::insert(kitty_id, owner);
		Self::kitty_list(owner).push(kitty_id);
	}

	fn remove_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
		for i in 0..Self::kitty_list(owner).len() {
			if Self::kitty_list(owner)[i] == kitty_id {
				Self::kitty_list(owner).remove(i);
				break;
			}
		}
	}

	fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
		(selector & dna1) | (!selector & dna2)
	}

	// 这是一只流氓猫，想和谁生就可以和谁生 !!!
	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {

		ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

		let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

		let kitty_id = Self::next_kitty_id()?;
		let kitty1_dna = kitty1.0;
		let kitty2_dna = kitty2.0;
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		for i in 0..kitty1_dna.len() {
			// 写为Self::combine_dna可读性更强
			new_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}
		Self::create_kitty(sender, kitty_id.clone(), Kitty(new_dna));
		Self::insert_kitty(sender, kitty_id);

		Self::kitty_children( (kitty_id_1, kitty_id_2) ).push(kitty_id);

		<KittiesParent<T>>::insert(kitty_id, (kitty_id_1, kitty_id_2));
		Self::kitty_children((kitty_id_1, kitty_id_2)).push(kitty_id);

		Ok(kitty_id)
	}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;