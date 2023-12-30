use std::convert;
use std::error;
use std::fmt;
use std::io;

use super::message::Uri;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// An error occurred decoding the header due to incorrect usage
    /// of text encoding by the sender.
    Encoding,
    /// The request line of the head part is malformed.
    RequestLineMalformed { line: String },
    /// The header first line does have a method and target URI, but
    /// it does not have a version, which is the required third part
    /// of the first line of the head.
    VersionMissing { line: String },
    /// The response status line does have a version, but does not have
    /// a status code which is required.
    StatusCodeMissing { line: String },
    /// The specified method is not a valid method.
    MethodUnknown { method: String },
    /// The header first line does have a method, but it does not have
    /// a target URI, which is the required second part of the first
    /// line of the head.
    UriMissing { line: String },
    /// The header first line has a Request-URI, but it could not be
    /// parsed correctly.
    UriMalformed { line: String, uri: String },
    /// The Request-URI is correct, but represents a relative path,
    /// which is not allowed in RTSP.
    UriNotAbsolute { uri: Uri },
    /// The response status line has a version and status code, but is
    /// missing a reason phrase which is required.
    ReasonPhraseMissing { line: String },
    /// The version specifier is incorrect. It should start with "RTSP/"
    /// followed by a digit, "." and another digit.
    VersionMalformed { line: String, version: String },
    /// The provided status code is not an unsigned integer or cannot be
    /// converted to one. It must be a 3-digit non-negative number.
    StatusCodeNotInteger { line: String, status_code: String },
    /// Header line is malformed.
    HeaderMalformed { line: String },
    /// The Content-Length header is missing, but it is required.
    ContentLengthMissing,
    /// The Content-Length header is not an integer value, or cannot be
    /// converted to an unsigned integer.
    ContentLengthNotInteger { value: String },
    /// This occurs when the caller invokes the state machine with a
    /// state that signals that parsing the head part of the request
    /// was already done before.
    HeadAlreadyDone,
    /// This occurs when the caller invokes the state machine with a
    /// state that signals that parsing the body part of the request
    /// was already done before.
    BodyAlreadyDone,
    /// Metadata was not parsed for some reason.
    MetadataNotParsed,
    /// This occurs when the caller tries to turn the parser into an
    /// actual request, but the parser was not ready yet.
    NotDone,
    /// This occurs when trying to serialize a request that does not
    /// have a known version.
    VersionUnknown,
    /// Transport header does not have protocol and profile string.
    /// The transport must start with `RTP/AVP`, where `RTP` denotes
    /// the protocol and `AVP` the profile.
    TransportProtocolProfileMissing { value: String },
    /// Transport header contains unknown lower protocol. Use either
    /// `TCP` or `UDP`.
    TransportLowerUnknown { value: String },
    /// Transport header contains unknown parameter. Please see RFC
    /// 2326 Section 12.39 for a list of permissable parameters.
    TransportParameterUnknown { var: String },
    /// Transport header contains parameter that should have a value,
    /// but does not have one.
    TransportParameterValueMissing { var: String },
    /// Transport header contains parameter with invalid value.
    TransportParameterValueInvalid { var: String, val: String },
    /// Transport header contains invalid or malformed parameter.
    TransportParameterInvalid { parameter: String },
    /// Transport header channel is malformed.
    TransportChannelMalformed { value: String },
    /// Transport header port is malformed.
    TransportPortMalformed { value: String },
    /// Tried to parse interleaved data but there is no interleaved
    /// header. Interleaved packets always start with `$` (0x24).
    InterleavedInvalid,
    /// Interleaved payload too large. The size cannot be larger than
    /// the maximum value of a 16-bit unsigned integer.
    InterleavedPayloadTooLarge,
    /// Range header value malformed.
    RangeMalformed { value: String },
    /// Parser does not support provided `Range` header unit.
    RangeUnitNotSupported { value: String },
    /// Parser does not support effective time in `Range` header.
    RangeTimeNotSupported { value: String },
    /// The NPT time (either the from or to part of the time specifier)
    /// is malformed.
    RangeNptTimeMalfored { value: String },
    /// RTP Info must always contain a URL.
    RtpInfoUrlMissing { value: String },
    /// RTP Info parameter is not known. This means that the RTP part
    /// contains an unknown or non-existant parameter variable.
    RtpInfoParameterUnknown { value: String },
    /// RTP Info parameter is invalid. This happens, for example, when
    /// the `seq` parameter is not an integer.
    RtpInfoParameterInvalid { value: String },
    /// RTP Info contains unexpected extra parameter.
    RtpInfoParameterUnexpected { value: String },
    /// Underlying socket was shut down. This is not really an error and
    /// consumers are expected to handle it gracefully.
    Shutdown,
    /// I/O error occurred.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Encoding => write!(f, "encoding incorrect"),
            Self::RequestLineMalformed { line } => write!(f, "request line malformed: {line}"),
            Self::VersionMissing { line } => {
                write!(f, "version missing in request line: {line}")
            }
            Self::StatusCodeMissing { line } => {
                write!(f, "status code missing in response line: {line}")
            }
            Self::MethodUnknown { method } => write!(f, "method unknown: {method}"),
            Self::UriMissing { line } => write!(f, "uri missing in request line: {line}"),
            Self::UriMalformed { line, uri } => {
                write!(f, "uri malformed: {uri} (in line: {line})")
            }
            Self::UriNotAbsolute { uri } => {
                write!(f, "uri must be absolute, but it is relative: {uri}")
            }
            Self::ReasonPhraseMissing { line } => {
                write!(f, "reason phrase missing in response line: {line}")
            }
            Self::VersionMalformed { line, version } => {
                write!(f, "version malformed: {version} (in line: {line})")
            }
            Self::StatusCodeNotInteger { line, status_code } => write!(
                f,
                "response has invalid status code: {status_code} (in response line: {line})",
            ),
            Self::HeaderMalformed { line } => write!(f, "header line malformed: {line}"),
            Self::ContentLengthMissing => write!(f, "request does not have Content-Length header"),
            Self::ContentLengthNotInteger { value } => {
                write!(f, "request has invalid value for Content-Length: {value}",)
            }
            Self::HeadAlreadyDone => write!(f, "head already done (cycle in state machine)"),
            Self::BodyAlreadyDone => write!(f, "body already done (cycle in state machine)"),
            Self::MetadataNotParsed => write!(f, "metadata not parsed"),
            Self::NotDone => write!(f, "parser not done yet"),
            Self::VersionUnknown => write!(f, "response has unknown version"),
            Self::TransportProtocolProfileMissing { value } => {
                write!(f, "transport protocol and/or profile missing: {value}")
            }
            Self::TransportLowerUnknown { value } => {
                write!(f, "transport lower protocol unknown: {value}")
            }
            Self::TransportParameterUnknown { var } => {
                write!(f, "transport parameter unknown: {var}")
            }
            Self::TransportParameterValueMissing { var } => write!(
                f,
                "transport parameter should have value but does not (var: {var})",
            ),
            Self::TransportParameterValueInvalid { var, val } => write!(
                f,
                "transport parameter value is invalid or malformed (var: {var}, val: {val})",
            ),
            Self::TransportParameterInvalid { parameter } => {
                write!(f, "transport parameter invalid: {parameter}")
            }
            Self::TransportChannelMalformed { value } => {
                write!(f, "transport channel malformed: {value}")
            }
            Self::TransportPortMalformed { value } => {
                write!(f, "transport port malformed: {value}")
            }
            Self::InterleavedInvalid => write!(
                f,
                "interleaved data does not have valid header magic character"
            ),
            Self::InterleavedPayloadTooLarge => write!(f, "interleaved payload too large"),
            Self::RangeMalformed { value } => write!(f, "range malformed: {value}"),
            Self::RangeUnitNotSupported { value } => {
                write!(f, "range unit not supported: {value}")
            }
            Self::RangeTimeNotSupported { value } => {
                write!(f, "range time not supported: {value}")
            }
            Self::RangeNptTimeMalfored { value } => {
                write!(f, "range npt time malformed: {value}")
            }
            Self::RtpInfoUrlMissing { value } => write!(f, "rtp info url missing: {value}"),
            Self::RtpInfoParameterUnknown { value } => {
                write!(f, "rtp info parameter unknown: {value}")
            }
            Self::RtpInfoParameterInvalid { value } => {
                write!(f, "rtp info parameter invalid: {value}")
            }
            Self::RtpInfoParameterUnexpected { value } => {
                write!(f, "rtp info contains unexpected parameter: {value}")
            }
            Self::Shutdown => write!(f, "underlying socket was shut down"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl error::Error for Error {}
