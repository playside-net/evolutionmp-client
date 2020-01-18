use crate::invoke;

pub fn get_nickname<'a>() -> &'a str {
    invoke!(&str, 0x198D161F458ECC7F)
}