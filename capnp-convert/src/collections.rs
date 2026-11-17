use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::fmt;
use std::str::FromStr;

use capnp::Error;
use capnp::traits::OwnedStruct;

use crate::{ReadFrom, ReadInto as _, SetFrom, SetInto};

impl<'b, T> SetInto<capnp::data_list::Builder<'b>> for Vec<T>
where
    T: AsRef<[u8]>,
{
    fn set_into(&self, builder: &mut capnp::data_list::Builder<'b>) -> capnp::Result<()> {
        for (index, item) in self.iter().enumerate() {
            builder.set(index as u32, item.as_ref());
        }
        Ok(())
    }

    fn len(&self) -> u32 {
        self.len() as u32
    }
}

impl<'r, T> ReadFrom<capnp::data_list::Reader<'r>> for Vec<T>
where
    for<'tr> T: ReadFrom<capnp::data::Reader<'tr>>,
{
    fn read_from(reader: capnp::data_list::Reader<'r>) -> Result<Self, Error> {
        let mut ret = Vec::with_capacity(reader.len() as usize);
        for item_r in reader.iter() {
            let item = item_r?.read_into()?;
            ret.push(item);
        }
        Ok(ret)
    }
}

impl<'b, I, IB> SetInto<capnp::struct_list::Builder<'b, IB>> for &[I]
where
    IB: OwnedStruct,
    for<'b2> <IB as OwnedStruct>::Builder<'b2>: SetFrom<I>,
{
    fn set_into(&self, builder: &mut capnp::struct_list::Builder<'b, IB>) -> capnp::Result<()> {
        for (index, item) in self.iter().enumerate() {
            builder.reborrow().get(index as u32).set_from(item)?;
        }
        Ok(())
    }

    fn len(&self) -> u32 {
        <[I]>::len(self) as u32
    }
}

impl<'d, 'b, KV, K, V> SetInto<capnp::struct_list::Builder<'b, KV>> for &'d BTreeMap<K, V>
where
    KV: OwnedStruct,
    for<'ib> <KV as OwnedStruct>::Builder<'ib>: SetFrom<(&'d K, &'d V)>,
{
    fn set_into(&self, builder: &mut capnp::struct_list::Builder<'b, KV>) -> capnp::Result<()> {
        for (index, item) in self.iter().enumerate() {
            builder.reborrow().get(index as u32).set_from(&item)?;
        }
        Ok(())
    }

    fn len(&self) -> u32 {
        (*self).len() as u32
    }
}

impl<'r, K, V, O> ReadFrom<capnp::struct_list::Reader<'r, O>> for BTreeMap<K, V>
where
    (K, V): ReadFrom<<O as OwnedStruct>::Reader<'r>>,
    K: Ord,
    O: OwnedStruct,
{
    fn read_from(reader: capnp::struct_list::Reader<'r, O>) -> Result<Self, Error> {
        let mut ret = BTreeMap::new();
        for item_r in reader.iter() {
            let (key, value) = item_r.read_into()?;
            ret.insert(key, value);
        }
        Ok(ret)
    }
}

impl<'d, 'b, KV, K, V> SetInto<capnp::struct_list::Builder<'b, KV>> for &'d HashMap<K, V>
where
    KV: OwnedStruct,
    for<'ib> <KV as OwnedStruct>::Builder<'ib>: SetFrom<(&'d K, &'d V)>,
{
    fn set_into(&self, builder: &mut capnp::struct_list::Builder<'b, KV>) -> capnp::Result<()> {
        for (index, item) in self.iter().enumerate() {
            builder.reborrow().get(index as u32).set_from(&item)?;
        }
        Ok(())
    }

    fn len(&self) -> u32 {
        (*self).len() as u32
    }
}

impl<'r, K, V, O> ReadFrom<capnp::struct_list::Reader<'r, O>> for HashMap<K, V>
where
    (K, V): ReadFrom<<O as OwnedStruct>::Reader<'r>>,
    K: Eq + std::hash::Hash,
    O: OwnedStruct,
{
    fn read_from(reader: capnp::struct_list::Reader<'r, O>) -> Result<Self, Error> {
        let mut ret = HashMap::with_capacity(reader.len() as usize);
        for item_r in reader.iter() {
            let (key, value) = item_r.read_into()?;
            ret.insert(key, value);
        }
        Ok(ret)
    }
}

macro_rules! set_text_list {
    ($type:ident) => {
        impl<'b, T> SetInto<capnp::text_list::Builder<'b>> for $type<T>
        where
            T: AsRef<str>,
        {
            fn set_into(&self, builder: &mut capnp::text_list::Builder<'b>) -> capnp::Result<()> {
                for (index, name) in self.iter().enumerate() {
                    builder.set(index as u32, name.as_ref());
                }
                Ok(())
            }

            fn len(&self) -> u32 {
                self.len() as u32
            }
        }
    };
}

macro_rules! read_text_list {
    ($type:ident, |$len:ident| $create:expr, |$ret:ident, $item:ident| $push:expr) => (
        read_text_list!($type, { }, |$len| $create, |$ret, $item| $push);
    );
    ($type:ident, { $($($where:tt)+)? }, |$len:ident| $create:expr, |$ret:ident, $item:ident| $push:expr) => (
        impl<'r, T> ReadFrom<capnp::text_list::Reader<'r>> for $type<T>
        where
            T: FromStr $(+ $($where)+)?,
            <T as FromStr>::Err: fmt::Display,
        {
            fn read_from(reader: capnp::text_list::Reader<'r>) -> Result<Self, Error> {
                let $len = reader.len() as usize;
                let mut $ret = $create;
                for item_r in reader.iter() {
                    let $item = item_r?
                        .to_str()?
                        .parse::<T>()
                        .map_err(|err| Error::failed(err.to_string()))?;
                    $push;
                }
                Ok($ret)
            }
        }
    );
}

macro_rules! set_struct_list {
    ($type:ident) => {
        impl<'b, T, B> SetInto<capnp::struct_list::Builder<'b, B>> for $type<T>
        where
            B: OwnedStruct,
            for<'b2> <B as OwnedStruct>::Builder<'b2>: SetFrom<T>,
        {
            fn set_into(
                &self,
                builder: &mut capnp::struct_list::Builder<'b, B>,
            ) -> capnp::Result<()> {
                for (index, item) in self.iter().enumerate() {
                    builder.reborrow().get(index as u32).set_from(&item)?;
                }
                Ok(())
            }

            fn len(&self) -> u32 {
                self.len() as u32
            }
        }
    };
}

macro_rules! read_struct_list {
    ($type:ident, |$len:ident| $create:expr, |$ret:ident, $item:ident| $push:expr) => (
        read_struct_list!($type, {}, |$len| $create, |$ret, $item| $push);
    );
    ($type:ident, { $($($where:tt)+)? }, |$len:ident| $create:expr, |$ret:ident, $item:ident| $push:expr) => (
        impl<'r, T, R> ReadFrom<capnp::struct_list::Reader<'r, R>> for $type<T>
        where
            T: ReadFrom<<R as OwnedStruct>::Reader<'r>> $(+ $($where)+)?,
            R: OwnedStruct,
        {
            fn read_from(reader: capnp::struct_list::Reader<'r, R>) -> Result<Self, Error> {
                let $len = reader.len() as usize;
                let mut $ret = $create;
                for item_r in reader.iter() {
                    let $item = T::read_from(item_r)?;
                    $push;
                }
                Ok($ret)
            }
        }
    );
}

macro_rules! impl_lists {
    ($type:ident, |$len:ident| $create:expr, |$ret:ident, $item:ident| $push:expr) => {
        impl_lists!($type, {}, |$len| $create, |$ret, $item| $push);
    };
    ($type:ident, { $($($where:tt)+)? }, |$len:ident| $create:expr, |$ret:ident, $item:ident| $push:expr) => {
        set_text_list!($type);
        read_text_list!($type, { $($($where)+ )? }, |$len| $create, |$ret, $item| $push);
        set_struct_list!($type);
        read_struct_list!($type, { $($($where)+ )? }, |$len| $create, |$ret, $item| $push);
    };
}

impl_lists!(
    BinaryHeap,
    { Ord },
    |len| BinaryHeap::with_capacity(len),
    |ret, item| ret.push(item)
);
impl_lists!(BTreeSet, { Ord }, |_len| BTreeSet::new(), |ret, item| ret
    .insert(item));
impl_lists!(
    HashSet,
    { Eq + std::hash::Hash },
    |len| HashSet::with_capacity(len),
    |ret, item| ret.insert(item)
);
impl_lists!(Vec, |len| Vec::with_capacity(len), |ret, item| ret
    .push(item));
impl_lists!(VecDeque, |len| VecDeque::with_capacity(len), |ret, item| {
    ret.push_back(item)
});
impl_lists!(LinkedList, |_len| LinkedList::new(), |ret, item| ret
    .push_back(item));
