use std::{fmt, str::FromStr};

use crate::{ReadFrom, SetInto};

impl<'b, S> SetInto<capnp::text::Builder<'b>> for S
where
    S: AsRef<str>,
{
    fn set_into(&self, builder: &mut capnp::text::Builder<'b>) -> capnp::Result<()> {
        builder.push_str(self.as_ref());
        Ok(())
    }

    fn len(&self) -> u32 {
        self.as_ref().len() as u32
    }
}

impl<T> ReadFrom<capnp::text::Reader<'_>> for T
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    fn read_from(reader: capnp::text::Reader<'_>) -> capnp::Result<Self> {
        reader
            .to_str()?
            .parse::<T>()
            .map_err(|err| capnp::Error::failed(err.to_string()))
    }
}
