use std::fmt;
use std::str::FromStr;

use super::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub start: Option<NptTime>,
    pub end: Option<NptTime>,
}

impl Range {
    const SUPPORTED_UNITS: [&'static str; 1] = ["npt"];

    #[must_use]
    pub const fn new(start: NptTime, end: NptTime) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }

    #[must_use]
    pub const fn new_for_live() -> Self {
        Self {
            start: Some(NptTime::Now),
            end: None,
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "npt=")?;
        match (self.start.as_ref(), self.end.as_ref()) {
            (Some(start), Some(end)) => write!(f, "{start}-{end}"),
            (Some(start), None) => write!(f, "{start}-"),
            (None, Some(end)) => write!(f, "-{end}"),
            (None, None) => write!(f, "-"),
        }
    }
}

impl FromStr for Range {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(';') {
            None => {
                if let Some((unit, value)) = s.split_once('=') {
                    if Self::SUPPORTED_UNITS.contains(&unit) {
                        if let Some((start, end)) = value.split_once('-') {
                            let start = if start.is_empty() {
                                None
                            } else {
                                Some(start.parse()?)
                            };
                            let end = if end.is_empty() {
                                None
                            } else {
                                Some(end.parse()?)
                            };
                            Ok(Self { start, end })
                        } else {
                            Err(Error::RangeMalformed {
                                value: s.to_string(),
                            })
                        }
                    } else {
                        Err(Error::RangeUnitNotSupported {
                            value: s.to_string(),
                        })
                    }
                } else {
                    Err(Error::RangeMalformed {
                        value: s.to_string(),
                    })
                }
            }
            Some((_, time)) => {
                if time.starts_with("time=") {
                    Err(Error::RangeTimeNotSupported {
                        value: s.to_string(),
                    })
                } else {
                    Err(Error::RangeMalformed {
                        value: s.to_string(),
                    })
                }
            }
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
            Self::Now => write!(f, "now"),
            Self::Time(seconds) => write!(f, "{seconds:.3}"),
        }
    }
}

impl FromStr for NptTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "now" => Ok(Self::Now),
            s => match *s.split(':').collect::<Vec<_>>().as_slice() {
                [npt_time] => {
                    let npt_time =
                        npt_time
                            .parse::<f64>()
                            .map_err(|_| Error::RangeNptTimeMalfored {
                                value: s.to_string(),
                            })?;
                    Ok(Self::Time(npt_time))
                }
                [npt_hh, npt_mm, npt_ss] => {
                    let npt_hh = npt_hh.parse::<u32>();
                    let npt_mm = npt_mm.parse::<u32>();
                    let npt_secs = npt_ss.parse::<f32>();
                    match (npt_hh, npt_mm, npt_secs) {
                        (Ok(hh), Ok(mm), Ok(secs)) => {
                            let npt_time =
                                f64::from(hh * 3600) + f64::from(mm * 60) + f64::from(secs);
                            Ok(Self::Time(npt_time))
                        }
                        _ => Err(Error::RangeNptTimeMalfored {
                            value: s.to_string(),
                        }),
                    }
                }
                _ => Err(Error::RangeNptTimeMalfored {
                    value: s.to_string(),
                }),
            },
        }
    }
}
