use crate::errors::Error;
use std::time::Duration;
use rand::Rng;
use tracing::warn;

use crate::input::mpd_status::MpdStatus;

// TODO make this stuff obj oriented
#[derive(Debug, Clone)]
pub struct Mpd;

pub fn add_from_playlist(
    mpd: &mut mpd::Client,
    name: &str,
    minimal_play_time: Duration,
    maximal_play_time: Duration,
) -> Result<(), mpd::error::Error> {
    let mut rng = rand::thread_rng();

    let songs = mpd.playlist(name);
    let mut songs = match songs {
        //report all errors except non existing playlist
        Ok(songs) => songs,
        Err(error) => match error {
            mpd::error::Error::Server(serv_error) => match serv_error.code {
                mpd::error::ErrorCode::NoExist => {
                    warn!("could not find playlist: {}", name);
                    return Ok(());
                }
                _ => return Err(mpd::error::Error::Server(serv_error).into()),
            },
            _ => return Err(error.into()),
        },
    };

    let mut time = Duration::from_secs(0);
    //add random songs until the playtime is larger then the minimum
    while time < minimal_play_time && songs.len() > 1 {
        let idx = rng.gen_range(0..(songs.len() - 1));
        let song = songs.remove(idx);
        if let Some(duration) = song.duration {
            if time + duration < maximal_play_time {
                time = time + song.duration.unwrap();
                mpd.push(song)?;
            }
        }
    }
    Ok(())
}

pub fn increase_volume(mpd_status: &mut MpdStatus) -> Result<(), Error> {
    const VOLUME_INCREMENT: i8 = 5;

    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    let current_volume = mpd_status.get_volume();
    if current_volume + VOLUME_INCREMENT > 100 {
        return Ok(());
    }

    client.volume(current_volume + VOLUME_INCREMENT)?;
    Ok(())
}

pub fn decrease_volume(mpd_status: &mut MpdStatus) -> Result<(), Error> {
    const VOLUME_INCREMENT: i8 = 5;

    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    let current_volume = mpd_status.get_volume();
    if current_volume - VOLUME_INCREMENT < 0 {
        return Ok(());
    }

    client.volume(current_volume - VOLUME_INCREMENT)?;
    Ok(())
}

pub fn next_song() -> Result<(), Error> {
    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    client.next()?;
    Ok(())
}

pub fn prev_song() -> Result<(), Error> {
    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    client.prev()?;
    Ok(())
}

pub fn toggle_playback(mpd_status: &mut MpdStatus) -> Result<(), Error> {
    let mut client = mpd::Client::connect("127.0.0.1:6600")?;

    match mpd_status.is_playing() {
        mpd::status::State::Stop => client.play()?,
        mpd::status::State::Pause => client.toggle_pause()?,
        mpd::status::State::Play => client.toggle_pause()?,
    }
    Ok(())
}

pub fn pause() -> Result<(), Error> {
    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    client.pause(true)?;
    Ok(())
}

pub fn save_current_playlist(mpd: &mut mpd::Client) -> Result<(), mpd::error::Error> {
    mpd.pl_remove("temp")?;
    mpd.save("temp")?;
    Ok(())
}
