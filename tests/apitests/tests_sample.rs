use boson::sample::add;
use boson::sample::sub;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1,2), 3);
    }

    #[test]
    fn test_sub() {
        assert_eq!(sub(2,1), 1);
    }
}