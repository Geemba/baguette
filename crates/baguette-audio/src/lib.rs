unsafe impl Sync for AudioPlayer{}

pub struct AudioPlayer
{
    clips : Box<[rodio::Sink]>,
    // these dudes need to stay here without dropping or audio playback will no longer work.
    #[allow(dead_code)] stream : rodio::OutputStream,
    #[allow(dead_code)] handle : rodio::OutputStreamHandle,
}

impl AudioPlayer
{   
    /// returns an audio player with the specified amount of simultaneously playable clips.
    /// setting the value to zero will set it to one instead
    pub fn new(max_playable_clips : usize) -> Self
    {  
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let clips = (0..max_playable_clips)
            .map(|_| rodio::Sink::try_new(&handle).unwrap())
            .collect::<Box<[rodio::Sink]>>();

        Self { clips, stream, handle }
    }

    /// plays the given audio file,
    /// if the player has reached the max amount of playable clips
    /// it will replace the first in the slice.
    /// 
    /// panics if the given path doesn't exist
    pub fn play<P: AsRef<std::path::Path>>(&self, path : P)
    {
        let source = rodio::Decoder::new(match std::fs::File::open(path)
        {
            Ok(file) => file,
            Err(err) => panic!("{}", err)
        }).unwrap();

        self.clips.iter()
        .find(|sink| sink.empty())
        .and_then(|sink| Some(sink.append(source)));
    }
}