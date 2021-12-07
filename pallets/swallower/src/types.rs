use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{RuntimeDebug};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Swallower<BoundedString> {
	pub(super) name: BoundedString,
	pub(super) init_gene: BoundedString,
}