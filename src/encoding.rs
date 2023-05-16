use crate::BINCODE_CONFIG;

pub trait DbEncode {
	fn enc(&self) -> Vec<u8>;
}

pub trait DbDecode {
	fn dec<B: AsRef<[u8]>>(bytes: B) -> Self;
}

impl<T> DbEncode for T
where
	T: bincode::Encode,
{
	fn enc(&self) -> Vec<u8> {
		bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap()
	}
}

impl<T> DbDecode for T
where
	T: bincode::Decode,
{
	fn dec<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		let (me, _) =
			bincode::decode_from_slice(bytes.as_ref(), BINCODE_CONFIG).unwrap();
		me
	}
}
