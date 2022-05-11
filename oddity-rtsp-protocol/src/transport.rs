use std::net::IpAddr;
use std::str::FromStr;
use std::fmt;

use super::{
  Method,
  Error,
};

pub struct Transport {
  lower: Option<Lower>,
  parameters: Vec<Parameter>,
}

impl fmt::Display for Transport {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "RTP/AVP")?;
    if let Some(lower) = self.lower.as_ref() {
      write!(f, "/{}", lower)?;
    }
    for parameter in self.parameters.iter() {
      write!(f, ";{}", parameter)?;
    }
    writeln!(f)
  }

}

impl FromStr for Transport {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split(",");
    let protocol = parts
      .next()
      .ok_or_else(||
        Error::TransportProtocolProfileMissing {
          value: s.to_string(),
        }
      )?;

    let rest = parts
      .next()
      .ok_or_else(||
        Error::TransportProtocolProfileMissing {
          value: s.to_string(),
        }
      )?;

    let mut rest_parts = rest.split(";");
    let profile = rest_parts
      .next()
      .ok_or_else(||
        Error::TransportProtocolProfileMissing {
          value: s.to_string(),
        }
      )?;

    // TODO THIS DOES NOT HANDLE CASE WHERE LOWER IS IN STRING
    let parameters = parts
      .next()
      .map(|s| {
        
      })
      .unwrap_or(Vec::new());

    Ok(
      Transport {
        lower: lower,
        parameters: parameters,
      }
    )
  }

}

pub enum Lower {
  Tcp,
  Udp,
}

impl fmt::Display for Lower {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Lower::Tcp => write!(f, "TCP"),
      Lower::Udp => write!(f, "UDP"),
    }
  }

}

impl FromStr for Lower {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "TCP" => Ok(Lower::Tcp),
      "UDP" => Ok(Lower::Udp),
      _     => Err(
        Error::TransportLowerUnknown {
          value: s.to_string(),
        },
      ),
    }
  }

}

pub enum Parameter {
  Unicast,
  Multicast,
  Destination(IpAddr),
  Interleaved(Channel),
  Append,
  Ttl(usize),
  Layers(usize),
  Port(Port),
  ClientPort(Port),
  ServerPort(Port),
  Ssrc(String),
  Mode(Method),
}

impl fmt::Display for Parameter {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Parameter::Unicast => {
        write!(f, "unicast")
      },
      Parameter::Multicast => {
        write!(f, "multicast")
      },
      Parameter::Destination(host) => {
        write!(f, "destination={}", host)
      },
      Parameter::Interleaved(channel) => {
        write!(f, "interleaved={}", channel)
      },
      Parameter::Append => {
        write!(f, "append")
      },
      Parameter::Ttl(ttl) => {
        write!(f, "ttl={}", ttl)
      },
      Parameter::Layers(layers) => {
        write!(f, "layers={}", layers)
      },
      Parameter::Port(port) => {
        write!(f, "port={}", port)
      },
      Parameter::ClientPort(client_port) => {
        write!(f, "client_port={}", client_port)
      },
      Parameter::ServerPort(server_port) => {
        write!(f, "server_port={}", server_port)
      },
      Parameter::Ssrc(ssrc) => {
        write!(f, "ssrc={}", ssrc)
      },
      Parameter::Mode(method) => {
        write!(f, "mode=\"{}\"", method)
      },
    }
  }

}

impl FromStr for Parameter {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split("=");
    let var = parts
      .next()
      .ok_or_else(||
        Error::TransportParameterInvalid {
          parameter: s.to_string(),
        }
      )?;

    let mut val_or_err = || {
      parts
        .next()
        .ok_or_else(||
          Error::TransportParameterValueMissing {
            var: var.to_string(),
          }
        )
    };

    fn parse_or_err<T: FromStr>(
      var: &str,
      val: &str
    ) -> Result<T, Error> {
      val
        .parse::<T>()
        .map_err(|_|
          Error::TransportParameterValueInvalid {
            var: var.to_string(),
            val: val.to_string(),
          }
        )
    }

    match var {
      // TODO check not both
      "unicast"       => Ok(Parameter::Unicast),
      "multicast"     => Ok(Parameter::Multicast),
      "destination"   => {
        let val = val_or_err()?;
        let host = parse_or_err(var, val)?;
        Ok(Parameter::Destination(host))
      },
      "interleaved"   => {
        let val = val_or_err()?;
        let channel = parse_or_err(var, val)?;
        Ok(Parameter::Interleaved(channel))
      },
      "append"         => Ok(Parameter::Append),
      "ttl"            => {
        let val = val_or_err()?;
        let ttl = parse_or_err(var, val)?;
        Ok(Parameter::Ttl(ttl))
      },
      "layers"         => {
        let val = val_or_err()?;
        let layers = parse_or_err(var, val)?;
        Ok(Parameter::Layers(layers))
      },
      "port"           => {
        let val = val_or_err()?;
        let port = parse_or_err(var, val)?;
        Ok(Parameter::Port(port))
      },
      "client_port"    => {
        let val = val_or_err()?;
        let port = parse_or_err(var, val)?;
        Ok(Parameter::ClientPort(port))
      },
      "server_port"    => {
        let val = val_or_err()?;
        let port = parse_or_err(var, val)?;
        Ok(Parameter::ServerPort(port))
      },
      "ssrc"           => {
        let val = val_or_err()?;
        Ok(Parameter::Ssrc(val.to_string()))
      },
      "mode"           => {
        let val = val_or_err()?;
        let method = parse_or_err(var, val)?;
        Ok(Parameter::Mode(method))
      },
      _ => Err(
        Error::TransportParameterUnknown {
          var: var.to_string(),
        }
      ),
    }
  }

}

pub enum Channel {
  Single(u16),
  Range(u16, u16),
}

impl fmt::Display for Channel {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Channel::Single(channel) => {
        write!(f, "{}", channel)
      },
      Channel::Range(channel_1, channel_2) => {
        write!(f, "{}-{}", channel_1, channel_2)
      }
    }
  }

}

impl FromStr for Channel {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split(",");
    let channel_1 = parts
      .next()
      .and_then(|channel| channel.parse::<u16>().ok())
      .ok_or_else(|| Error::TransportChannelMalformed { value: s.to_string(), })?;
    let channel_2 = parts
      .next()
      .map(|channel| channel
        .parse::<u16>()
        .map_err(|_| Error::TransportChannelMalformed { value: s.to_string(), })
      );

    Ok(
      if let Some(channel_2) = channel_2 {
        Channel::Range(channel_1, channel_2?)
      } else {
        Channel::Single(channel_1)
      }
    )
  }

}

pub enum Port {
  Single(u16),
  Range(u16, u16),
}

impl fmt::Display for Port {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Port::Single(port) => {
        write!(f, "{}", port)
      },
      Port::Range(port_1, port_2) => {
        write!(f, "{}-{}", port_1, port_2)
      }
    }
  }

}

impl FromStr for Port {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split(",");
    let port_1 = parts
      .next()
      .and_then(|port| port.parse::<u16>().ok())
      .ok_or_else(|| Error::TransportPortMalformed { value: s.to_string(), })?;
    let port_2 = parts
      .next()
      .map(|port| port
        .parse::<u16>()
        .map_err(|_| Error::TransportPortMalformed { value: s.to_string(), })
      );

    Ok(
      if let Some(port_2) = port_2 {
        Port::Range(port_1, port_2?)
      } else {
        Port::Single(port_1)
      }
    )
  }

}