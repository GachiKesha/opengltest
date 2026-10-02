use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::error::Error;
use std::io::Cursor;

pub struct Music {
    _sink: MixerDeviceSink,
    player: Player,
    data: Vec<u8>,
}

impl Music {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut _sink = DeviceSinkBuilder::open_default_sink()?;
        _sink.log_on_drop(false);
        let player = Player::connect_new(_sink.mixer());
        let data = std::fs::read(path)?;
        Ok(Self { _sink, player, data,})
    }

    pub fn start(&self) {
        self.player.stop();
        match Decoder::try_from(Cursor::new(self.data.clone())) {
            Ok(source) => self.player.append(source),
            Err(err) => eprintln!("{}", err)
        };

    }

    pub fn stop(&self) {
        self.player.stop()
    }
}
