pub struct Reader<'b> {
  buffer: &'b[u8],
  head: usize,
}

impl<'b> Reader<'b> {

  pub fn new(buffer: &'b[u8]) -> Self {
    Self {
      buffer,
      head: 0,
    }
  }

  pub fn read_until_crlf(&mut self) -> (bool, String) {
    for i in self.head..self.buffer.len() - 1 {
      if self.buffer[i] == b'\r' &&
         self.buffer[i + 1] == b'\n' {
        return (true, self.read(i, 2))
      }
    }

    (false, String::new())
  }

  #[inline]
  fn read(&mut self, end: usize, and_skip: usize) -> String {
    let bite = String::from_utf8_lossy(&self.buffer[self.head..end])
      .trim()
      .to_string();
    self.head += end + and_skip;
    bite
  }

}