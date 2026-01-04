use std::sync::mpsc;

use mpris_server::{
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Time,
    TrackId, Volume,
    zbus::{Result, fdo},
};

#[derive(Debug)]
pub enum MprisState {
    Play,
    Pause,
    PlayPause,
    Next,
    Previous,
    Shuffle(bool),
    Loop,
    Metadata,
    Volume(f64),
    Seek(i64),
    Stop,
    Position(i64),
}

pub struct MprisHandler {
    pub tx: mpsc::Sender<MprisState>,
}

impl RootInterface for MprisHandler {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _: bool) -> Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok("SanctumPlayer".to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("Sanctum.Player".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
}

impl PlayerInterface for MprisHandler {
    async fn next(&self) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::Next);
        Ok(())
    }

    async fn previous(&self) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::Previous);
        Ok(())
    }

    async fn pause(&self) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::Pause);
        Ok(())
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::PlayPause);
        Ok(())
    }

    async fn stop(&self) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::Stop);
        Ok(())
    }

    async fn play(&self) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::Play);
        Ok(())
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_shuffle(&self, shuffle: bool) -> Result<()> {
        let _ = self.tx.send(MprisState::Shuffle(shuffle));
        Ok(())
    }

    async fn position(&self) -> fdo::Result<Time> {
        Ok(Time::from_secs(69))
    }

    async fn set_position(&self, _track_id: TrackId, position: Time) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::Position(position.as_secs()));
        Ok(())
    }

    async fn open_uri(&self, uri: String) -> fdo::Result<()> {
        println!("OpenUri({uri})");
        Ok(())
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        Ok(PlaybackStatus::Paused)
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        println!("LoopStatus");
        Ok(LoopStatus::None)
    }

    async fn set_loop_status(&self, loop_status: LoopStatus) -> Result<()> {
        println!("SetLoopStatus({loop_status})");
        Ok(())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        println!("Rate");
        Ok(PlaybackRate::default())
    }

    async fn set_rate(&self, rate: PlaybackRate) -> Result<()> {
        println!("No support for set ({rate})");
        Ok(())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        Ok(Metadata::default())
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        Ok(Volume::default())
    }

    async fn set_volume(&self, volume: Volume) -> Result<()> {
        let _ = self.tx.send(MprisState::Volume(volume));
        Ok(())
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        let _ = self.tx.send(MprisState::Seek(offset.as_secs()));
        Ok(())
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        println!("MinimumRate");
        Ok(PlaybackRate::default())
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        println!("MaximumRate");
        Ok(PlaybackRate::default())
    }
}
