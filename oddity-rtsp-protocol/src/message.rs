use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use super::{parse::Parse, serialize::Serialize, Error};

pub use bytes::Bytes;
pub use http::uri::Uri;

pub trait Message: Serialize + fmt::Display {
    type Metadata: Parse;

    fn new(metadata: Self::Metadata, headers: Headers, body: Option<Bytes>) -> Self;
}

pub type Headers = BTreeMap<String, String>;

#[derive(Clone, PartialEq, Eq, Debug)]
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

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Describe => write!(f, "DESCRIBE"),
            Self::Announce => write!(f, "ANNOUNCE"),
            Self::Setup => write!(f, "SETUP"),
            Self::Play => write!(f, "PLAY"),
            Self::Pause => write!(f, "PAUSE"),
            Self::Record => write!(f, "RECORD"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Redirect => write!(f, "REDIRECT"),
            Self::Teardown => write!(f, "TEARDOWN"),
            Self::GetParameter => write!(f, "GET_PARAMETER"),
            Self::SetParameter => write!(f, "SET_PARAMETER"),
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DESCRIBE" => Ok(Self::Describe),
            "ANNOUNCE" => Ok(Self::Announce),
            "SETUP" => Ok(Self::Setup),
            "PLAY" => Ok(Self::Play),
            "PAUSE" => Ok(Self::Pause),
            "RECORD" => Ok(Self::Record),
            "OPTIONS" => Ok(Self::Options),
            "REDIRECT" => Ok(Self::Redirect),
            "TEARDOWN" => Ok(Self::Teardown),
            "GET_PARAMETER" => Ok(Self::GetParameter),
            "SET_PARAMETER" => Ok(Self::SetParameter),
            _ => Err(Error::MethodUnknown {
                method: s.to_string(),
            }),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Version {
    V1,
    V2,
    Unknown,
}

impl Default for Version {
    #[inline]
    fn default() -> Self {
        Self::V1
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::V1 => write!(f, "1.0"),
            Self::V2 => write!(f, "2.0"),
            Self::Unknown => write!(f, "?"),
        }
    }
}

pub type StatusCode = usize;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StatusCategory {
    Informational,
    Success,
    Redirection,
    ClientError,
    ServerError,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Continue,
    Ok,
    Created,
    LowonStorageSpace,
    MultipleChoices,
    MovedPermanently,
    MovedTemporarily,
    SeeOther,
    UseProxy,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Gone,
    LengthRequired,
    PreconditionFailed,
    RequestEntityTooLarge,
    RequestUriTooLong,
    UnsupportedMediaType,
    InvalidParameter,
    IllegalConferenceIdentifier,
    NotEnoughBandwidth,
    SessionNotFound,
    MethodNotValidInThisState,
    HeaderFieldNotValid,
    InvalidRange,
    ParameterIsReadOnly,
    AggregateOperationNotAllowed,
    OnlyAggregateOperationAllowed,
    UnsupportedTransport,
    DestinationUnreachable,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    RTSPVersionNotSupported,
    OptionNotSupported,
}

pub const fn status_to_code(status: Status) -> StatusCode {
    match status {
        Status::Continue => 100,
        Status::Ok => 200,
        Status::Created => 201,
        Status::LowonStorageSpace => 250,
        Status::MultipleChoices => 300,
        Status::MovedPermanently => 301,
        Status::MovedTemporarily => 302,
        Status::SeeOther => 303,
        Status::UseProxy => 305,
        Status::BadRequest => 400,
        Status::Unauthorized => 401,
        Status::PaymentRequired => 402,
        Status::Forbidden => 403,
        Status::NotFound => 404,
        Status::MethodNotAllowed => 405,
        Status::NotAcceptable => 406,
        Status::ProxyAuthenticationRequired => 407,
        Status::RequestTimeout => 408,
        Status::Gone => 410,
        Status::LengthRequired => 411,
        Status::PreconditionFailed => 412,
        Status::RequestEntityTooLarge => 413,
        Status::RequestUriTooLong => 414,
        Status::UnsupportedMediaType => 415,
        Status::InvalidParameter => 451,
        Status::IllegalConferenceIdentifier => 452,
        Status::NotEnoughBandwidth => 453,
        Status::SessionNotFound => 454,
        Status::MethodNotValidInThisState => 455,
        Status::HeaderFieldNotValid => 456,
        Status::InvalidRange => 457,
        Status::ParameterIsReadOnly => 458,
        Status::AggregateOperationNotAllowed => 459,
        Status::OnlyAggregateOperationAllowed => 460,
        Status::UnsupportedTransport => 461,
        Status::DestinationUnreachable => 462,
        Status::InternalServerError => 500,
        Status::NotImplemented => 501,
        Status::BadGateway => 502,
        Status::ServiceUnavailable => 503,
        Status::GatewayTimeout => 504,
        Status::RTSPVersionNotSupported => 505,
        Status::OptionNotSupported => 551,
    }
}

pub const fn status_to_reason(status: Status) -> &'static str {
    match status {
        Status::Continue => "Continue",
        Status::Ok => "OK",
        Status::Created => "Created",
        Status::LowonStorageSpace => "Low on Storage Space",
        Status::MultipleChoices => "Multiple Choices",
        Status::MovedPermanently => "Moved Permanently",
        Status::MovedTemporarily => "Moved Temporarily",
        Status::SeeOther => "See Other",
        Status::UseProxy => "Use Proxy",
        Status::BadRequest => "Bad Request",
        Status::Unauthorized => "Unauthorized",
        Status::PaymentRequired => "Payment Required",
        Status::Forbidden => "Forbidden",
        Status::NotFound => "Not Found",
        Status::MethodNotAllowed => "Method Not Allowed",
        Status::NotAcceptable => "Not Acceptable",
        Status::ProxyAuthenticationRequired => "Proxy Authentication Required",
        Status::RequestTimeout => "Request Timeout",
        Status::Gone => "Gone",
        Status::LengthRequired => "Length Required",
        Status::PreconditionFailed => "Precondition Failed",
        Status::RequestEntityTooLarge => "Request Entity Too Large",
        Status::RequestUriTooLong => "Request-URI Too Long",
        Status::UnsupportedMediaType => "Unsupported Media Type",
        Status::InvalidParameter => "Invalid parameter",
        Status::IllegalConferenceIdentifier => "Illegal Conference Identifier",
        Status::NotEnoughBandwidth => "Not Enough Bandwidth",
        Status::SessionNotFound => "Session Not Found",
        Status::MethodNotValidInThisState => "Method Not Valid In This State",
        Status::HeaderFieldNotValid => "Header Field Not Valid",
        Status::InvalidRange => "Invalid Range",
        Status::ParameterIsReadOnly => "Parameter Is Read-Only",
        Status::AggregateOperationNotAllowed => "Aggregate Operation Not Allowed",
        Status::OnlyAggregateOperationAllowed => "Only Aggregate Operation Allowed",
        Status::UnsupportedTransport => "Unsupported Transport",
        Status::DestinationUnreachable => "Destination Unreachable",
        Status::InternalServerError => "Internal Server Error",
        Status::NotImplemented => "Not Implemented",
        Status::BadGateway => "Bad Gateway",
        Status::ServiceUnavailable => "Service Unavailable",
        Status::GatewayTimeout => "Gateway Timeout",
        Status::RTSPVersionNotSupported => "RTSP Version Not Supported",
        Status::OptionNotSupported => "Option Not Supported",
    }
}
