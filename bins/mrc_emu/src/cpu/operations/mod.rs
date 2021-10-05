pub mod arithmetic;
pub mod logic;

use crate::cpu::{Flags, SignificantBit};

#[cfg(test)]
pub fn test_flags(
    flags: &Flags,
    carry: bool,
    parity: bool,
    aux_carry: bool,
    zero: bool,
    sign: bool,
    overflow: bool,
) {
    assert_eq!(carry, flags.contains(Flags::CARRY));
    assert_eq!(parity, flags.contains(Flags::PARITY));
    assert_eq!(aux_carry, flags.contains(Flags::AUX_CARRY));
    assert_eq!(zero, flags.contains(Flags::ZERO));
    assert_eq!(sign, flags.contains(Flags::SIGN));
    assert_eq!(overflow, flags.contains(Flags::OVERFLOW));
}

const PARITY_TABLE: [u8; 0x100] = [
    1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
    0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
    0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
    1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
    0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
    1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
    1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
    0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
];

pub fn flags_from_byte_result(flags: &mut Flags, result: u8) {
    flags.set(Flags::ZERO, result == 0);
    flags.set(Flags::SIGN, result.most_significant_bit());
    flags.set(Flags::PARITY, PARITY_TABLE[result as usize] == 1);
}

pub fn flags_from_word_result(flags: &mut Flags, result: u16) {
    flags.set(Flags::ZERO, result == 0);
    flags.set(Flags::SIGN, result.most_significant_bit());
    flags.set(Flags::PARITY, PARITY_TABLE[(result & 0xFF) as usize] == 1);
}
