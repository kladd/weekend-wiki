use crate::BINCODE_CONFIG;

pub trait AsBytes {
	fn as_bytes(&self) -> Vec<u8>;
}

pub trait FromBytes {
	fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Self;
}

impl<T> AsBytes for T
where
	T: bincode::Encode,
{
	fn as_bytes(&self) -> Vec<u8> {
		bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap()
	}
}

impl<T> FromBytes for T
where
	T: bincode::Decode,
{
	fn from_bytes<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		let (me, _) =
			bincode::decode_from_slice(bytes.as_ref(), BINCODE_CONFIG).unwrap();
		me
	}
}
