// Transcribed from William Lilly, Christian Astrology (London, 1659), p. 116, "Two necessary
// Tables of the Signs": docs/degree-qualities.md records the source and the readings of the
// cells the scan leaves unclear. Degrees are ordinal: degree n is n-1°00' to n-1°59'.

use super::{DegreeRun, SignDegrees};

const M: &str = "masculine";
const F: &str = "feminine";
const L: &str = "light";
const D: &str = "dark";
const S: &str = "smoky";
const V: &str = "void";

const fn r(quality: &'static str, end: u8) -> DegreeRun {
    DegreeRun { quality, end }
}

pub const DEGREE_QUALITIES: [SignDegrees; 12] = [
    SignDegrees {
        sign: "Aries",
        gender: &[r(M, 8), r(F, 9), r(M, 15), r(F, 22), r(M, 30)],
        light: &[r(D, 3), r(L, 8), r(D, 16), r(L, 20), r(V, 24), r(L, 29), r(V, 30)],
        pitted: &[6, 11, 16, 23, 29],
        azimene: &[],
        fortune: &[19],
    },
    SignDegrees {
        sign: "Taurus",
        gender: &[r(F, 5), r(M, 11), r(F, 17), r(M, 21), r(F, 24), r(M, 30)],
        light: &[r(D, 3), r(L, 7), r(V, 12), r(L, 15), r(V, 20), r(L, 28), r(D, 30)],
        pitted: &[5, 12, 24, 25],
        azimene: &[6, 7, 8, 9, 10],
        fortune: &[3, 15, 27],
    },
    SignDegrees {
        sign: "Gemini",
        gender: &[r(F, 5), r(M, 16), r(F, 22), r(M, 26), r(F, 30)],
        light: &[r(L, 4), r(D, 7), r(L, 12), r(V, 16), r(L, 22), r(D, 27), r(V, 30)],
        pitted: &[2, 12, 17, 26, 30],
        azimene: &[],
        fortune: &[11],
    },
    SignDegrees {
        sign: "Cancer",
        gender: &[r(M, 2), r(F, 8), r(M, 10), r(F, 12), r(M, 23), r(F, 27), r(M, 30)],
        light: &[r(L, 12), r(D, 14), r(V, 18), r(S, 20), r(L, 28), r(V, 30)],
        pitted: &[12, 17, 23, 26, 30],
        azimene: &[9, 10, 11, 12, 13, 14, 15],
        fortune: &[1, 2, 3, 4, 15],
    },
    SignDegrees {
        sign: "Leo",
        gender: &[r(M, 5), r(F, 8), r(M, 15), r(F, 23), r(M, 30)],
        light: &[r(D, 10), r(S, 20), r(V, 25), r(L, 30)],
        pitted: &[6, 13, 15, 22, 23, 28],
        azimene: &[18, 27, 28],
        fortune: &[2, 5, 7, 19],
    },
    SignDegrees {
        sign: "Virgo",
        gender: &[r(F, 8), r(M, 12), r(F, 20), r(M, 30)],
        light: &[r(D, 5), r(L, 8), r(V, 10), r(L, 16), r(S, 22), r(V, 27), r(D, 30)],
        pitted: &[8, 13, 16, 21, 22],
        azimene: &[],
        fortune: &[3, 14, 20],
    },
    SignDegrees {
        sign: "Libra",
        gender: &[r(M, 5), r(F, 15), r(M, 20), r(F, 27), r(M, 30)],
        light: &[r(L, 5), r(D, 10), r(L, 18), r(D, 21), r(L, 27), r(V, 30)],
        pitted: &[1, 7, 20, 30],
        azimene: &[],
        fortune: &[3, 15, 21],
    },
    SignDegrees {
        sign: "Scorpio",
        gender: &[r(M, 4), r(F, 14), r(M, 17), r(F, 25), r(M, 30)],
        light: &[r(D, 3), r(L, 8), r(V, 14), r(L, 22), r(S, 24), r(V, 29), r(D, 30)],
        pitted: &[9, 10, 22, 23, 27],
        azimene: &[19, 28],
        fortune: &[7, 18, 20],
    },
    SignDegrees {
        sign: "Sagittarius",
        gender: &[r(M, 2), r(F, 5), r(M, 12), r(F, 24), r(M, 30)],
        light: &[r(L, 9), r(D, 12), r(L, 19), r(S, 23), r(L, 30)],
        pitted: &[7, 12, 15, 24, 27, 30],
        azimene: &[1, 7, 8, 18, 19],
        fortune: &[13, 20],
    },
    SignDegrees {
        sign: "Capricorn",
        gender: &[r(M, 11), r(F, 19), r(M, 30)],
        light: &[r(D, 7), r(L, 10), r(S, 15), r(L, 19), r(D, 22), r(V, 25), r(D, 30)],
        pitted: &[7, 17, 22, 24, 29],
        azimene: &[26, 27, 28, 29],
        fortune: &[12, 13, 14, 20],
    },
    SignDegrees {
        sign: "Aquarius",
        gender: &[r(M, 5), r(F, 15), r(M, 21), r(F, 25), r(M, 27), r(F, 30)],
        light: &[r(S, 4), r(L, 9), r(D, 13), r(L, 21), r(V, 25), r(L, 30)],
        pitted: &[1, 12, 17, 22, 24, 29],
        azimene: &[18, 19],
        fortune: &[7, 16, 17, 20],
    },
    SignDegrees {
        sign: "Pisces",
        gender: &[r(M, 10), r(F, 20), r(M, 23), r(F, 28), r(M, 30)],
        light: &[r(D, 6), r(L, 12), r(D, 18), r(L, 22), r(V, 25), r(L, 28), r(D, 30)],
        pitted: &[4, 9, 24, 27, 28],
        azimene: &[],
        fortune: &[13, 20],
    },
];
