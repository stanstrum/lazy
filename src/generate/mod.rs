use crate::lang::Lazy;

#[derive(Debug)]
pub enum Error {

}

type Result<T = ()> = std::result::Result<T, Error>;

pub fn entry(lazy: &mut Lazy) -> Result {
  todo!()
}
