unsafe impl<const MAX_PLAYABLE_CLIPS : usize> Sync for AudioPlayer<MAX_PLAYABLE_CLIPS>{}

/// to change the max amount of playable clips simoultaneosly set a custom value for [MAX_PLAYABLE_CLIPS]
pub struct AudioPlayer<const MAX_PLAYABLE_CLIPS : usize = 4>
{
    clips : [rodio::Sink; MAX_PLAYABLE_CLIPS],
    // these dudes need to stay here without dropping or audio playback will no longer work.
    #[allow(dead_code)] stream : rodio::OutputStream,
    #[allow(dead_code)] handle : rodio::OutputStreamHandle,
}

impl<const MAX_PLAYABLE_CLIPS : usize> Default for AudioPlayer<MAX_PLAYABLE_CLIPS>
{
    fn default() -> Self 
    {
        Self::new()
    }
}

impl<const MAX_PLAYABLE_CLIPS : usize> AudioPlayer<MAX_PLAYABLE_CLIPS>
{   
    /// returns an audio player with the specified amount of simultaneously playable clips.
    /// setting the value to zero will set it to one instead.
    /// 
    /// the max amount of playable clips simoultaneosly is set by the constant value on the struct
    pub fn new() -> Self
    {  
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let clips = std::array::from_fn(|_| rodio::Sink::try_new(&handle).unwrap());

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
            Err(err) => panic!("{err}")
        }).unwrap();

        if let Some(sink) = self.clips.iter().find(|sink| sink.empty())
        {
            sink.append(source)
        }
    }
}