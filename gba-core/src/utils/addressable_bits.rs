pub trait AddressableBits<T> {
    fn bit(&self, index: usize) -> T;
    fn bits(&self, start: usize, end_inclusive: usize) -> T;
    fn set_bit(self, index: usize, value: bool) -> T;
    fn mut_bit(&mut self, index: usize, value: bool);
}

macro_rules! addressable_bits_impl {
    ($SelfT:ty) => {
        impl AddressableBits<$SelfT> for $SelfT {
            #[inline]
            fn bit(&self, index: usize) -> $SelfT {
                (self >> index) & 1
            }

            #[inline]
            fn bits(&self, start: usize, end_inclusive: usize) -> $SelfT {
                let len = end_inclusive - start + 1;
                (self >> start) & ((1 << len) - 1)
            }

            #[inline]
            fn set_bit(self, index: usize, value: bool) -> $SelfT {
                let mask = !(1 << index);
                (self & mask) | (if value { 1 } else { 0 }) << index
            }

            #[inline]
            fn mut_bit(&mut self, index: usize, value: bool) {
                let mask = !(1 << index);
                *self = (*self & mask) | (if value { 1 } else { 0 }) << index;
            }
        }
    };
}

addressable_bits_impl! { u64 }
addressable_bits_impl! { u32 }
addressable_bits_impl! { u16 }
addressable_bits_impl! { u8 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits() {
        assert_eq!(0b00111100u32.bits(2, 5), 0b1111);
        assert_eq!(0b00000100u32.bits(2, 2), 1);
        assert_eq!(0b00001000u32.bits(2, 2), 0);
    }

    #[test]
    fn test_set_bit() {
        assert_eq!(0u32.set_bit(2, true), 4);
        assert_eq!(0b1010u32.set_bit(3, false), 2);
    }
}
