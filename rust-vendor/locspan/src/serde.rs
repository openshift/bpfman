use crate::Meta;

impl<T: serde::Serialize, M> serde::Serialize for Meta<T, M> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.serialize(serializer)
	}
}

impl<'de, T: serde::Deserialize<'de>, M: Default> serde::Deserialize<'de> for Meta<T, M> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let t = T::deserialize(deserializer)?;
		Ok(Self(t, M::default()))
	}
}
