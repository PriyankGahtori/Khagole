# Essential dignities: tables and rules

`crates/khagole-engine/src/dignities.rs` gives each of the seven planets a `condition`
and each chart its `sect`, `receptions` and `antiscia`. These are additions to the
reference implementation, so the golden fixtures do not cover them; `tests/dignities.rs` and
the module's unit tests do. All tables are public domain.

## Two schemes

A request's `dignity_scheme` chooses the triplicities and bounds:

| | `lilly` (default) | `dorothean` |
|---|---|---|
| Triplicity lords | by day and by night; Mars rules water both | by day, by night and participating |
| A planet has triplicity when it is | the lord of the chart's sect | the lord of the sect, or the participating lord |
| Bounds | Ptolemaic terms as Lilly printed them | Egyptian bounds |
| Faces | Chaldean order from Mars at 0° Aries | the same |

Domicile and exaltation are the same in both: the seven-planet rulerships and Ptolemy's
exaltations (Sun 19° Aries, Moon 3° Taurus, Jupiter 15° Cancer, Mercury 15° Virgo, Saturn
21° Libra, Mars 28° Capricorn, Venus 27° Pisces). Detriment and fall are the opposite signs.

## Sources

- **Triplicities**: Dorotheus, *Carmen Astrologicum* I.1; Lilly, *Christian Astrology*
  (London, 1647), "A Table of the Essential Dignities of the Planets", p. 104.
- **Egyptian bounds**: Ptolemy, *Tetrabiblos* I.20, which reports them. Each planet's bounds
  add up to its minor years (Saturn 57, Jupiter 79, Mars 66, Venus 82, Mercury 76), which a
  unit test checks.
- **Lilly's terms**: the same table of Lilly, p. 104. Lilly's Ptolemaic terms differ from
  the critical Greek editions of the *Tetrabiblos*, notably in Gemini, where Lilly gives
  Saturn the 4th term and Mars the 5th (D. Houlding, "The Transmission of Ptolemy's Terms",
  *Culture and Cosmos* 11, 2007). The values were checked against the independent
  transcription in flatlib (`flatlib/dignities/tables.py`, `LILLY_TERMS`); in Pisces, Mars's
  term ends at 25°, as in both of flatlib's Ptolemaic tables. They have not yet been checked
  against a scan of the 1647 page.

## Points

Lilly's scores (*Christian Astrology*, the table of fortitudes and debilities): domicile +5,
exaltation +4, triplicity +3, term +2, face +1; detriment −5, fall −4. A planet with none of the five dignities is
peregrine, −5. Mutual reception and the accidental dignities are not scored.

## The rest of the condition

- **Sect**: the chart is diurnal when the Sun is above the horizon, measured on the ecliptic
  from the Ascendant. The Sun, Jupiter and Saturn are diurnal; the Moon, Venus and Mars
  nocturnal; Mercury is diurnal when oriental and nocturnal when occidental. Hayz (sect,
  hemisphere and the gender of the sign together) is not given: the authors disagree on it.
- **Phase to the Sun**: cazimi within 17′, combust within 8°30′, under the beams within 15°.
- **Orientality** (not for the luminaries): oriental when the planet rises before the Sun,
  that is, when it is behind the Sun in the zodiac by less than 180°.
- **Motion**: against Lilly's mean daily motions (Saturn 2′01″, Jupiter 4′59″, Mars 31′27″,
  the Sun, Venus and Mercury 59′08″, the Moon 13°10′36″): `fast` at or above the mean,
  `slow` below it, and `stationary` (not for the luminaries) below a tenth of it.
  Retrogradation is the placement's `is_retrograde`.

## Receptions and antiscia

A planet is received by the lord of any of the five dignities where it stands. The chart
lists the receptions between planets joined by one of its aspects, and the mutual
receptions by domicile or exaltation even without an aspect. A reception is mutual when
each planet stands in the other's domicile or exaltation.

The antiscion of a longitude λ mirrors it across the solstitial axis (180° − λ); the
contra-antiscion mirrors it across the equinoctial axis (360° − λ). Contacts are listed
among the seven planets, the Ascendant and the Midheaven, within 1°.

## Judging again

`chart_dignities` recomputes the condition, the sect and the receptions of an existing chart
JSON under either scheme, from its planets' longitudes and speeds and its aspects. It gives the
same result as computing the chart with that scheme (`tests/dignities.rs` checks it), so an
application can store charts with one scheme and let each reader choose theirs.

