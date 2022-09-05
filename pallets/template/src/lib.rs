#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_core::U256;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type Name: Get<&'static str>;

		#[pallet::constant]
		type Symbol: Get<&'static str>;

		#[pallet::constant]
		type Decimal: Get<u8>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Transfer { from: T::AccountId, to: T::AccountId, value: U256 },
		Approval { owner: T::AccountId, spender: T::AccountId, value: U256 }
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
		InsufficientAllowance,
	}

	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub type TotalSupply<T> = StorageValue<_, U256>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub type Balance<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, U256>;

	#[pallet::storage]
	#[pallet::getter(fn allowance)]
	pub type Allowance<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, T::AccountId, U256>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, value: U256, ) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::transfer_impl(owner, to, value)
		}

		#[pallet::weight(0)]
		pub fn approve(origin: OriginFor<T>, spender: T::AccountId, value: U256) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			<Allowance<T>>::insert(&owner, &spender, value);

			Self::deposit_event(Event::Approval { owner, spender, value });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			value: U256
		) -> DispatchResult {
			let spender = ensure_signed(origin)?;

			let allowance = Self::allowance_impl(from.clone(), spender.clone())
				.checked_sub(value)
				.ok_or(Error::<T>::InsufficientAllowance)?;

			Self::transfer_impl(from.clone(), to, value);
			Self::approve_impl(from, spender, allowance-value);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn allowance_impl(
			owner: T::AccountId,
			spender: T::AccountId,
		) -> U256 {
			let allowance = <Allowance<T>>::get(owner, spender).unwrap_or(U256::zero());

			allowance
		}

		fn transfer_impl(from: T::AccountId, to: T::AccountId, value: U256) -> DispatchResult {
			let from_balance = <Balance<T>>::get(&from)
				.unwrap_or(U256::zero())
				.checked_sub(value)
				.ok_or(Error::<T>::InsufficientBalance)?;

			<Balance<T>>::insert(&from, from_balance-value);

			let to_balance = <Balance<T>>::get(&to).unwrap_or(U256::zero());
			<Balance<T>>::insert(&to, to_balance+value);

			Self::deposit_event(Event::Transfer { from, to, value });

			Ok(())
		}

		fn approve_impl(
			owner: T::AccountId,
			spender: T::AccountId,
			value: U256,
		) -> DispatchResult {
			<Allowance<T>>::insert(&owner, &spender, value);

			Self::deposit_event(Event::Approval { owner, spender, value });

			Ok(())
		}
	}
}