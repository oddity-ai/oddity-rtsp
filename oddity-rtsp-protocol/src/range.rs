use std::str::FromStr;
use std::fmt;

use super::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
  from: Option<NptTime>,
  to: Option<NptTime>,
}

impl Range {
  const SUPPORTED_UNITS: [&'static str; 1] = [
    "npt",
  ];
}

impl fmt::Display for Range {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match (self.from.as_ref(), self.to.as_ref()) {
      (Some(from), Some(to))
        => write!(f, "{}-{}", from, to),
      (Some(from), None)
        => write!(f, "{}-", from),
      (None, Some(to))
        => write!(f, "-{}", to),
      (None, None)
        => write!(f, "-"),
    }
  }

}

impl FromStr for Range {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.split_once(";") {
      None => {
        if let Some((unit, value)) = s.split_once("=") {
          if Self::SUPPORTED_UNITS.contains(&unit) {
            if let Some((from, to)) = value.split_once("-") {
              let from = if !from.is_empty() { Some(from.parse()?) } else { None };
              let to = if !to.is_empty() { Some(to.parse()?) } else { None };
              Ok(Range {
                from,
                to,
              })
            } else {
              Err(Error::RangeMalformed { value: s.to_string() })
            }
          } else {
            Err(Error::RangeUnitNotSupported { value: s.to_string() })
          }
        } else {
          Err(Error::RangeMalformed { value: s.to_string() })
        }
      },
      Some((_, time)) => {
        if time.starts_with("time=") {
          Err(Error::RangeTimeNotSupported { value: s.to_string() })
        } else {
          Err(Error::RangeMalformed { value: s.to_string() })
        }
      },
    }
  }

}

#[derive(Debug, Clone, PartialEq)]
pub enum NptTime {
  Now,
  Time(f64),
}

impl fmt::Display for NptTime {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      NptTime::Now
        => write!(f, "now"),
      NptTime::Time(seconds)
        => write!(f, "{:.3}", seconds),
    }
  }

}

impl FromStr for NptTime {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "now" => Ok(NptTime::Now),
      s => {
        match s.split(":").collect::<Vec<_>>().as_slice() {
          &[npt_time] => {
            let npt_time = npt_time.parse::<f64>()
              .map_err(|_| Error::RangeNptTimeMalfored {
                value: s.to_string(),
              })?;
            Ok(NptTime::Time(npt_time))
          },
          &[npt_hh, npt_mm, npt_ss] => {
            let npt_hh = npt_hh.parse::<u32>();
            let npt_mm = npt_mm.parse::<u32>();
            let npt_secs = npt_ss.parse::<f32>();
            match (npt_hh, npt_mm, npt_secs) {
              (Ok(hh), Ok(mm), Ok(secs)) => {
                let npt_time =
                  ((hh * 3600) as f64) +
                  ((mm * 60) as f64) +
                  (secs as f64);
                Ok(NptTime::Time(npt_time))
              },
              _ => {
                Err(Error::RangeNptTimeMalfored {
                  value: s.to_string(),
                })
              }
            }
          },
          _ => Err(Error::RangeNptTimeMalfored {
            value: s.to_string(),
          }),
        }
      },
    }
  }

}