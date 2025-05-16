/// Adds two integer numbers together and returns their sum.
///
/// This function takes two i32 integers and returns their mathematical sum.
/// It handles both positive and negative numbers.
///
/// # Arguments
///
/// * `a` - The first integer to add
/// * `b` - The second integer to add
///
/// # Returns
///
/// The sum of `a` and `b` as an i32
///
/// # Examples
///
/// ```
/// use agentic_protos::tdd_sample::add_two;
///
/// assert_eq!(add_two(2, 2), 4);
/// assert_eq!(add_two(-1, 5), 4);
/// assert_eq!(add_two(0, 0), 0);
/// ```
pub fn add_two(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_adds_two_numbers() {
        let sum = add_two(2, 2);
        assert_eq!(sum, 4);
    }

    #[test]
    fn it_handles_negative_numbers() {
        assert_eq!(add_two(-3, 5), 2);
        assert_eq!(add_two(10, -5), 5);
        assert_eq!(add_two(-7, -3), -10);
    }

    #[test]
    fn it_handles_zero() {
        assert_eq!(add_two(0, 5), 5);
        assert_eq!(add_two(10, 0), 10);
        assert_eq!(add_two(0, 0), 0);
    }

    #[test]
    fn it_handles_large_numbers() {
        assert_eq!(add_two(10000, 20000), 30000);
        // Test for numbers close to i32 limits, but avoid overflow
        assert_eq!(add_two(i32::MAX - 10, 5), i32::MAX - 5);
        assert_eq!(add_two(i32::MIN + 10, -5), i32::MIN + 5);
    }
}