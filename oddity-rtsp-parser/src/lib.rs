mod reader;

use std::io::{Cursor, BufRead};

type Result<T> = std::result::Result<T, Error>;

pub struct Parser {
  state: State,
}

pub enum State {
  FirstLine(String),
  HeaderVar(String),
  HeaderVal(String, String),
  Done,
}

pub enum Out {
  Done(Request),
  Hungry,
}

pub enum Line {
  Complete(String),
  Incomplete(String),
}

pub enum Method {
  Describe,
  Announce,
  Setup,
  Play,
  Pause,
  Record,
  Options,
  Redirect,
  Teardown,
  GetParameter,
  SetParameter,
}

pub struct Request;

impl Parser {

  pub fn new() -> Self {
    Self {
      state: State::FirstLine(String::new()),
    }
  }

  pub fn feed(&mut self, buffer: &[u8]) -> Result<Out> {
    let mut cursor = Cursor::new(buffer);

    while cursor.get_ref().len() > 0 {
      self.state = self.feed_inner(&mut cursor)?;

      if let State::Done = self.state {
        break;
      }
    }

    if let State::Done = self.state {
      Ok(Out::Done(Request {}))
    } else {
      Ok(Out::Hungry)
    }
  }

  fn feed_inner(&mut self, cursor: &mut Cursor<&[u8]>) -> Result<State> {
    Ok(match &self.state {
      State::FirstLine(first_line) => {
        match Self::parse_line(&mut cursor, &first_line)? {
          Line::Complete(line) => {
            State::HeaderVar(String::new())
          },
          Line::Incomplete(line) =>
            State::FirstLine(line)
        }
      },
      State::HeaderVar(header_var) => {

      },
      State::HeaderVal(header_var, header_val) => {

      },
      State::Done =>
        return Err(Error::AlreadyDone),
    })
  }

  #[inline]
  fn parse_line(cursor: &mut Cursor<&[u8]>, existing_part: &str) -> Result<Line> {
    let mut line_buf = String::new();
    let read_bytes = cursor.read_line(&mut line_buf)
      .map_err(|_| Error::Encoding)?;

    let line = existing_part.to_owned() + &line_buf;

    Ok(if read_bytes == 0 {
      Line::Incomplete(line)
    } else {
      Line::Complete(line)
    })
  }

  // TODO fn parse_first_line(line: String) -> (Method, Url, Version)

}

pub enum Error {
  Encoding,
  AlreadyDone,
}