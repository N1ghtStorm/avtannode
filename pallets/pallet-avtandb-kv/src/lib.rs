#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::{DispatchResult, Vec}, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::sp_runtime::{
		offchain as rt_offchain,
		offchain::{
			storage::StorageValueRef,
			storage_lock::{BlockAndTime, StorageLock},
		},
		transaction_validity::{
			InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
		},
		RuntimeDebug,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_n: BlockNumberFor<T>) {
            // frame_support::debug::print!("looool lLLLLLLLLLol lol lol {:?}", _n);
			log::info!("================================================= {:?}", _n);
			let a = match Self::fetch_from_remote(){
				Err(e) => {
					log::info!("=====ERROR:{:?}", e);
					return;
				},
				Ok(v) => v
			};

			log::info!("ANSWER {:?}", a);
            // debug::debug!("Entering off-chain workers {:?}", _n);
            // todo!();
        }
    }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn getvalue)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type KvStore<T> = StorageMap<
		_,
		Blake2_128Concat,
        Vec<u8>,
        Vec<u8>,
        OptionQuery
	>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// /// Event documentation should end with an array that provides descriptive names for event
		// /// parameters. [something, who]
		// SomethingStored(u32, T::AccountId),
		NewKeyAdded(Vec<u8>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		KeyAlreadyExists,
		HttpFetchingError
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn add_value(origin: OriginFor<T>, key: Vec<u8>, value: Vec<u8>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			KvStore::<T>::try_mutate(key.clone(), |val_opt| -> DispatchResult {
				match val_opt {
					Some(_) => {
						return Err(Error::<T>::KeyAlreadyExists.into())
					},
					None => {
						*val_opt = Some(value)
					}
				}
				Ok(())
			})?;
			Self::deposit_event(Event::NewKeyAdded(key));
			Ok(())
		}
	}

	impl<T> Pallet<T> where T: Config {
		fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
			let request = rt_offchain::http::Request::get("http://localhost:18085/avtan");

			let timeout = sp_io::offchain::timestamp()
				.add(rt_offchain::Duration::from_millis(3000));
				let pending = request
				// .add_header("User-Agent", HTTP_HEADER_USER_AGENT)
				.deadline(timeout) // Setting the timeout time
				.send() // Sending the request out by the host
				.map_err(|_| <Error<T>>::HttpFetchingError)?;

			let response = pending
				.try_wait(timeout)
				.map_err(|_| <Error<T>>::HttpFetchingError)?
				.map_err(|_| <Error<T>>::HttpFetchingError)?;
		  
			if response.code != 200 {
				// debug::error!("Unexpected http request status code: {}", response.code);
				return Err(<Error<T>>::HttpFetchingError);
			}
		  
			// Next we fully read the response body and collect it to a vector of bytes.
			Ok(response.body().collect::<Vec<u8>>())
		}
	}
}
