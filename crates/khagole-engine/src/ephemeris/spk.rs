//! Minimal reader for JPL SPK kernels (NAIF DAF container, segment types 2 and 3).
//!
//! Positions are returned in km and velocities in km/s, in the kernel's frame (ICRF/J2000
//! for the JPL planetary ephemerides), relative to the segment's center body.
//! Time is TDB seconds past J2000 ("ET"), see [`jd_tdb_to_et`].
//!
//! Format reference: NAIF "DAF Required Reading" and "SPK Required Reading".

use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

const RECORD_BYTES: usize = 1024;
const WORD_BYTES: usize = 8;
const J2000_JD: f64 = 2_451_545.0;
const SECONDS_PER_DAY: f64 = 86_400.0;

/// TDB Julian day → TDB seconds past J2000.
pub fn jd_tdb_to_et(jd_tdb: f64) -> f64 {
    (jd_tdb - J2000_JD) * SECONDS_PER_DAY
}

/// Failure reading or evaluating an SPK kernel.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SpkError {
    /// The file could not be read.
    Io(String),
    /// The file is not a valid SPK/DAF kernel.
    Format(String),
    /// A segment uses an SPK data type other than 2 or 3.
    UnsupportedType(i32),
    /// No segment for this center → target pair covers the requested time.
    OutOfRange {
        /// NAIF code of the target body.
        target: i32,
        /// NAIF code of the center body.
        center: i32,
        /// Requested time, TDB seconds past J2000.
        et: f64,
    },
    /// The kernel has no segment at all for this center → target pair.
    NoSegment {
        /// NAIF code of the target body.
        target: i32,
        /// NAIF code of the center body.
        center: i32,
    },
}

impl fmt::Display for SpkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpkError::Io(msg) => write!(f, "SPK I/O error: {msg}"),
            SpkError::Format(msg) => write!(f, "malformed SPK file: {msg}"),
            SpkError::UnsupportedType(t) => write!(f, "unsupported SPK segment type {t}"),
            SpkError::OutOfRange { target, center, et } => write!(
                f,
                "no SPK segment for {center} -> {target} covers ET {et:.1} s"
            ),
            SpkError::NoSegment { target, center } => {
                write!(f, "kernel has no segment for {center} -> {target}")
            }
        }
    }
}

impl std::error::Error for SpkError {}

impl From<std::io::Error> for SpkError {
    fn from(err: std::io::Error) -> Self {
        SpkError::Io(err.to_string())
    }
}

/// Where the kernel bytes live: fully in memory (WASM, mobile, small kernels) or read on
/// demand from disk (multi-GB kernels such as de441 on the server).
enum Storage {
    Memory(Vec<u8>),
    File(Mutex<FileStorage>),
}

/// Reads of a file-backed kernel kept for reuse: a chart evaluates the same few dozen
/// Chebyshev records hundreds of times (most of them in the sunrise search), and each
/// read from the file costs a seek and a read.
const CACHED_READS: usize = 64;

struct FileStorage {
    file: File,
    /// (byte offset, bytes) of recent reads.
    cache: Vec<(usize, Box<[u8]>)>,
    /// The cache slot the next miss replaces, round robin.
    next: usize,
}

impl Storage {
    fn file(file: File) -> Self {
        Storage::File(Mutex::new(FileStorage {
            file,
            cache: Vec::with_capacity(CACHED_READS),
            next: 0,
        }))
    }

    /// Size of the kernel in bytes.
    fn len(&self) -> Result<u64, SpkError> {
        match self {
            Storage::Memory(data) => Ok(data.len() as u64),
            Storage::File(file) => Ok(file
                .lock()
                .map_err(|_| SpkError::Io("poisoned lock".into()))?
                .file
                .metadata()?
                .len()),
        }
    }

    fn read_bytes(&self, offset: usize, buf: &mut [u8]) -> Result<(), SpkError> {
        match self {
            Storage::Memory(data) => {
                let end = offset
                    .checked_add(buf.len())
                    .filter(|end| *end <= data.len())
                    .ok_or_else(|| SpkError::Format("read past end of file".into()))?;
                buf.copy_from_slice(&data[offset..end]);
                Ok(())
            }
            Storage::File(storage) => {
                let mut storage = storage
                    .lock()
                    .map_err(|_| SpkError::Io("poisoned lock".into()))?;
                let FileStorage { file, cache, next } = &mut *storage;
                if let Some((_, bytes)) = cache
                    .iter()
                    .find(|(at, bytes)| *at == offset && bytes.len() == buf.len())
                {
                    buf.copy_from_slice(bytes);
                    return Ok(());
                }
                file.seek(SeekFrom::Start(offset as u64))?;
                file.read_exact(buf)?;
                let entry = (offset, Box::from(&*buf));
                if cache.len() < CACHED_READS {
                    cache.push(entry);
                } else {
                    cache[*next] = entry;
                    *next = (*next + 1) % CACHED_READS;
                }
                Ok(())
            }
        }
    }
}

/// One SPK segment: a center → target trajectory over a time span.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Segment name from the DAF name record (e.g. "DE-0440LE-0440").
    pub name: String,
    /// NAIF code of the center body.
    pub center: i32,
    /// NAIF code of the target body.
    pub target: i32,
    /// NAIF code of the reference frame (1 = J2000/ICRF).
    pub frame: i32,
    /// SPK data type (2: Chebyshev positions, 3: positions and velocities).
    pub data_type: i32,
    /// Start of coverage, TDB seconds past J2000.
    pub start_et: f64,
    /// End of coverage, TDB seconds past J2000.
    pub end_et: f64,
    /// 1-based DAF word addresses of the segment data.
    start_word: usize,
    // Chebyshev directory (types 2 and 3), read from the segment's last four words.
    init: f64,
    interval: f64,
    record_size: usize,
    record_count: usize,
}

impl Segment {
    /// Whether the segment covers `et` (TDB seconds past J2000).
    pub fn covers(&self, et: f64) -> bool {
        et >= self.start_et && et <= self.end_et
    }

    fn components(&self) -> usize {
        if self.data_type == 3 {
            6
        } else {
            3
        }
    }
}

/// A JPL SPK (DAF) kernel, read from a file or from memory.
pub struct Spk {
    storage: Storage,
    /// Size of the kernel in 8-byte words: every read is checked against it before its
    /// buffer is allocated, so a corrupt header cannot request gigabytes.
    words: usize,
    little_endian: bool,
    segments: Vec<Segment>,
}

impl fmt::Debug for Spk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Spk")
            .field("segments", &self.segments)
            .finish()
    }
}

/// Position (km) and velocity (km/s).
pub type State = ([f64; 3], [f64; 3]);

impl Spk {
    /// Load a kernel held in memory (e.g. fetched by a browser or bundled in an app).
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, SpkError> {
        Self::load(Storage::Memory(data))
    }

    /// Open a kernel on disk; segment data is read on demand, so size is not a concern.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SpkError> {
        Self::load(Storage::file(File::open(path)?))
    }

    /// The kernel's segments, in file order.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    fn load(storage: Storage) -> Result<Self, SpkError> {
        let mut header = [0u8; RECORD_BYTES];
        storage.read_bytes(0, &mut header)?;
        if &header[0..7] != b"DAF/SPK" {
            return Err(SpkError::Format("not a DAF/SPK file".into()));
        }
        let little_endian = match &header[88..96] {
            b"LTL-IEEE" => true,
            b"BIG-IEEE" => false,
            // Pre-1995 files lack LOCFMT: infer from ND, which is always 2 for SPK.
            _ => i32::from_le_bytes(header[8..12].try_into().unwrap()) == 2,
        };
        let words = usize::try_from(storage.len()? / WORD_BYTES as u64).unwrap_or(usize::MAX);
        let mut spk = Spk {
            storage,
            words,
            little_endian,
            segments: Vec::new(),
        };
        let nd = spk.int_at(&header, 8) as usize;
        let ni = spk.int_at(&header, 12) as usize;
        if nd != 2 || ni != 6 {
            return Err(SpkError::Format(format!("unexpected ND={nd} NI={ni}")));
        }
        let summary_words = nd + ni.div_ceil(2);

        let mut record_number = spk.address_at(&header, 76)?;
        let mut visited = 0;
        while record_number != 0 {
            visited += 1;
            if visited > 100_000 {
                return Err(SpkError::Format("summary record chain does not end".into()));
            }
            let offset = (record_number - 1)
                .checked_mul(RECORD_BYTES)
                .ok_or_else(|| SpkError::Format("summary record out of range".into()))?;
            let mut record = [0u8; RECORD_BYTES];
            spk.storage.read_bytes(offset, &mut record)?;
            let next = spk.double_at(&record, 0);
            if !(next >= 0.0 && next.fract() == 0.0 && next <= i32::MAX as f64) {
                return Err(SpkError::Format("bad next summary record".into()));
            }
            let count = spk.double_at(&record, 16) as usize;
            // The name record follows its summary record.
            let mut names = [0u8; RECORD_BYTES];
            spk.storage.read_bytes(offset + RECORD_BYTES, &mut names)?;
            let name_chars = summary_words * WORD_BYTES;
            for i in 0..count {
                let base = 24 + i * summary_words * WORD_BYTES;
                if base + summary_words * WORD_BYTES > RECORD_BYTES {
                    return Err(SpkError::Format("summary overflows its record".into()));
                }
                let mut segment = spk.read_segment(&record, base)?;
                let raw = &names[i * name_chars..(i + 1) * name_chars];
                segment.name = String::from_utf8_lossy(raw).trim_end().to_string();
                spk.segments.push(segment);
            }
            record_number = next as usize;
        }
        Ok(spk)
    }

    fn read_segment(&self, record: &[u8], base: usize) -> Result<Segment, SpkError> {
        let start_et = self.double_at(record, base);
        let end_et = self.double_at(record, base + 8);
        let ints = base + 16;
        let target = self.int_at(record, ints);
        let center = self.int_at(record, ints + 4);
        let frame = self.int_at(record, ints + 8);
        let data_type = self.int_at(record, ints + 12);
        let start_word = self.address_at(record, ints + 16)?;
        let end_word = self.address_at(record, ints + 20)?;

        let mut segment = Segment {
            name: String::new(),
            center,
            target,
            frame,
            data_type,
            start_et,
            end_et,
            start_word,
            init: 0.0,
            interval: 0.0,
            record_size: 0,
            record_count: 0,
        };
        if data_type == 2 || data_type == 3 {
            let bad =
                || SpkError::Format(format!("bad Chebyshev directory for {center} -> {target}"));
            if end_word < start_word.max(4) || end_word > self.words {
                return Err(SpkError::Format(format!(
                    "bad word range for {center} -> {target}"
                )));
            }
            let dir = self.read_words(end_word - 3, 4)?;
            let count = |x: f64| {
                (x >= 1.0 && x.fract() == 0.0 && x <= self.words as f64)
                    .then_some(x as usize)
                    .ok_or_else(bad)
            };
            segment.init = dir[0];
            segment.interval = dir[1];
            segment.record_size = count(dir[2])?;
            segment.record_count = count(dir[3])?;
            // The records and the directory must fit in the segment's words.
            let data_words = segment
                .record_size
                .checked_mul(segment.record_count)
                .and_then(|w| w.checked_add(4))
                .ok_or_else(bad)?;
            let coefficients = segment.record_size.saturating_sub(2);
            if !segment.init.is_finite()
                || !(segment.interval > 0.0 && segment.interval.is_finite())
                || data_words > end_word - start_word + 1
                || coefficients == 0
                || coefficients % segment.components() != 0
            {
                return Err(SpkError::Format(format!(
                    "bad Chebyshev directory for {center} -> {target}"
                )));
            }
        }
        Ok(segment)
    }

    /// State of `target` relative to `center` from the single segment that covers `et`.
    pub fn state(&self, target: i32, center: i32, et: f64) -> Result<State, SpkError> {
        let mut found_pair = false;
        for segment in &self.segments {
            if segment.target != target || segment.center != center {
                continue;
            }
            found_pair = true;
            if segment.covers(et) {
                return self.segment_state(segment, et);
            }
        }
        if found_pair {
            Err(SpkError::OutOfRange { target, center, et })
        } else {
            Err(SpkError::NoSegment { target, center })
        }
    }

    /// Evaluate one segment at `et` (which must lie inside it).
    pub fn segment_state(&self, segment: &Segment, et: f64) -> Result<State, SpkError> {
        if !segment.covers(et) {
            return Err(SpkError::OutOfRange {
                target: segment.target,
                center: segment.center,
                et,
            });
        }
        self.check_type(segment)?;
        let index = (((et - segment.init) / segment.interval).floor().max(0.0) as usize)
            .min(segment.record_count - 1);
        self.with_record(segment, index, |words| {
            let (mid, radius) = (words[0], words[1]);
            self.evaluate(segment, words, (et - mid) / radius)
        })
    }

    /// Evaluate one segment at a TDB Julian date given as `whole + fraction`, with the
    /// same interval arithmetic as jplephem (which Skyfield uses), keeping the precision
    /// of the split date.
    pub fn segment_state_split(
        &self,
        segment: &Segment,
        whole: f64,
        fraction: f64,
    ) -> Result<State, SpkError> {
        self.check_type(segment)?;
        let et = (whole - J2000_JD + fraction) * SECONDS_PER_DAY;
        let out_of_range = SpkError::OutOfRange {
            target: segment.target,
            center: segment.center,
            et,
        };
        let intlen = segment.interval;
        let a = (whole - J2000_JD) * SECONDS_PER_DAY - segment.init;
        let (index1, offset1) = (a.div_euclid(intlen), a.rem_euclid(intlen));
        let b = fraction * SECONDS_PER_DAY;
        let (index2, offset2) = (b.div_euclid(intlen), b.rem_euclid(intlen));
        let c = offset1 + offset2;
        let (index3, mut offset) = (c.div_euclid(intlen), c.rem_euclid(intlen));
        let mut index = index1 + index2 + index3;
        let count = segment.record_count as f64;
        if index == count {
            index -= 1.0;
            offset += intlen;
        }
        if index < 0.0 || index >= count {
            return Err(out_of_range);
        }
        self.with_record(segment, index as usize, |words| {
            self.evaluate(segment, words, 2.0 * offset / intlen - 1.0)
        })
    }

    fn check_type(&self, segment: &Segment) -> Result<(), SpkError> {
        if segment.data_type == 2 || segment.data_type == 3 {
            Ok(())
        } else {
            Err(SpkError::UnsupportedType(segment.data_type))
        }
    }

    /// `f` applied to the words of record `index` of `segment`, decoded into a stack
    /// buffer (records of the planetary ephemerides are a few dozen words).
    fn with_record<R>(
        &self,
        segment: &Segment,
        index: usize,
        f: impl FnOnce(&[f64]) -> R,
    ) -> Result<R, SpkError> {
        const STACK_WORDS: usize = 128;
        let (first, count) = (
            segment.start_word + index * segment.record_size,
            segment.record_size,
        );
        if count > STACK_WORDS {
            return Ok(f(&self.read_words(first, count)?));
        }
        self.check_words(first, count)?;
        let mut bytes = [0u8; STACK_WORDS * WORD_BYTES];
        let bytes = &mut bytes[..count * WORD_BYTES];
        self.storage.read_bytes((first - 1) * WORD_BYTES, bytes)?;
        let mut words = [0.0; STACK_WORDS];
        for (word, raw) in words.iter_mut().zip(bytes.chunks_exact(WORD_BYTES)) {
            *word = self.to_f64(raw.try_into().unwrap());
        }
        Ok(f(&words[..count]))
    }

    /// Position and velocity from one Chebyshev record at normalized time `s`.
    fn evaluate(&self, segment: &Segment, words: &[f64], s: f64) -> State {
        let radius = words[1];
        let n = (segment.record_size - 2) / segment.components();
        let coeffs = &words[2..];
        let mut position = [0.0; 3];
        let mut velocity = [0.0; 3];
        for axis in 0..3 {
            let c = &coeffs[axis * n..(axis + 1) * n];
            let (value, derivative) = chebyshev(c, s);
            position[axis] = value;
            velocity[axis] = derivative / radius;
        }
        if segment.data_type == 3 {
            // Type 3 stores velocity as its own series.
            for (axis, v) in velocity.iter_mut().enumerate() {
                let c = &coeffs[(axis + 3) * n..(axis + 4) * n];
                *v = chebyshev(c, s).0;
            }
        }
        (position, velocity)
    }

    /// A new SPK file with only the data covering TDB Julian dates `start_jd` to
    /// `end_jd` (like `python -m jplephem excerpt`): whole Chebyshev records are kept, so
    /// positions inside the range are identical to this kernel's. Segments that do not
    /// overlap the range are left out. Written little-endian; types 2 and 3 only.
    pub fn excerpt(&self, start_jd: f64, end_jd: f64) -> Result<Vec<u8>, SpkError> {
        const SUMMARIES_PER_RECORD: usize = 25; // (128 words - 3) / 5 words per summary
        const WORDS_PER_RECORD: usize = RECORD_BYTES / WORD_BYTES;
        if end_jd <= start_jd {
            return Err(SpkError::Format("excerpt range is empty".into()));
        }
        let (start_et, end_et) = (jd_tdb_to_et(start_jd), jd_tdb_to_et(end_jd));

        struct Kept<'a> {
            segment: &'a Segment,
            start_et: f64,
            end_et: f64,
            first_word: usize,
            last_word: usize,
        }
        let overlapping: Vec<&Segment> = self
            .segments
            .iter()
            .filter(|s| s.start_et <= end_et && s.end_et >= start_et)
            .collect();
        let summary_records = overlapping.len().div_ceil(SUMMARIES_PER_RECORD).max(1);
        // Record 1 is the file record, then summary + name record pairs, then data.
        let mut word = (1 + 2 * summary_records) * WORDS_PER_RECORD + 1;
        let mut data: Vec<f64> = Vec::new();
        let mut kept = Vec::new();
        for segment in overlapping {
            self.check_type(segment)?;
            let last = segment.record_count as i64 - 1;
            let first_index =
                (((start_et - segment.init) / segment.interval).floor() as i64).clamp(0, last);
            let last_index =
                (((end_et - segment.init) / segment.interval).floor() as i64).clamp(0, last);
            let (i0, i1) = (first_index as usize, last_index as usize);
            let count = i1 - i0 + 1;
            let records = self.read_words(
                segment.start_word + i0 * segment.record_size,
                count * segment.record_size,
            )?;
            let init = segment.init + i0 as f64 * segment.interval;
            let first_word = word;
            data.extend_from_slice(&records);
            data.extend_from_slice(&[
                init,
                segment.interval,
                segment.record_size as f64,
                count as f64,
            ]);
            word += records.len() + 4;
            kept.push(Kept {
                segment,
                start_et: segment.start_et.max(init),
                end_et: segment.end_et.min(init + count as f64 * segment.interval),
                first_word,
                last_word: word - 1,
            });
        }

        let data_records = data.len().div_ceil(WORDS_PER_RECORD);
        let mut out = vec![0u8; (1 + 2 * summary_records + data_records) * RECORD_BYTES];
        let put_i32 =
            |out: &mut [u8], at: usize, v: i32| out[at..at + 4].copy_from_slice(&v.to_le_bytes());
        let put_f64 =
            |out: &mut [u8], at: usize, v: f64| out[at..at + 8].copy_from_slice(&v.to_le_bytes());

        // File record.
        out[0..8].copy_from_slice(b"DAF/SPK ");
        put_i32(&mut out, 8, 2);
        put_i32(&mut out, 12, 6);
        let ifname = format!("{:<60}", "khagole-engine excerpt");
        out[16..76].copy_from_slice(&ifname.as_bytes()[..60]);
        put_i32(&mut out, 76, 2);
        put_i32(&mut out, 80, (2 + 2 * (summary_records - 1)) as i32);
        put_i32(&mut out, 84, word as i32);
        out[88..96].copy_from_slice(b"LTL-IEEE");
        out[699..727].copy_from_slice(b"FTPSTR:\r:\n:\r\n:\r\x00:\x81:\x10\xce:ENDFTP");

        // Summary and name records.
        let chunks: Vec<&[Kept]> = if kept.is_empty() {
            vec![&[]]
        } else {
            kept.chunks(SUMMARIES_PER_RECORD).collect()
        };
        for (k, chunk) in chunks.iter().enumerate() {
            let record = 2 + 2 * k;
            let base = (record - 1) * RECORD_BYTES;
            let next = if k + 1 < chunks.len() { record + 2 } else { 0 };
            let prev = if k > 0 { record - 2 } else { 0 };
            put_f64(&mut out, base, next as f64);
            put_f64(&mut out, base + 8, prev as f64);
            put_f64(&mut out, base + 16, chunk.len() as f64);
            let names = base + RECORD_BYTES;
            out[names..names + RECORD_BYTES].fill(b' ');
            for (i, kept) in chunk.iter().enumerate() {
                let at = base + 24 + i * 40;
                put_f64(&mut out, at, kept.start_et);
                put_f64(&mut out, at + 8, kept.end_et);
                let s = kept.segment;
                for (j, v) in [
                    s.target,
                    s.center,
                    s.frame,
                    s.data_type,
                    kept.first_word as i32,
                    kept.last_word as i32,
                ]
                .into_iter()
                .enumerate()
                {
                    put_i32(&mut out, at + 16 + 4 * j, v);
                }
                let name = s.name.as_bytes();
                let n = name.len().min(40);
                out[names + i * 40..names + i * 40 + n].copy_from_slice(&name[..n]);
            }
        }

        // Segment data.
        let data_base = (1 + 2 * summary_records) * RECORD_BYTES;
        for (i, v) in data.iter().enumerate() {
            put_f64(&mut out, data_base + i * WORD_BYTES, *v);
        }
        Ok(out)
    }

    /// Words `first_word..first_word + count` (1-based) must lie inside the file.
    fn check_words(&self, first_word: usize, count: usize) -> Result<(), SpkError> {
        if first_word == 0 {
            return Err(SpkError::Format("word address 0".into()));
        }
        if first_word
            .checked_add(count)
            .is_none_or(|end| end - 1 > self.words)
        {
            return Err(SpkError::Format("read past end of file".into()));
        }
        Ok(())
    }

    fn read_words(&self, first_word: usize, count: usize) -> Result<Vec<f64>, SpkError> {
        self.check_words(first_word, count)?;
        let mut buf = vec![0u8; count * WORD_BYTES];
        self.storage
            .read_bytes((first_word - 1) * WORD_BYTES, &mut buf)?;
        Ok(buf
            .chunks_exact(WORD_BYTES)
            .map(|b| self.to_f64(b.try_into().unwrap()))
            .collect())
    }

    /// A DAF record or word number (1-based; 0 ends a record chain).
    fn address_at(&self, bytes: &[u8], offset: usize) -> Result<usize, SpkError> {
        let n = self.int_at(bytes, offset);
        usize::try_from(n).map_err(|_| SpkError::Format(format!("negative address {n}")))
    }

    fn double_at(&self, bytes: &[u8], offset: usize) -> f64 {
        self.to_f64(bytes[offset..offset + 8].try_into().unwrap())
    }

    fn int_at(&self, bytes: &[u8], offset: usize) -> i32 {
        let raw: [u8; 4] = bytes[offset..offset + 4].try_into().unwrap();
        if self.little_endian {
            i32::from_le_bytes(raw)
        } else {
            i32::from_be_bytes(raw)
        }
    }

    fn to_f64(&self, raw: [u8; 8]) -> f64 {
        if self.little_endian {
            f64::from_le_bytes(raw)
        } else {
            f64::from_be_bytes(raw)
        }
    }
}

/// Chebyshev series value and derivative with respect to `s` at `s` in [-1, 1].
fn chebyshev(coeffs: &[f64], s: f64) -> (f64, f64) {
    let (mut t_prev, mut t) = (1.0, s);
    let (mut d_prev, mut d) = (0.0, 1.0);
    let mut value = coeffs[0];
    let mut derivative = 0.0;
    if coeffs.len() > 1 {
        value += coeffs[1] * s;
        derivative += coeffs[1];
    }
    for &c in &coeffs[2.min(coeffs.len())..] {
        let t_next = 2.0 * s * t - t_prev;
        let d_next = 2.0 * t + 2.0 * s * d - d_prev;
        value += c * t_next;
        derivative += c * d_next;
        (t_prev, t) = (t, t_next);
        (d_prev, d) = (d, d_next);
    }
    (value, derivative)
}

#[cfg(test)]
mod tests {
    use super::chebyshev;

    #[test]
    fn chebyshev_matches_closed_form() {
        // 1 + 2 T1 + 3 T2 + 4 T3 with T2 = 2s²-1, T3 = 4s³-3s.
        let s: f64 = 0.37;
        let expected =
            1.0 + 2.0 * s + 3.0 * (2.0 * s * s - 1.0) + 4.0 * (4.0 * s.powi(3) - 3.0 * s);
        let expected_d = 2.0 + 12.0 * s + 4.0 * (12.0 * s * s - 3.0);
        let (v, d) = chebyshev(&[1.0, 2.0, 3.0, 4.0], s);
        assert!((v - expected).abs() < 1e-12);
        assert!((d - expected_d).abs() < 1e-12);
    }
}
