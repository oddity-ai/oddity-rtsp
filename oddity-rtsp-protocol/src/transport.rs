use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use super::{Error, Method};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transport {
    lower: Option<Lower>,
    parameters: Vec<Parameter>,
}

impl Transport {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lower: None,
            parameters: Vec::new(),
        }
    }

    #[must_use]
    pub const fn with_lower_protocol(mut self, lower: Lower) -> Self {
        self.lower = Some(lower);
        self
    }

    #[must_use]
    pub fn with_parameter(mut self, parameter: Parameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    #[must_use]
    pub fn with_parameters(mut self, parameters: impl IntoIterator<Item = Parameter>) -> Self {
        self.parameters.extend(parameters);
        self
    }

    #[must_use]
    pub const fn lower_protocol(&self) -> Option<&Lower> {
        self.lower.as_ref()
    }

    #[must_use]
    pub const fn parameters(&self) -> &impl IntoIterator<Item = Parameter> {
        &self.parameters
    }

    pub fn parameters_iter(&self) -> impl Iterator<Item = &Parameter> {
        self.parameters.iter()
    }

    #[must_use]
    pub fn destination(&self) -> Option<&IpAddr> {
        self.parameters_iter().find_map(|parameter| {
            if let Parameter::Destination(ip_addr) = parameter {
                Some(ip_addr)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn port(&self) -> Option<&Port> {
        self.parameters_iter().find_map(|parameter| {
            if let Parameter::Port(port) = parameter {
                Some(port)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn client_port(&self) -> Option<&Port> {
        self.parameters_iter().find_map(|parameter| {
            if let Parameter::ClientPort(port) = parameter {
                Some(port)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn server_port(&self) -> Option<&Port> {
        self.parameters_iter().find_map(|parameter| {
            if let Parameter::ServerPort(port) = parameter {
                Some(port)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn interleaved_channel(&self) -> Option<&Channel> {
        self.parameters_iter().find_map(|parameter| {
            if let Parameter::Interleaved(channel) = parameter {
                Some(channel)
            } else {
                None
            }
        })
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RTP/AVP")?;
        if let Some(lower) = self.lower.as_ref() {
            write!(f, "/{lower}")?;
        }
        for parameter in &self.parameters {
            write!(f, ";{parameter}")?;
        }
        Ok(())
    }
}

impl FromStr for Transport {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (spec, params) = s
            .split_once(';')
            .map_or_else(|| (s, None), |(spec, params)| (spec, Some(params)));

        if spec.starts_with("RTP/AVP") {
            let lower = spec.split('/').nth(2).map(str::parse).transpose()?;

            let parameters = params
                .map(|params| {
                    params
                        .split(';')
                        .map(str::parse)
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();

            Ok(Self { lower, parameters })
        } else {
            Err(Error::TransportProtocolProfileMissing {
                value: s.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lower {
    Tcp,
    Udp,
}

impl fmt::Display for Lower {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Tcp => write!(f, "TCP"),
            Self::Udp => write!(f, "UDP"),
        }
    }
}

impl FromStr for Lower {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TCP" => Ok(Self::Tcp),
            "UDP" => Ok(Self::Udp),
            _ => Err(Error::TransportLowerUnknown {
                value: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            Self::Unicast => {
                write!(f, "unicast")
            }
            Self::Multicast => {
                write!(f, "multicast")
            }
            Self::Destination(host) => {
                write!(f, "destination={host}")
            }
            Self::Interleaved(channel) => {
                write!(f, "interleaved={channel}")
            }
            Self::Append => {
                write!(f, "append")
            }
            Self::Ttl(ttl) => {
                write!(f, "ttl={ttl}")
            }
            Self::Layers(layers) => {
                write!(f, "layers={layers}")
            }
            Self::Port(port) => {
                write!(f, "port={port}")
            }
            Self::ClientPort(client_port) => {
                write!(f, "client_port={client_port}")
            }
            Self::ServerPort(server_port) => {
                write!(f, "server_port={server_port}")
            }
            Self::Ssrc(ssrc) => {
                write!(f, "ssrc={ssrc}")
            }
            Self::Mode(method) => {
                write!(f, "mode=\"{method}\"")
            }
        }
    }
}

fn parse_or_err<T: FromStr>(var: &str, value: &str) -> Result<T, Error> {
    value
        .parse::<T>()
        .map_err(|_| Error::TransportParameterValueInvalid {
            var: var.to_string(),
            val: value.to_string(),
        })
}

impl FromStr for Parameter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('=');
        let var = parts
            .next()
            .ok_or_else(|| Error::TransportParameterInvalid {
                parameter: s.to_string(),
            })?;

        let mut val_or_err = || {
            parts
                .next()
                .ok_or_else(|| Error::TransportParameterValueMissing {
                    var: var.to_string(),
                })
        };

        match var {
            "unicast" => Ok(Self::Unicast),
            "multicast" => Ok(Self::Multicast),
            "destination" => {
                let value = val_or_err()?;
                let host = parse_or_err(var, value)?;
                Ok(Self::Destination(host))
            }
            "interleaved" => {
                let value = val_or_err()?;
                let channel = parse_or_err(var, value)?;
                Ok(Self::Interleaved(channel))
            }
            "append" => Ok(Self::Append),
            "ttl" => {
                let value = val_or_err()?;
                let ttl = parse_or_err(var, value)?;
                Ok(Self::Ttl(ttl))
            }
            "layers" => {
                let value = val_or_err()?;
                let layers = parse_or_err(var, value)?;
                Ok(Self::Layers(layers))
            }
            "port" => {
                let value = val_or_err()?;
                let port = parse_or_err(var, value)?;
                Ok(Self::Port(port))
            }
            "client_port" => {
                let value = val_or_err()?;
                let port = parse_or_err(var, value)?;
                Ok(Self::ClientPort(port))
            }
            "server_port" => {
                let value = val_or_err()?;
                let port = parse_or_err(var, value)?;
                Ok(Self::ServerPort(port))
            }
            "ssrc" => {
                let value = val_or_err()?;
                Ok(Self::Ssrc(value.to_string()))
            }
            "mode" => {
                let value = val_or_err()?;
                let value = value
                    .strip_prefix('"')
                    .unwrap_or(value)
                    .strip_suffix('"')
                    .unwrap_or(value);
                let method = parse_or_err(var, value)?;
                Ok(Self::Mode(method))
            }
            _ => Err(Error::TransportParameterUnknown {
                var: var.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Channel {
    Single(u8),
    Range(u8, u8),
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Single(channel) => {
                write!(f, "{channel}")
            }
            Self::Range(channel_1, channel_2) => {
                write!(f, "{channel_1}-{channel_2}")
            }
        }
    }
}

impl FromStr for Channel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let channel_1 = parts
            .next()
            .and_then(|channel| channel.parse::<u8>().ok())
            .ok_or_else(|| Error::TransportChannelMalformed {
                value: s.to_string(),
            })?;
        let channel_2 = parts.next().map(|channel| {
            channel
                .parse::<u8>()
                .map_err(|_| Error::TransportChannelMalformed {
                    value: s.to_string(),
                })
        });

        Ok(if let Some(channel_2) = channel_2 {
            Self::Range(channel_1, channel_2?)
        } else {
            Self::Single(channel_1)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Port {
    Single(u16),
    Range(u16, u16),
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Single(port) => {
                write!(f, "{port}")
            }
            Self::Range(port_1, port_2) => {
                write!(f, "{port_1}-{port_2}")
            }
        }
    }
}

impl FromStr for Port {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let port_1 = parts
            .next()
            .and_then(|port| port.parse::<u16>().ok())
            .ok_or_else(|| Error::TransportPortMalformed {
                value: s.to_string(),
            })?;
        let port_2 = parts.next().map(|port| {
            port.parse::<u16>()
                .map_err(|_| Error::TransportPortMalformed {
                    value: s.to_string(),
                })
        });

        Ok(if let Some(port_2) = port_2 {
            Self::Range(port_1, port_2?)
        } else {
            Self::Single(port_1)
        })
    }
}

#[cfg(test)]
mod tests {

    use super::{Channel, Error, Lower, Method, Parameter, Port, Transport};

    #[test]
    fn parse_minimal() {
        assert_eq!("RTP/AVP".parse::<Transport>().unwrap(), Transport::new(),);
    }

    #[test]
    fn parse_lower_tcp() {
        assert_eq!(
            "RTP/AVP/TCP".parse::<Transport>().unwrap(),
            Transport::new().with_lower_protocol(Lower::Tcp),
        );
    }

    #[test]
    fn parse_lower_udp() {
        assert_eq!(
            "RTP/AVP/UDP".parse::<Transport>().unwrap(),
            Transport::new().with_lower_protocol(Lower::Udp),
        );
    }

    #[test]
    fn parse_unicast() {
        assert_eq!(
            "RTP/AVP;unicast".parse::<Transport>().unwrap(),
            Transport::new().with_parameter(Parameter::Unicast),
        );
    }

    #[test]
    fn parse_destination_missing_value() {
        assert!(matches!(
            "RTP/AVP/UDP;destination".parse::<Transport>(),
            Err(Error::TransportParameterValueMissing { var: _ }),
        ),);
    }

    #[test]
    fn parse_destination_ip() {
        assert_eq!(
            "RTP/AVP/UDP;destination=127.0.0.1"
                .parse::<Transport>()
                .unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Destination([127, 0, 0, 1].into())),
        );
    }

    #[test]
    fn parse_interleaved_invalid() {
        assert!(matches!(
            "RTP/AVP/UDP;interleaved=invalid".parse::<Transport>(),
            Err(Error::TransportParameterValueInvalid { var: _, val: _ }),
        ),);
    }

    #[test]
    fn parse_interleaved_channel() {
        assert_eq!(
            "RTP/AVP/UDP;interleaved=8-9".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Interleaved(Channel::Range(8, 9))),
        );
    }

    #[test]
    fn parse_layers() {
        assert_eq!(
            "RTP/AVP/UDP;layers=3".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Layers(3)),
        );
    }

    #[test]
    fn parse_port_single() {
        assert_eq!(
            "RTP/AVP/UDP;port=3".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Port(Port::Single(3))),
        );
    }

    #[test]
    fn parse_server_port_range() {
        assert_eq!(
            "RTP/AVP/UDP;server_port=3-4".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::ServerPort(Port::Range(3, 4))),
        );
    }

    #[test]
    fn parse_ssrc() {
        assert_eq!(
            "RTP/AVP/UDP;ssrc=ABCDEF".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Ssrc("ABCDEF".to_string())),
        );
    }

    #[test]
    fn parse_mode_method_unknown() {
        assert!(matches!(
            "RTP/AVP/UDP;mode=UNKNOWN".parse::<Transport>(),
            Err(Error::TransportParameterValueInvalid { var: _, val: _ }),
        ),);
    }

    #[test]
    fn parse_mode_method() {
        assert_eq!(
            "RTP/AVP/UDP;mode=PLAY".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Mode(Method::Play)),
        );
        assert_eq!(
            "RTP/AVP/UDP;mode=\"PLAY\"".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Mode(Method::Play)),
        );
    }

    #[test]
    fn parse_rfc2326_section_12_39_examples() {
        assert_eq!(
            "RTP/AVP;multicast;ttl=127;mode=\"PLAY\""
                .parse::<Transport>()
                .unwrap(),
            Transport::new()
                .with_parameter(Parameter::Multicast)
                .with_parameter(Parameter::Ttl(127))
                .with_parameter(Parameter::Mode(Method::Play)),
        );
        assert_eq!(
            "RTP/AVP;unicast;client_port=3456-3457;mode=\"PLAY\""
                .parse::<Transport>()
                .unwrap(),
            Transport::new()
                .with_parameter(Parameter::Unicast)
                .with_parameter(Parameter::ClientPort(Port::Range(3456, 3457)))
                .with_parameter(Parameter::Mode(Method::Play)),
        );
    }

    #[test]
    fn format_minimal() {
        assert_eq!(&Transport::new().to_string(), "RTP/AVP",);
    }

    #[test]
    fn format_lower_tcp() {
        assert_eq!(
            &Transport::new().with_lower_protocol(Lower::Tcp).to_string(),
            "RTP/AVP/TCP",
        );
    }

    #[test]
    fn format_lower_udp() {
        assert_eq!(
            &Transport::new().with_lower_protocol(Lower::Udp).to_string(),
            "RTP/AVP/UDP",
        );
    }

    #[test]
    fn format_unicast() {
        assert_eq!(
            &Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Unicast)
                .to_string(),
            "RTP/AVP/UDP;unicast",
        );
    }

    #[test]
    fn format_rfc2326_section_12_39_examples() {
        assert_eq!(
            &Transport::new()
                .with_parameter(Parameter::Multicast)
                .with_parameter(Parameter::Ttl(127))
                .with_parameter(Parameter::Mode(Method::Play))
                .to_string(),
            "RTP/AVP;multicast;ttl=127;mode=\"PLAY\"",
        );
        assert_eq!(
            &Transport::new()
                .with_parameter(Parameter::Unicast)
                .with_parameter(Parameter::ClientPort(Port::Range(3456, 3457)))
                .with_parameter(Parameter::Mode(Method::Play))
                .to_string(),
            "RTP/AVP;unicast;client_port=3456-3457;mode=\"PLAY\"",
        );
    }

    #[test]
    fn format_all_parameters() {
        assert_eq!(
      &Transport::new()
        .with_lower_protocol(Lower::Tcp)
        .with_parameter(Parameter::Unicast)
        .with_parameter(Parameter::Multicast)
        .with_parameter(Parameter::Destination([1, 2, 3, 4].into()))
        .with_parameter(Parameter::Interleaved(Channel::Range(12, 13)))
        .with_parameter(Parameter::Append)
        .with_parameter(Parameter::Ttl(999))
        .with_parameter(Parameter::Layers(2))
        .with_parameter(Parameter::Port(Port::Single(8)))
        .with_parameter(Parameter::ClientPort(Port::Range(9, 10)))
        .with_parameter(Parameter::ServerPort(Port::Range(11, 12)))
        .with_parameter(Parameter::Ssrc("01234ABCDEF".to_string()))
        .with_parameter(Parameter::Mode(Method::Describe))
        .to_string(),
      "RTP/AVP/TCP;unicast;multicast;destination=1.2.3.4;interleaved=12-13;append;ttl=999;layers=2;port=8;client_port=9-10;server_port=11-12;ssrc=01234ABCDEF;mode=\"DESCRIBE\"",
    );
    }
}
