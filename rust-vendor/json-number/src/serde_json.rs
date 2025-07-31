use crate::{Buffer, Number, NumberBuf};

impl<B: Buffer> From<serde_json::Number> for NumberBuf<B> {
	#[inline(always)]
	fn from(n: serde_json::Number) -> Self {
		NumberBuf::new(B::from_vec(n.to_string().into_bytes()))
			.ok()
			.expect("invalid `serde_json::Number`")
	}
}

impl<B: Buffer> From<NumberBuf<B>> for serde_json::Number {
	#[inline(always)]
	fn from(n: NumberBuf<B>) -> Self {
		Self::from(n.as_number())
	}
}

impl<'n> From<&'n Number> for serde_json::Number {
	fn from(n: &'n Number) -> Self {
		if let Some(u) = n.as_u64() {
			u.into()
		} else if let Some(i) = n.as_i64() {
			i.into()
		} else {
			match n.as_str().parse() {
				Ok(n) => n,
				Err(_) => Self::from_f64(n.as_f64_lossy()).unwrap(),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::NumberBuf;

	#[test]
	fn serde_json_arbitrary_precision_compatibility() {
		let n = NumberBuf::new("1.1".to_owned().into_bytes()).unwrap();
		let serde_json::Value::Number(serde_json_n) = serde_json::to_value(n.clone()).unwrap()
		else {
			panic!("not a number")
		};

		let m: NumberBuf = serde_json_n.into();
		assert_eq!(n, m)
	}
}
