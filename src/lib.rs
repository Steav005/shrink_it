use std::ops::Deref;
use itertools::Itertools;
use std::mem::size_of;

pub fn shrink_it<S, T>(iter: S, bits: usize) -> Vec<u8>
where
    S: IntoIterator<Item = T> + Deref<Target = [T]>,
    T: core::ops::Shr<Output = T> + core::ops::BitAnd<Output = T> + From<u8> + Copy + PartialEq,
{
    if size_of::<T>() * 8 <= bits{
        panic!("The datatype is already at the target bit size or even smaller")
    }

    let necessary_bits = iter.len() * bits;
    let padding_bits = match necessary_bits % 8 {
        0 => 0,
        o => 8 - o,
    };
    //let _necessary_bytes = (necessary_bits + padding_bits) / 8;

    iter.into_iter()
        .flat_map(|n| NumberBitIter(n).take(bits))
        .take(necessary_bits)
        .chain(vec![false; padding_bits])
        .chunks(8)
        .into_iter()
        .map(|byte| byte.fold(0 as u8, |acc, x|
            (acc << 1) | (x as u8)
        ))
        .collect::<Vec<u8>>()
}

pub fn expand_it<S, O>(iter: S, bits: usize) -> Vec<O>
    where
        S: IntoIterator<Item = u8> + Deref<Target = [u8]>,
        O: core::ops::Shr<Output = O> + core::ops::Shl<Output = O> + core::ops::Not<Output = O> + core::ops::BitOr<Output = O> + core::ops::BitAnd<Output = O> + From<u8> + Copy + PartialEq
{
    let all_bits = iter.len() * 8;
    let relevant_bits = all_bits - (all_bits % bits);
    //let values = relevant_bits / bits;
    let output_size = size_of::<O>() * 8;
    let output_shift = O::from((size_of::<O>() * 8 - 1) as u8);
    let one = O::from(1 as u8);
    let zero = O::from(0 as u8);

    iter.into_iter()
        .flat_map(|n| NumberBitIter(n.reverse_bits()).take(8))
        .take(relevant_bits)
        .chunks(bits)
        .into_iter()
        .map(|b| b.fold(zero, |acc, x| {
            if x {
                acc.shr(one)
                    | one.shl(output_shift)
            } else {
                acc.shr(one)
                    & one.shl(output_shift).not()
            }
        }
        ).shr(O::from((output_size - bits) as u8)))
        .collect::<Vec<O>>()
}

pub struct NumberBitIter<T>(T)
where
    T: core::ops::Shr<Output = T> + core::ops::BitAnd<Output = T> + From<u8> + Copy + PartialEq;

impl<T> Iterator for NumberBitIter<T>
where
    T: core::ops::Shr<Output = T> + core::ops::BitAnd<Output = T> + From<u8> + Copy + PartialEq,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let value = Some(self.0 & T::from(1) == T::from(1));
        self.0 = self.0.shr(T::from(1));
        value
    }
}

#[cfg(test)]
mod tests {
    use crate::{shrink_it, expand_it};

    #[test]
    fn basic() {
        let array: Vec<i32> = vec![-10, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        println!("{:?}", array);
        let shrunk = shrink_it(array.clone(), 7);
        println!("{:?}", shrunk);
        let expand: Vec<i32> = expand_it(shrunk, 7);
        println!("{:?}", expand);
        assert_eq!(array, expand);
    }
}
