pub type Error = ::tokenizer::Error<crate::lazy::LazyStructures>;

pub type Tokenizer<'pool, const N: usize, T> = ::tokenizer::Tokenizer<'pool, crate::lazy::LazyStructures, N, T>;

