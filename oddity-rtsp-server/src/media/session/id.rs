use std::fmt;

use rand::Rng;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
  const SESSION_ID_LEN: usize = 16;

  pub fn generate() -> Id {
    Id(
      rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(Self::SESSION_ID_LEN)
        .map(char::from)
        .collect()
    )
  }

}

impl fmt::Display for Id {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.0.fmt(f)
  }

}