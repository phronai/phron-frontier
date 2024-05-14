
use ethereum_types::{H256, U256};
use serde::{
	ser::{Error, SerializeSeq},
	Serializer,
};

pub fn serialize_seq_h256<S>(data: &Option<Vec<H256>>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if let Some(vec) = data {
		let mut seq = serializer.serialize_seq(Some(vec.len()))?;
		for hash in vec {
			seq.serialize_element(&format!("{:x}", hash))?;
		}
		seq.end()
	} else {
		let seq = serializer.serialize_seq(Some(0))?;
		seq.end()
	}
}

pub fn serialize_bytes_0x<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
}

pub fn serialize_option_bytes_0x<S>(
	bytes: &Option<Vec<u8>>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if let Some(bytes) = bytes.as_ref() {
		return serializer.serialize_str(&format!("0x{}", hex::encode(&bytes[..])));
	}
	Err(S::Error::custom("String serialize error."))
}

pub fn serialize_opcode<S>(opcode: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let d = std::str::from_utf8(opcode)
		.map_err(|_| S::Error::custom("Opcode serialize error."))?
		.to_uppercase();
	serializer.serialize_str(&d)
}

pub fn serialize_string<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let d = std::str::from_utf8(value)
		.map_err(|_| S::Error::custom("String serialize error."))?
		.to_string();
	serializer.serialize_str(&d)
}

pub fn serialize_option_string<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if let Some(value) = value.as_ref() {
		let d = std::str::from_utf8(&value[..])
			.map_err(|_| S::Error::custom("String serialize error."))?
			.to_string();
		return serializer.serialize_str(&d);
	}
	Err(S::Error::custom("String serialize error."))
}

pub fn serialize_u256<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_u64(data.low_u64())
}

pub fn serialize_h256<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("{:x}", data))
}

pub fn serialize_h256_0x<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("0x{:x}", data))
}
