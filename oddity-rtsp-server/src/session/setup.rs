use std::error;
use std::fmt;
use std::net::SocketAddr;

use oddity_rtsp_protocol as rtsp;
use video_rs as video;

use crate::media::video::rtp_muxer;
use crate::media::MediaInfo;
use crate::net::connection::ResponseSenderTx;
use crate::session::transport;

pub struct SessionSetup {
    pub rtsp_transport: rtsp::Transport,
    pub rtp_muxer: video::rtp::RtpMuxer,
    pub rtp_target: SessionSetupTarget,
}

impl SessionSetup {
    pub async fn from_rtsp_candidate_transports(
        candidate_transports: impl IntoIterator<Item = rtsp::Transport>,
        media_info: MediaInfo,
        sender: ResponseSenderTx,
    ) -> Result<Self, SessionSetupError> {
        let transport = candidate_transports
            .into_iter()
            .find(transport::is_supported)
            .ok_or(SessionSetupError::TransportNotSupported)?;
        tracing::trace!(%transport, "selected transport");

        tracing::trace!("initializing muxer");
        rtp_muxer::make_rtp_muxer_builder()
            .await
            .map_err(SessionSetupError::Media)
            .and_then(|mut rtp_muxer_builder| {
                let resolved_transport = transport::resolve_transport(&transport);
                tracing::trace!(%resolved_transport, "resolved transport");
                let rtp_target =
                    SessionSetupTarget::from_rtsp_transport(&resolved_transport, sender)
                        .ok_or(SessionSetupError::DestinationInvalid)?;
                tracing::debug!(?rtp_target, "calculated target");

                for stream_info in media_info.streams {
                    tracing::trace!(stream_index = stream_info.index, "adding stream to muxer");
                    rtp_muxer_builder = rtp_muxer_builder
                        .with_stream(stream_info)
                        .map_err(SessionSetupError::Media)?;
                }

                let rtp_muxer = rtp_muxer_builder.build();
                Ok(Self {
                    rtsp_transport: resolved_transport,
                    rtp_muxer,
                    rtp_target,
                })
            })
    }
}

#[derive(Debug)]
pub enum SessionSetupTarget {
    RtpUdp(SendOverSocket),
    RtpTcp(SendInterleaved),
}

#[derive(Debug)]
pub struct SendOverSocket {
    pub rtp_remote: SocketAddr,
    pub rtcp_remote: SocketAddr,
}

#[derive(Debug)]
pub struct SendInterleaved {
    pub sender: ResponseSenderTx,
    pub rtp_channel: u8,
    pub rtcp_channel: u8,
}

impl SessionSetupTarget {
    pub fn from_rtsp_transport(
        rtsp_transport: &rtsp::Transport,
        sender: ResponseSenderTx,
    ) -> Option<Self> {
        Some(match rtsp_transport.lower_protocol()? {
            rtsp::Lower::Udp => {
                let client_ip_addr = rtsp_transport.destination()?;
                let (client_rtp_port, client_rtcp_port) = match rtsp_transport.client_port()? {
                    rtsp::Port::Single(rtp_port) => (*rtp_port, rtp_port + 1),
                    rtsp::Port::Range(rtp_port, rtcp_port) => (*rtp_port, *rtcp_port),
                };

                SessionSetupTarget::RtpUdp(SendOverSocket {
                    rtp_remote: (*client_ip_addr, client_rtp_port).into(),
                    rtcp_remote: (*client_ip_addr, client_rtcp_port).into(),
                })
            }
            rtsp::Lower::Tcp => {
                let (rtp_channel, rtcp_channel) = match rtsp_transport.interleaved_channel()? {
                    rtsp::Channel::Single(rtp_channel) => (*rtp_channel, rtp_channel + 1),
                    rtsp::Channel::Range(rtp_channel, rtcp_channel) => {
                        (*rtp_channel, *rtcp_channel)
                    }
                };

                SessionSetupTarget::RtpTcp(SendInterleaved {
                    sender,
                    rtp_channel,
                    rtcp_channel,
                })
            }
        })
    }
}

#[derive(Debug)]
pub enum SessionSetupError {
    TransportNotSupported,
    DestinationInvalid,
    Media(video::Error),
}

impl fmt::Display for SessionSetupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionSetupError::TransportNotSupported => write!(f, "transport not supported"),
            SessionSetupError::DestinationInvalid => write!(f, "destination invalid"),
            SessionSetupError::Media(error) => write!(f, "media error: {}", error),
        }
    }
}

impl error::Error for SessionSetupError {}
