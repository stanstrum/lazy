use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fs::File;

/// The default window of bytes we will check at a time for
/// newlines surrounding a PrintableMessage
const WINDOW: usize = 64;

/// Takes `reader` and seeks to `end`.  Looks for the
/// trailing newline (or EOF), goes to the byte that comes
/// before it, and returns its position.
///
/// This methods flushes the reader's internal buffer.
pub(super) fn find_ending_newline(reader: &mut BufReader<File>, end: usize) -> usize {
  // Seek to the end of the reader to find the last position
  let length = reader.seek(SeekFrom::End(0))
    .expect("seek to end");

  // Seek to the range end to look for the ending newline
  reader.seek(SeekFrom::Start(end as _)).expect("seek to range end");

  let mut buffer = [0u8; WINDOW];
  loop {
    // Get the current stream position
    let stream_position = reader.stream_position()
      .expect("stream position");

    // Calculate how many bytes are left
    let remaining = length - stream_position;

    // If we're at the end, no newline was found
    if remaining == 0 {
      break;
    };

    // Make sure the window isn't bigger than the remainder
    // of the stream
    let this_window = WINDOW.min(remaining as _);

    // Read the window to a byte buffer.  Note that this seeks us to the end of
    // the window.
    let window = &mut buffer[..this_window];
    reader.read_exact(window).expect("read window");

    // This result yields how many bytes into the window the newline is.
    // If the newline is in the 0 position, we want to seek _zero_ bytes so as to
    // not include this newline in the result.  TODO: is this correct?
    let search = window.iter().position(|ch| *ch == b'\n');

    if let Some(offset) = search {
      // As above, seek `offset` bytes
      reader.seek_relative(offset as _).expect("seek forwards");
      break;
    };

    // If no newline is found here, stay where we are -- at the end of the
    // window -- and try the next window
  };

  reader.stream_position().expect("stream position") as _
}

/// Takes `reader` and seeks to `start`.  Looks for the
/// preceding newline (or start of file), goes to the byte
/// that comes after it, and returns its position.
///
/// This methods flushes the reader's internal buffer.
pub(super) fn find_starting_newline(reader: &mut BufReader<File>, start: usize) -> usize {
  // Bring the reader to the start of the range
  reader.seek(SeekFrom::Start(start as _))
    .expect("seek to start failed");

  let mut buffer = [0u8; WINDOW];
  loop {
    // Get the current stream position
    let stream_position = reader.stream_position()
      .expect("stream position");

    // If we're at the beginning, we can't go back any further
    if stream_position == 0 {
      break;
    };

    // Make sure we aren't trying to seek past the beginning
    // of the file in the case that our window is greater
    // than the current stream position
    let this_window = WINDOW.min(stream_position as _);
    reader.seek_relative(-(this_window as i64))
      .expect("seek backwards");

    // Read the window to a byte buffer
    let window = &mut buffer[..this_window];
    reader.read_exact(window).expect("read window");
    // ^ `read_exact` advances the stream back to where we
    // started from.

    // Find the last newline -- note that these are UTF-8
    // bytes, however the special code bytes have the high-
    // bit set to distinguish them from ordinary ASCII.
    // Just look for `0x0A`.
    let search = window.iter()
      // We're looking for an offset from the end
      .rev()
      .position(|ch| *ch == b'\n');

    // If it's found here, rewind to that newline
    if let Some(offset) = search {
      // Note that, if the newline is at the very end of this
      // buffer, then we need to rewind _one_ byte.  That is,
      // if the newline is at position zero (of the reversed
      // iterator), there's an off-by-one here.  We'll use
      // this to our advantage, since we actually want to
      // rewind to the character _after_ the newline.
      reader.seek_relative(-(offset as i64))
        .expect("seek backwards");
      break;
    };

    // Otherwise, seek back and try the next window
    reader.seek_relative(-(this_window as i64))
      .expect("seek backwards");
  };

  reader.stream_position().expect("stream position") as _
}
