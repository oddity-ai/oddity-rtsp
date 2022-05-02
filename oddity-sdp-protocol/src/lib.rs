// TODO just for writing
pub struct Minimal {
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
  pub network_type: NetworkType,
  pub address_type: AddressType,
  pub address: String,
  /* a= */
  pub tags: Vec<Tag>,
  /* t= */
  pub timing: (u64, u64),
  /* ... */
  pub media: Vec<Media>,
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
  Udp,
  RtpAvp,
  RtpSAvp,
}