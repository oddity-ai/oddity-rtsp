use std::net::IpAddr;

// TODO just for writing
pub struct Sdp {
  /* v= */
  pub version: Version,
  /* o= */
  pub origin_username: String,
  pub origin_session_id: String,
  pub origin_session_version: String,
  pub origin_network_type: NetworkType,
  pub origin_address_type: AddressType,
  pub origin_unicast_address: String,
  /* s= */
  pub session_name: String,
  /* i= */
  pub session_description: Option<String>,
  /* c= */
  pub connection_network_type: NetworkType,
  pub connection_address_type: AddressType,
  pub connection_address: String,
  /* a= */
  pub tags: Vec<Tag>,
  /* t= */
  pub timing: (u64, u64),
  /* ... */
  pub media: Vec<Media>,
}

impl Sdp {

  pub fn new(
    origin: IpAddr,
    name: String,
    destination: IpAddr,
    time_range: TimeRange,
  ) -> Self {
    Self {
      version: Version::V0,
      origin_username: "-".to_string(),
      origin_session_id: 0.to_string(), // TODO current time NTP
      origin_session_version: 0.to_string(),
      origin_network_type: NetworkType::Internet,
      origin_address_type: ip_addr_type(&origin),
      origin_unicast_address: origin.to_string(),
      session_name: name,
      session_description: None,
      connection_network_type: NetworkType::Internet,
      connection_address_type: ip_addr_type(&destination),
      connection_address: destination.to_string(),
      tags: Vec::new(),
      timing: time_range.into(),
      media: Vec::new(),
    }
  }

  pub fn with_username(
    mut self,
    username: &str,
  ) -> Self {
    self.origin_username = username.to_string();
    self
  }

  pub fn with_session_version(
    mut self,
    version: usize,
  ) -> Self {
    self.origin_session_version = version.to_string();
    self
  }

  pub fn with_description(
    mut self,
    description: &str,
  ) -> Self {
    self.session_description = Some(description.to_string());
    self
  }

  pub fn with_tag(
    mut self,
    tag: Tag,
  ) -> Self {
    self.tags.push(tag);
    self
  }

  pub fn with_tags(
    mut self,
    tags: impl IntoIterator<Item=Tag>,
  ) -> Self {
    self.tags.extend(tags);
    self
  }

  pub fn with_media(
    mut self,
    kind: Kind,
    port: u16,
    protocol: Protocol,
    codec: Codec,
  ) -> Self {
    self.media.push(Media {
      kind,
      port,
      protocol,
      format: 96, // TODO binding
      tags: vec![
        Tag::Value("rtmap".to_string(), "".to_string()), // TODO based on codec
      ],
    });
    self
  }


}

fn ip_addr_type(addr: &IpAddr) -> AddressType {
  match addr {
    IpAddr::V4(_) => AddressType::IpV4,
    IpAddr::V6(_) => AddressType::IpV6,
  }
}

#[derive(Clone, Copy)]
pub enum TimeRange {
  Live,
  Playback {
    start: u64,
    end: u64,
  }
}

impl From<TimeRange> for (u64, u64) {

  fn from(time_range: TimeRange) -> (u64, u64) {
    match time_range {
      TimeRange::Live
        => (0, 0),
      TimeRange::Playback {
        start,
        end,
      } => (start, end),
    }
  }

}

pub struct Media {
  /* m= */
  pub kind: Kind,
  pub port: u16,
  pub protocol: Protocol,
  pub format: usize,
  /* a= */
  pub tags: Vec<Tag>,
}

pub struct Timing {
  pub start: u64,
  pub stop: u64,
}

pub enum Version {
  V0,
}

pub enum NetworkType {
  Internet,
}

pub enum AddressType {
  IpV4,
  IpV6,
}

pub enum Tag {
  Property(String),
  Value(String, String),
}

pub enum Kind {
  Video,
  Audio,
}

pub enum Protocol {
  RtpAvp,
  RtpSAvp,
}

pub enum Codec {
  H264,
}
