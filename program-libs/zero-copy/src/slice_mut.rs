use core::{fmt, slice};
use std::{
    mem::size_of,
    ops::{Index, IndexMut},
};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Ref};

use crate::{add_padding, errors::ZeroCopyError};

// pub type ZeroCopySliceMutUsize<'a, T> = ZeroCopySliceMut<'a, usize, T>;
pub type ZeroCopySliceMutU32<'a, T> = ZeroCopySliceMut<'a, u32, T>;
pub type ZeroCopySliceMutU64<'a, T> = ZeroCopySliceMut<'a, u64, T>;
pub type ZeroCopySliceMutU16<'a, T> = ZeroCopySliceMut<'a, u16, T>;
pub type ZeroCopySliceMutU8<'a, T> = ZeroCopySliceMut<'a, u8, T>;

pub trait ZeroCopyTraits: Copy + KnownLayout + Immutable + FromBytes + IntoBytes {}

impl<T> ZeroCopyTraits for T where T: Copy + KnownLayout + Immutable + FromBytes + IntoBytes {}

#[repr(C)]
pub struct ZeroCopySliceMut<'a, L, T, const PAD: bool = true>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
{
    length: Ref<&'a [u8], L>,
    data: Ref<&'a mut [u8], [T]>,
}

impl<'a, L, T, const PAD: bool> ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    pub fn new(length: L, data: &'a mut [u8]) -> Result<Self, ZeroCopyError> {
        Ok(Self::new_at(length, data)?.0)
    }

    pub fn new_at(length: L, data: &'a mut [u8]) -> Result<(Self, &'a mut [u8]), ZeroCopyError> {
        let len = Self::required_size_for_capacity(length);
        if len > data.len() {
            return Err(ZeroCopyError::InsufficientMemoryAllocated(data.len(), len));
        }
        // write new value then deserialize as immutable
        {
            let (mut len, _) = Ref::<&mut [u8], L>::from_prefix(data).unwrap();
            Ref::<&mut [u8], L>::write(&mut len, length);
        }
        let (meta_data, data) = data.split_at_mut(Self::metadata_size());
        let len = Ref::<&[u8], L>::from_bytes(&meta_data[..size_of::<L>()]).unwrap();
        let len_usize: usize = u64::try_from(length).unwrap() as usize;
        let (data, bytes) = Ref::<&mut [u8], [T]>::from_prefix_with_elems(data, len_usize).unwrap();
        Ok((Self { length: len, data }, bytes))
    }

    pub fn new_at_multiple(
        num_slices: usize,
        capacity: L,
        mut bytes: &'a mut [u8],
    ) -> Result<(Vec<ZeroCopySliceMut<'a, L, T, PAD>>, &'a mut [u8]), ZeroCopyError> {
        let mut slices = Vec::with_capacity(num_slices);
        for _ in 0..num_slices {
            let (slice, _bytes) = Self::new_at(capacity, bytes)?;
            bytes = _bytes;
            slices.push(slice);
        }
        Ok((slices, bytes))
    }

    pub fn from_bytes(bytes: &'a mut [u8]) -> Result<Self, ZeroCopyError> {
        Ok(Self::from_bytes_at(bytes)?.0)
    }

    pub fn from_bytes_at(
        bytes: &'a mut [u8],
    ) -> Result<(ZeroCopySliceMut<'a, L, T, PAD>, &'a mut [u8]), ZeroCopyError> {
        let meta_data_size = Self::metadata_size();
        if bytes.len() < meta_data_size {
            return Err(ZeroCopyError::InsufficientMemoryAllocated(
                bytes.len(),
                meta_data_size,
            ));
        }

        let (meta_data, bytes) = bytes.split_at_mut(meta_data_size);
        let length = Ref::<&[u8], L>::from_bytes(&meta_data[..size_of::<L>()]).unwrap();
        let usize_len: usize = u64::try_from(*length).unwrap() as usize;
        let full_vector_size = Self::data_size(*length);
        if bytes.len() < full_vector_size {
            return Err(ZeroCopyError::InsufficientMemoryAllocated(
                bytes.len(),
                full_vector_size + meta_data_size,
            ));
        }
        let (data, bytes) =
            Ref::<&mut [u8], [T]>::from_prefix_with_elems(bytes, usize_len).unwrap();
        Ok((ZeroCopySliceMut { length, data }, bytes))
    }

    pub fn from_bytes_at_multiple(
        num_slices: usize,
        mut bytes: &'a mut [u8],
    ) -> Result<(Vec<ZeroCopySliceMut<'a, L, T, PAD>>, &'a mut [u8]), ZeroCopyError> {
        let mut slices = Vec::with_capacity(num_slices);
        for _ in 0..num_slices {
            let (slice, _bytes) = Self::from_bytes_at(bytes)?;
            bytes = _bytes;
            slices.push(slice);
        }
        Ok((slices, bytes))
    }

    pub fn try_into_array<const N: usize>(&self) -> Result<[T; N], ZeroCopyError> {
        if self.len() != N {
            return Err(ZeroCopyError::ArraySize(N, self.len()));
        }
        Ok(std::array::from_fn(|i| *self.get(i).unwrap()))
    }

    #[inline]
    pub fn metadata_size() -> usize {
        let mut size = size_of::<L>();
        if PAD {
            add_padding::<L, T>(&mut size);
        }
        size
    }

    #[inline]
    pub fn data_size(length: L) -> usize {
        let usize_len: usize = u64::try_from(length).unwrap() as usize;
        usize_len * size_of::<T>()
    }

    #[inline]
    pub fn required_size_for_capacity(capacity: L) -> usize {
        Self::metadata_size() + Self::data_size(capacity)
    }
}

impl<'a, L, T, const PAD: bool> ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    #[inline]
    pub fn len(&self) -> usize {
        let usize_len: usize = u64::try_from(*self.length).unwrap() as usize;
        usize_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn first(&self) -> Option<&T> {
        self.get(0)
    }

    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.get(self.len().saturating_sub(1))
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len().saturating_sub(1))
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.data.as_ref()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data.as_mut()
    }

    #[inline]
    pub fn data_as_ptr(&self) -> *const T {
        self.as_slice().as_ptr()
    }

    #[inline]
    pub fn data_as_mut_ptr(&mut self) -> *mut T {
        self.as_mut_slice().as_mut_ptr()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_mut_slice().get_mut(index)
    }
}
impl<'a, L, T, const PAD: bool> ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    pub fn to_vec(&self) -> Vec<T> {
        self.as_slice().to_vec()
    }
}

impl<'a, L, T, const PAD: bool> IndexMut<usize> for ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<'a, L, T, const PAD: bool> Index<usize> for ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<'a, 'b, L, T, const PAD: bool> IntoIterator for &'b ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    type Item = &'b T;
    type IntoIter = slice::Iter<'b, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'b, L, T, const PAD: bool> IntoIterator for &'b mut ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    type Item = &'b mut T;
    type IntoIter = slice::IterMut<'b, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'b, 'a, L, T, const PAD: bool> ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits,
    u64: From<L>,
{
    #[inline]
    pub fn iter(&'b self) -> slice::Iter<'b, T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&'b mut self) -> slice::IterMut<'b, T> {
        self.as_mut_slice().iter_mut()
    }
}

impl<'a, L, T, const PAD: bool> PartialEq for ZeroCopySliceMut<'a, L, T, PAD>
where
    L: ZeroCopyTraits,
    T: ZeroCopyTraits + PartialEq,
    u64: From<L>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice() && self.len() == other.len()
    }
}

impl<'a, L, T, const PAD: bool> fmt::Debug for ZeroCopySliceMut<'a, L, T, PAD>
where
    T: ZeroCopyTraits + fmt::Debug,
    L: ZeroCopyTraits,
    u64: From<L>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_vec())
    }
}
