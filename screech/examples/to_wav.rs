use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use wavv::{Data, Wav};

pub fn to_wav_file(buffer: &[f32], sample_rate: usize, name: &str) -> Result<(), Box<dyn Error>> {
    let normalized: Vec<i16> = buffer
        .into_iter()
        .map(|x| (x * (i16::MAX as f32)) as i16)
        .collect();

    let wav = Wav::from_data(Data::BitDepth16(normalized), sample_rate, 1);

    let file = format!("./examples/{}.wav", name);
    let path = Path::new(&file);
    let mut file = File::create(&path)?;
    file.write_all(&wav.to_bytes())?;

    Ok(())
}

#[allow(dead_code)]
fn main() {}
