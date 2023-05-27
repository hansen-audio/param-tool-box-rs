// Copyright(c) 2023 Hansen Audio.

pub mod cbindings;
pub mod convert;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
