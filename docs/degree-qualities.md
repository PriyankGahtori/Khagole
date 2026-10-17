# Degree qualities: transcription notes

The table in `crates/khagole-engine/src/catalog/degree_qualities.rs` is transcribed from
William Lilly, *Christian Astrology*, second edition (London, 1659), p. 116: "Two necessary
Tables of the Signs, fit to be understood by every Astrologer, or Practitioner". The work is
in the public domain. The scan used is on Wikimedia Commons:
[Christian Astrology (Lilly, 1659).djvu](https://commons.wikimedia.org/wiki/File:Christian_Astrology_(Lilly,_1659).djvu),
page 141 of the file.

## How the table reads

- **Masculine and feminine** ("mas. 8.15.30. fem. 9.22." for Aries): the numbers are the last
  degree of each run. Merged in order they alternate: masculine to 8, feminine 9, masculine
  to 15, feminine to 22, masculine to 30.
- **Light, dark, smoky, void** ("d. 3. l. 8. d. 16. …"): the same, with `l.` light, `d.`
  dark, `sm.` (once `s.`, Capricorn) smoky and `v.` void.
- **Deep or pitted**, **lame or deficient** (azimene) and **increasing fortune** list single
  degrees.
- Degrees are ordinal: the 6th degree is 5°00′–5°59′ of the sign.

## Readings of unclear cells

The scan is worn in places. Where a cell is unclear, the reading below is the one forced by
the table's structure (runs increase and end at 30, and lists are in ascending order).

| Sign | Column | Printed | Read as |
|---|---|---|---|
| Libra | masculine | 5.20.50 | 5.20.30 (the run must end at 30) |
| Gemini | pitted | … 26 40 | … 26 30 |
| Cancer | pitted | … 26 20 | … 26 30 (after 26) |
| Libra | pitted | 20 ?0 | 20 30 |
| Aries | pitted | 2? 29 | 23 29 |
| Aquarius | feminine | 15.25.3? | 15.25.30 |
| Capricorn | light | f.15 | smoky to 15 (`s.`, as `sm.` elsewhere) |
| Capricorn | pitted | 7 1? 22 | 7 17 22 |
| Pisces | light | d.18 (smudged) | dark to 18 |
| Pisces | fortune | c3 20 | 13 20 |

Proofread any change against the scan.
