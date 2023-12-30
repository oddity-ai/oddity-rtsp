use std::fmt;

use super::{
    message::{Bytes, Headers, Message, Method, Uri, Version},
    range::Range,
    transport::Transport,
    Error,
};

#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: Headers,
    pub body: Option<Bytes>,
}

impl Message for Request {
    type Metadata = RequestMetadata;

    fn new(metadata: RequestMetadata, headers: Headers, body: Option<Bytes>) -> Self {
        Self {
            method: metadata.method,
            uri: metadata.uri,
            version: metadata.version,
            headers,
            body,
        }
    }
}

impl Request {
    pub const fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn path(&self) -> &str {
        self.uri.path().trim_end_matches('/')
    }

    pub fn require(&self) -> Option<&str> {
        self.headers.get("Require").map(String::as_str)
    }

    pub fn accept(&self) -> Vec<&str> {
        self.headers
            .get("Accept")
            .map(|val| val.split(',').map(str::trim).collect::<Vec<_>>())
            .unwrap_or_default()
    }

    pub fn session(&self) -> Option<&str> {
        self.headers.get("Session").map(String::as_str)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn transport(&self) -> Result<Vec<Transport>, Error> {
        self.headers.get("Transport").map_or_else(
            || Ok(Vec::new()),
            |value| {
                value
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<Vec<_>, _>>()
            },
        )
    }

    pub fn range(&self) -> Option<Result<Range, Error>> {
        self.headers.get("Range").map(|value| value.parse())
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Version: {}, Method: {}, Uri: {}",
            self.version, self.method, self.uri
        )?;

        if !self.headers.is_empty() {
            writeln!(f, "\nHeaders:")?;
            for (var, val) in &self.headers {
                writeln!(f, " - {}: {}", &var, &val)?;
            }
        }

        if let Some(body) = &self.body {
            writeln!(f, "[{} bytes]", body.len())?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RequestMetadata {
    method: Method,
    uri: Uri,
    version: Version,
}

impl RequestMetadata {
    pub(super) const fn new(method: Method, uri: Uri, version: Version) -> Self {
        Self {
            method,
            uri,
            version,
        }
    }
}
