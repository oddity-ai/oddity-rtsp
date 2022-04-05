use std::string::FromUtf8Error;

#[derive(Default)]
pub struct Buffer {
  pos: usize,
  bytes: Vec<u8>,
}

impl Buffer {

  pub fn new() -> Self {
    Self {
      pos: 0,
      bytes: Vec::new(),
    }
  }

  pub fn from(bytes: Vec<u8>) -> Self {
    Self {
      pos: 0,
      bytes,
    }
  }

  pub fn extend(&mut self, bytes: Vec<u8>) {
    self.bytes.extend(bytes);
  }

  pub fn extend_from_slice(&mut self, bytes: &[u8]) {
    self.bytes.extend_from_slice(bytes);
  }

  /// Note: Catches CR, LF and CRLF.
  pub fn read_line(&mut self) -> Option<Result<String, FromUtf8Error>> {
    let mut found = false;
    let mut end = 0; // Index of LN, CR or CRLN
    let mut skip = 0; // Size of LN, CR or CRLN

    for i in self.pos..(self.bytes.len() - 1) {
      if self.bytes[i] == CR && self.bytes[i + 1] == LN {
        // Found CRLN at [i]
        (found, end, skip) = (true, i, 2);
      }

      if self.bytes[i] == CR || self.bytes[i] == LN {
        // Found CR or LN at [i]
        (found, end, skip) = (true, i, 1);
      }
    }

    if !found {
      let last = self.bytes.len() - 1;
      if self.bytes[last] == CR || self.bytes[last] == LN {
        // Found CRat [i]
        (found, end, skip) = (true, last, 1);
      }
    }

    if found {
      let start = self.pos;
      self.pos += end + skip;
      Some(self.extract_as_string(start, end))
    } else {
      None
    }
  }

  fn extract_as_string(
    &self,
    from: usize,
    len: usize
  ) -> Result<String, FromUtf8Error> {
    String::from_utf8(self.bytes[from..from + len].to_vec())
  }

}

const LN: u8 = b'\x0a';
const CR: u8 = b'\x0d';