use std::fmt::LowerHex;




pub fn to_hex<T>(num: T) -> String
where
    T: Copy + PartialEq + From<u8> + std::ops::DivAssign + std::ops::Rem + Into<u64>
{
    let mut result = String::new();
    let hex_chars = "0123456789ABCDEF".chars().collect::<Vec<_>>();

    let mut value: u64 = num.into();
    while value > 0 {
        let remainder = (value % 16) as usize;
        result.insert(0, hex_chars[remainder]);
        value /= 16;
    }

    while result.len() % 2 != 0 {
        result.insert(0, '0');
    }

    if result.is_empty() {
        result.push('0');
    }

    result
}