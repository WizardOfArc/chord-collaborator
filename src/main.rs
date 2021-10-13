extern crate rand;
extern crate coreaudio;

use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat};
use coreaudio::audio_unit::render_callback::{self, data};
use std::f64::consts::PI;

use std::cmp;
use std::io;

use rand::Rng;

/// midi notes range between 0 and 127 - so u8 is quite enough
type Chord = (u8, u8, u8, u8);

/// what I want to do is
/// interactively build a chord progression
/// for phase 1 - it'll be text based, for phase 2 - actually play the sounds
fn main() {
    play_sound();
    println!("Welcome to Chord Collaborator!");
    println!("Would you like to start building your progression?");
    println!("Type 'yes' otherwise I'll bid you farewell.");
    let mut user_response = String::new();
    io::stdin().read_line(&mut user_response).unwrap();

    if user_response.trim() == "yes".to_string() {
        let mut prog = Vec::<Chord>::new();
        loop {
            let mut choice = String::new();
            let new_chord: Chord = generate_chord();
            print_progression(&prog);
            print!(">>>\t");
            print_chord(&new_chord);
            println!("What would you think?");
            println!("Please enter the corresponding number?");
            println!("\t1 - add chord to progression");
            println!("\t2 - done");
            println!("Any other input means I'll pick another chord");
            io::stdin().read_line(&mut choice).unwrap();
            if choice.trim() == "2".to_string() {
                break;
            }
            if choice.trim() == "1".to_string() {
                prog = add_to_progression(prog, &new_chord);
            }
        }
        println!("Here's your progression:");
        print_progression(&prog);
    }
    println!("Good Bye and Farewell!");
}

fn print_progression(prog: &Vec<Chord>){
    for chord in prog {
        print_chord(chord);
    }
}

fn print_chord(chord: &Chord){
    println!("{}", chord_to_string(chord));
}

fn generate_chord() -> Chord {
    // randomly generate 4 notes that fit
    // standard choral ranges
    let mut rng = rand::thread_rng();
    let bass: u8 = rng.gen_range(40..65);
    let tenor: u8 = rng.gen_range(bass..cmp::min(69, bass+25));
    let alto: u8 = rng.gen_range(tenor..cmp::min(74, tenor+13));
    let soprano: u8 = rng.gen_range(alto..cmp::min(80, alto+13));
    (bass, tenor, alto, soprano)
}

fn add_to_progression(mut progression: Vec<Chord>, chord: &Chord) -> Vec<Chord>{
    progression.push(*chord);
    progression
}

fn pitch_class_symbol(pitch_class: &u8) -> &str {
    match *pitch_class {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "Eb",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "Ab",
        9 => "A",
        10 => "Bb",
        11 => "B",
        _ => "",
    }
}

fn note_val_to_symbol(note: &u8) -> String {
    let pc = *note % 12;
    let symbol = pitch_class_symbol(&pc);
    let octave: i32 = ((*note - pc)/12).into();
    format!("{}{}", symbol, octave-1)
}

fn chord_to_string(chord: &Chord) -> String {
    format!("[{} {} {} {}]",
        note_val_to_symbol(&(chord.0)),
        note_val_to_symbol(&(chord.1)),
        note_val_to_symbol(&(chord.2)),
        note_val_to_symbol(&(chord.3)),
    )
}

struct SineWaveGenerator {
    time: f64,
    freq: f64,
    volume: f64,
}

impl SineWaveGenerator {
    fn new(freq: f64, volume: f64) -> Self {
        SineWaveGenerator {
            time: 0.,
            freq,
            volume,
        }
    }
}

impl Iterator for SineWaveGenerator {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        self.time += 1. / 44_100.;
        let output = ((self.time * PI * 2.).sin() * self.volume) as f32;
        Some(output)
    }
}

fn play_sound() -> Result<(), coreaudio::Error> {
    let frequency_hz = 440.;
    let volume = 0.15;
    let mut samples = SineWaveGenerator::new(frequency_hz, volume);

    // Construct an Output audio unit that delivers audio to the default output device.
    let mut audio_unit = AudioUnit::new(IOType::DefaultOutput)?;

    let stream_format = audio_unit.output_stream_format()?;
    println!("{:#?}", &stream_format);

    // For this example, our sine wave expects `f32` data.
    assert!(SampleFormat::F32 == stream_format.sample_format);

    type Args = render_callback::Args<data::NonInterleaved<f32>>;
    audio_unit.set_render_callback(move |args| {
        let Args { num_frames, mut data, .. } = args;
        for i in 0..num_frames {
            let sample = samples.next().unwrap();
            for channel in data.channels_mut() {
                channel[i] = sample;
            }
        }
        Ok(())
    })?;

    audio_unit.start()?;

    std::thread::sleep(std::time::Duration::from_millis(3000));

    Ok(())
}
