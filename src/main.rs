pub mod map;

use anyhow::{Context, Result};
use clap::Parser;
use midly::{
    num::{u28, u4, u7},
    MetaMessage, MidiMessage, Smf, Track, TrackEvent, TrackEventKind,
};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, path, str};
use log::log;

#[derive(Parser)]
struct Args {
    /// Input midi file
    input_filepath: PathBuf,

    /// output path
    output_filepath: PathBuf, // TODO: make optional

    /// Input midi map // TODO: default to GM
    #[arg(short, long)]
    input_map_path: Option<PathBuf>,

    /// Output midi map
    #[arg(short, long)]
    output_map_path: Option<PathBuf>,

    /// Channel number to convert
    #[arg(short = 'n', long)]
    track_number: Option<usize>,
}

fn get_input_files(args: Args) -> Result<Vec<PathBuf>> {
    Ok(vec![args.input_filepath])
}

fn main() {
    let args = Args::parse();
    let input = fs::read(&args.input_filepath).unwrap();

    let mut smf = Smf::parse(&input).unwrap();

    let input_map = args.input_map_path.unwrap();
    let output_map = args.output_map_path.unwrap();

    // initialize map
    let conversion_map = map::make_conversion_map(&input_map, &output_map).unwrap();

    debug_smf(&smf);

    let track = pick_drum_track(smf.clone(), args.track_number).unwrap();
    let changed_track = convert_track(&track, &conversion_map);

    smf.tracks.remove(args.track_number.unwrap_or(0));
    smf.tracks
        .insert(args.track_number.unwrap_or(0), changed_track);

    // write output
    smf.save(args.output_filepath).unwrap();
}

fn convert_files(files: Vec<&Path>, output_dir: &Path, conversion_map: &HashMap<u7, u7>) {
    // Is there a os.walk alternative?
    for file in files.iter() {
        let mut output_file_path = PathBuf::new();
        output_file_path.push(output_dir);
        let file_name = file.file_name();
        if let None = file_name {
            eprintln!("failed extracting file name from {file}");
            continue
        }
        output_file_path.push(file_name.unwrap());
        convert_file(file, output_dir, conversion_map)
    }
}

fn convert_file<'a>(input_filepath: &Path, output_filepath: &Path, conversion_map: &HashMap<u7, u7>) -> Result<Track<'a>> {
    let input = fs::read(input_filepath).context("failed reading file")?;
    let mut smf = Smf::parse(&input).context("failed parsing midi file")?;

    debug_smf(&smf);

    // TODO: pass args.track_number
    let track_num: usize = 0;
    let track = pick_drum_track(smf.clone(), Some(track_num)).unwrap();
    let changed_track = convert_track(&track, &conversion_map);

    smf.tracks.remove(track_num);
    smf.tracks.insert(track_num, changed_track);

    // write output
    smf.save(output_filepath).into()
}

fn convert_track<'a>(track: &'a Track, conversion_map: &HashMap<u7, u7>) -> Track<'a> {
    let mut result = Track::new();
    for event in track.iter() {
        if let TrackEventKind::Midi { channel, message } = event.kind {
            match message {
                MidiMessage::NoteOff { key, vel } => {
                    result.push(make_track_event(
                        event.delta,
                        channel,
                        MidiMessage::NoteOff {
                            key: convert_event(key, conversion_map),
                            vel,
                        },
                    ));
                    continue;
                }
                MidiMessage::NoteOn { key, vel } => {
                    result.push(make_track_event(
                        event.delta,
                        channel,
                        MidiMessage::NoteOn {
                            key: convert_event(key, conversion_map),
                            vel,
                        },
                    ));
                    continue;
                }
                MidiMessage::Aftertouch { key, vel } => {
                    result.push(make_track_event(
                        event.delta,
                        channel,
                        MidiMessage::Aftertouch {
                            key: convert_event(key, conversion_map),
                            vel,
                        },
                    ));
                    continue;
                }
                _ => (),
            }
        }
        result.push(*event);
    }
    result
}

fn make_track_event(delta: u28, channel: u4, message: MidiMessage) -> TrackEvent<'static> {
    TrackEvent {
        delta,
        kind: TrackEventKind::Midi { channel, message },
    }
}

fn convert_event(input: u7, conversion_map: &HashMap<u7, u7>) -> u7 {
    // *conversion_map.get(&input).unwrap_or(&input)
    if let Some(value) = conversion_map.get(&input) {
        println!("converting {input} to {value}");
        return *value;
    }
    input
}

fn debug_smf(smf: &Smf) {
    eprintln!("---");
    eprintln!("file meta");
    eprintln!("tracks: {}", smf.tracks.len());
    smf.tracks.iter().for_each(|track| {
        eprint!("track:");
        if let Some(name) = track_name(track) {
            if let Ok(name) = str::from_utf8(name) {
                eprint!("{name}")
            } else {
                eprint!("Bad string - {name:?}")
            }
        } else {
            eprint!("No Event Found")
        }
        eprintln!()
    });
    eprintln!("---");
}

fn track_name<'a>(track: &'a Track) -> Option<&'a [u8]> {
    for event in track.iter() {
        if let TrackEventKind::Meta(MetaMessage::TrackName(name)) = event.kind {
            return Some(name);
        }
    }
    None
}

fn pick_drum_track(smf: Smf, track_number: Option<usize>) -> Option<Track> {
    smf.tracks.get(track_number.unwrap_or(0)).cloned()
}
