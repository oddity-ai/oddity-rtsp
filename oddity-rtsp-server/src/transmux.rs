// TODO
pub type Sink = ();

// TODO MOVE
pub enum Event {
  Play,
  Pause,
  Stop,
  Subscribe(Sink),
  Unsubscribe(Sink),
}

pub trait ControlMessageFromEvent {

  fn into_control_message_or_none(event: Event) -> Option<Self>;

}

pub trait Transmux {
  type ControlMessage: ControlMessageFromEvent;
  type Source;
  type Sink;

  fn transmux_loop(
    rx: Receiver<ControlMessage>,
    source: Source,
    sink: Sink
  );

}

pub struct FileLoop;

impl Transmux for FileLoop {

  fn transmux_loop(
    rx: Receiver<ControlMessage>,
    source: Source,
    sink: Sink
  ) {

  }

}

pub struct Stream;

impl Transmux for Stream {

  fn transmux_loop(
    rx: Receiver<ControlMessage>,
    source: Source,
    sink: Sink
  ) {

  }

}

pub struct StreamMultiplex;

impl Transmux for FileLoop {

  fn transmux_loop(
    rx: Receiver<ControlMessage>,
    source: Source,
    sink: Sink
  ) {

  }

}
