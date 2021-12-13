use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{RuntimeDebug};
use scale_info::TypeInfo;
use frame_support::inherent::Vec;

use crate::{AssetBalanceOf, Config};


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug,TypeInfo)]
// #[scale_info(skip_type_params(T))]
pub struct Swallower {
	pub(super) no:u64,
	pub(super) name: Vec<u8>,
	pub(super) init_gene: [u8;16],
	pub(super) gene: Vec<u8>,
}

#[derive(Clone,Encode,Decode,PartialEq, Eq,RuntimeDebug,TypeInfo)]
pub struct FeeConfig<T:Config>{
	change_name_fee:AssetBalanceOf<T>,
}

impl<T:Config> Default for FeeConfig<T>{
    fn default() -> Self {
        FeeConfig{
			change_name_fee:1u32.into(),
		}
    }
}

impl Swallower {
	pub(crate) fn new(name: Vec<u8>,init_gene:[u8;16],no:u64) -> Self {
		Swallower{
			no,
			name,
			init_gene,
			gene:init_gene.to_vec(),
		}

	}
}
