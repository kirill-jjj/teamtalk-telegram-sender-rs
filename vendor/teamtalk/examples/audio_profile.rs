use teamtalk::Client;
use teamtalk::client::audio::AudioDeviceProfile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    let profile = client.default_audio_profile();
    let _ = client.apply_audio_profile(profile);
    let duplex = AudioDeviceProfile::duplex(profile.input_id, profile.output_id);
    let _ = client.apply_audio_profile(duplex);
    Ok(())
}
