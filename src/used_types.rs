pub struct Iter<T: Clone> {
    inner: Vec<T>
}

impl<T: Clone> Iterator for Iter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.len() == 0 { None } else { Some(self.inner.remove(0)) }
    }
}

/*#[derive(Debug)]
pub enum EncoderError {
    UnsupportedFormat(String)
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncoderError::UnsupportedFormat(format) => write!(f, "Unsupported format (must be \"bin\", \"octal\", or \"dance\"): {}", format)
        }
    }
}

impl std::error::Error for EncoderError {}*/

pub trait Bits {
    fn to_bits(&self) -> Vec<u8>;
}

pub trait Chunked<T: Clone> {
    fn next_chunk_of(&mut self, size: usize) -> Option<Vec<T>>;

    fn chunks_of(&mut self, size: usize) -> Iter<Vec<T>>;
}

// woah, at least take me out to dinner first
// vscode autofill is the funniest shit ever: ", encoder_utils.rs, before you start writing all over me with your string processing and your binary conversions and your stateful encoder state and your pattern matching and your flatmaps and your chunking and your whatever the fuck else you have in store for me, encoder_utils.rs. at least let me put on a nice dress and do my hair before you start writing all over me with your string processing and your binary conversions and your stateful encoder state and your pattern matching and your flatmaps and your chunking and your whatever the"
pub trait STRIP {
    fn try_strip_prefix(&self, pattern: &str) -> String;

    fn try_strip_suffix(&self, pattern: &str) -> String;
}

impl Bits for u8 {
    fn to_bits(&self) -> Vec<u8> {
        vec![
            self >> 7,
            (self << 1) >> 7,
            (self << 2) >> 7,
            (self << 3) >> 7,
            (self << 4) >> 7,
            (self << 5) >> 7,
            (self << 6) >> 7
        ]
    }
}

impl Bits for String {
    fn to_bits(&self) -> Vec<u8> {
        self
            .as_bytes()
            .iter()
            .flat_map(|s| s.to_bits())
            .collect()
    }
}

impl<T: Clone, I: Iterator<Item = T>> Chunked<T> for I {
    fn next_chunk_of(&mut self, size: usize) -> Option<Vec<T>> {
        let mut whole = vec![];
        
        for _ in 0..size {
            let next = self.next();
            if next.is_none() { return None; }
            else { whole.push(next.unwrap()); }
        }

        return Some(whole);
    }

    fn chunks_of(&mut self, size: usize) -> Iter<Vec<T>> {
        let mut ret = vec![];
        let mut chunk_opt = self.next_chunk_of(size);
        while chunk_opt.is_some() {
            ret.push(chunk_opt.unwrap());
            chunk_opt = self.next_chunk_of(size);
        }
        return Iter { inner: ret };
    }
}

impl STRIP for String {
    fn try_strip_prefix(&self, pattern: &str) -> String {
        return self.strip_prefix(pattern).unwrap_or(self).to_string();
    }

    fn try_strip_suffix(&self, pattern: &str) -> String {
        return self.strip_suffix(pattern).unwrap_or(self).to_string();
    }
}
