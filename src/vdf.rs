use std::borrow::Cow;

use keyvalues_serde::parser::Vdf;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	KeyvaluesParser(Box<keyvalues_serde::parser::error::Error>),
	#[error(transparent)]
	KeyvaluesSerde(Box<keyvalues_serde::Error>),
	#[allow(clippy::enum_variant_names)]
	#[error(transparent)]
	SerdePathToError(Box<serde_path_to_error::Error<keyvalues_serde::Error>>),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub fn from_str_with_key<'de, T: Deserialize<'de>>(s: &'de str) -> Result<(T, Cow<'de, str>)> {
	let vdf = keyvalues_serde::parser::parse(s)
		.map(Vdf::from)
		.map_err(Box::new)
		.map_err(Error::KeyvaluesParser)?;
	let (mut deserializer, key) = keyvalues_serde::Deserializer::new_with_key(vdf)
		.map_err(Box::new)
		.map_err(Error::KeyvaluesSerde)?;

	let mut track = serde_path_to_error::Track::new();
	let value = match T::deserialize(serde_path_to_error::Deserializer::new(
		&mut deserializer,
		&mut track,
	)) {
		Ok(value) => value,
		Err(error) => {
			return Err(Error::SerdePathToError(Box::new(
				serde_path_to_error::Error::new(track.path(), error),
			)))
		}
	};

	// keyvalues-serde has no public trailing-token check (it uses this internally too)
	#[allow(deprecated)]
	if !deserializer.is_empty() {
		return Err(Error::KeyvaluesSerde(Box::new(
			keyvalues_serde::Error::TrailingTokens,
		)));
	}

	Ok((value, key))
}

pub fn from_str<'de, T: Deserialize<'de>>(s: &'de str) -> Result<T> {
	from_str_with_key(s).map(|(value, _key)| value)
}
