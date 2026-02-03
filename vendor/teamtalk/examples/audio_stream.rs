use teamtalk::client::ffi;
use teamtalk::types::UserId;

fn main() -> teamtalk::Result<()> {
    let client = teamtalk::Client::new()?;
    let sink = teamtalk::WriterSink::new(std::io::sink());
    let _sub =
        client.stream_audio_blocks(UserId(1), ffi::StreamType::STREAMTYPE_VOICE as u32, sink);
    Ok(())
}
