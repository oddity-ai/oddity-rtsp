use oddity_rtsp_protocol::Transport;

// TODO turns out ffmpeg cannot natively mux RTP/AVP/TCP so we need to
// do some custom stuff (writing to buffer I think), ffserver does it like so:
// https://github.com/Malinskiy/ffmpeg/blob/master/ffserver.c

pub fn determine_transport(
  constraints: impl IntoIterator<Item=Transport>,
) -> Option<Transport> {
  constraints
    .into_iter()
    .filter_map(|constraint| {
      
    })
    .next()
}
