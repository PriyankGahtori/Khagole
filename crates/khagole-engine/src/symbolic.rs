//! Symbolic (1-30) degree numbering used by degree-symbolism systems such as Sabian.

/// `get_symbolic_degree_number`: a position at degree `d`, minute `m` of a sign falls
/// in symbolic degree `d + 1` once any minute has passed.
pub fn symbolic_degree_number(degree: i64, minute: i64) -> i64 {
    if degree == 0 && minute == 0 {
        return 1;
    }
    if minute > 0 {
        return (degree + 1).min(30);
    }
    degree.clamp(1, 30)
}

#[cfg(test)]
mod tests {
    use super::symbolic_degree_number;

    #[test]
    fn numbering() {
        assert_eq!(symbolic_degree_number(0, 0), 1);
        assert_eq!(symbolic_degree_number(0, 30), 1);
        assert_eq!(symbolic_degree_number(15, 40), 16);
        assert_eq!(symbolic_degree_number(15, 0), 15);
        assert_eq!(symbolic_degree_number(29, 1), 30);
    }
}
