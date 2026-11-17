//! Traits for setting values on Cap'n Proto builders and constructing values from
//! Cap'n Proto readers.
//!
//! # Examples
//!
//! With a `IpAddress` Cap'n Proto struct of:
//! ```capnp
//! struct IpAddress {
//!     lower64 @0 :UInt64;
//!     upper64 @1 :UInt64;
//! }
//! ```
//!
//! ## SetFrom/SetInto
//!
//! [`SetFrom`] and [`SetInto`] can be used and implemented in the following way:
//!
//! ```
//! use std::net::Ipv6Addr;
//! use capnp_convert::{SetFrom, SetInto};
//! # include!("./doctests/ip_capnp.rs");
//!
//! // We can implement `SetInto` for a conversion from `std::net::Ipv6Addr`
//! // to our `IpAddress` like this:
//! impl<'b> SetInto<ip_address::Builder<'b>> for Ipv6Addr {
//!     fn set_into(&self, builder: &mut ip_address::Builder<'b>) -> capnp::Result<()> {
//!         let octets = self.octets();
//!         let (upper, lower) = octets.split_at(8);
//!         builder.set_upper64(u64::from_be_bytes(<[u8; 8]>::try_from(upper).unwrap()));
//!         builder.set_lower64(u64::from_be_bytes(<[u8; 8]>::try_from(lower).unwrap()));
//!         Ok(())
//!     }
//! }
//!
//! fn main() {
//!     let mut builder = capnp::message::TypedBuilder::<ip_address::Owned>::new_default();
//!     let mut builder_root = builder.init_root();
//!
//!     // We can use `SetFrom` to use our `SetInto` conversion on a builder.
//!     builder_root.set_from(&Ipv6Addr::LOCALHOST).unwrap();
//!
//!     let reader = builder.get_root_as_reader().unwrap();
//!     assert_eq!(0, reader.get_upper64());
//!     assert_eq!(1, reader.get_lower64());
//! }
//! ```
//!
//! ## ReadFrom/ReadInto
//!
//! [`ReadFrom`] and [`ReadInto`] can be implemented and used in the following way:
//!
//! ```
//! use std::net::Ipv6Addr;
//! use capnp_convert::{ReadFrom, ReadInto};
//! # include!("./doctests/ip_capnp.rs");
//!
//! // We can implement `ReadFrom` for conversion back to a
//! // `std::net::Ipv6Addr` like this:
//! impl<'r> ReadFrom<ip_address::Reader<'r>> for std::net::Ipv6Addr {
//!     fn read_from(reader: ip_address::Reader<'r>) -> Result<Self, capnp::Error> {
//!         let mut octets = [0u8; 16];
//!         octets[0..8].copy_from_slice(&reader.get_upper64().to_be_bytes());
//!         octets[8..16].copy_from_slice(&reader.get_lower64().to_be_bytes());
//!         Ok(std::net::Ipv6Addr::from_octets(octets))
//!     }
//! }
//!
//! fn main() {
//!     let mut builder = capnp::message::TypedBuilder::<ip_address::Owned>::new_default();
//!     let mut builder_root = builder.init_root();
//!     builder_root.set_upper64(0);
//!     builder_root.set_lower64(1);
//!     let reader = builder.get_root_as_reader().unwrap();
//!
//!     // We can use `ReadInto` to use our `ReadFrom` conversion on a reader.
//!     let ip : Ipv6Addr = reader.read_into().unwrap();
//!     assert_eq!(Ipv6Addr::LOCALHOST, ip);
//! }
//! ```

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(test, deny(warnings))]

mod collections;
mod text;

/// Populate a builder from a value. It is the reciprocal of [`SetFrom`].
///
/// See [module example] for how to implement and use this trait.
///
/// # Generic implementations
///
/// - `SetInto<B> for V` implies [`SetFrom`]`<V> for B`
///
/// [module example]: index.html#setfromsetinto
#[expect(clippy::len_without_is_empty)]
pub trait SetInto<B> {
    /// Uses this value to set fields on provided `builder`.
    fn set_into(&self, builder: &mut B) -> capnp::Result<()>;

    /// Return length of `self`.
    fn len(&self) -> u32 {
        0
    }
}

/// Populate a builder from a value. It is the reciprocal of [`SetInto`].
///
/// See [module example] for how to use this trait.
///
/// # Generic implementations
///
/// - [`SetInto`]`<B> for V` implies `SetFrom<V> for B`
///
/// [module example]: index.html#setfromsetinto
pub trait SetFrom<V> {
    /// Set values of this builder from provided `input`.
    fn set_from(&mut self, input: &V) -> capnp::Result<()>;
}

impl<B, V> SetFrom<V> for B
where
    V: SetInto<B>,
{
    fn set_from(&mut self, input: &V) -> capnp::Result<()> {
        input.set_into(self)
    }
}

/// Constructs a new value from a Cap'n Proto reader. It is the reciprocal of [`ReadInto`].
///
/// See [module example] for how to implement and use this trait.
///
/// # Generic implementations
///
/// - `ReadFrom<B> for V` implies [`ReadInto`]`<V> for B`
///
/// [module example]: index.html#readfromreadinto
pub trait ReadFrom<R>: Sized {
    /// Read a new `Self` value from provided reader.
    fn read_from(reader: R) -> capnp::Result<Self>;
}

/// Convert a reader into a value. It is the reciprocal of [`ReadFrom`].
///
/// See [module example] for how to use this trait.
///
/// # Generic implementations
///
/// - [`ReadFrom`]`<R> for V` implies `ReadInto<V> for R`
///
/// [module example]: index.html#readfromreadinto
pub trait ReadInto<V> {
    /// Read the fields of this reader and convert them into a `V` value.
    fn read_into(self) -> capnp::Result<V>;
}

impl<R, V> ReadInto<V> for R
where
    V: ReadFrom<R>,
{
    fn read_into(self) -> capnp::Result<V> {
        V::read_from(self)
    }
}

#[cfg(feature = "bytes")]
impl ReadFrom<capnp::data::Reader<'_>> for bytes::Bytes {
    fn read_from(reader: capnp::data::Reader<'_>) -> capnp::Result<Self> {
        Ok(bytes::Bytes::copy_from_slice(reader))
    }
}
