use crate::{mark_initialized, uninit_buf};

/// Build an array with a function that creates elements based on their index
///
/// ```
/// # use unarray::*;
/// let array: [usize; 5] = build_array(|i| i * 2);
/// assert_eq!(array, [0, 2, 4, 6, 8]);
/// ```
///
/// For builder functions which might fail, consider using [`build_array_result`] or
/// [`build_array_option`]
pub fn build_array<T, const N: usize, F: FnMut(usize) -> T>(mut f: F) -> [T; N] {
    let mut result = uninit_buf();

    for (index, slot) in result.iter_mut().enumerate() {
        let value = f(index);
        slot.write(value);
    }

    // SAFETY:
    // We have iterated over every element in result and called `.write()` on it, so every element
    // is initialized
    unsafe { mark_initialized(result) }
}

/// Build an array with a function that creates elements based on their value, short-circuiting if
/// any index returns an `Err`
///
/// ```
/// # use unarray::*;
///
/// let success: Result<_, ()> = build_array_result(|i| Ok(i * 2));
/// assert_eq!(success, Ok([0, 2, 4]));
/// ```
pub fn build_array_result<T, E, const N: usize, F: FnMut(usize) -> Result<T, E>>(
    mut f: F,
) -> Result<[T; N], E> {
    let mut result = uninit_buf();

    for (index, slot) in result.iter_mut().enumerate() {
        match f(index) {
            Ok(value) => slot.write(value),
            Err(e) => return Err(e),
        };
    }

    // SAFETY:
    // We have iterated over every element in result and called `.write()` on it, so every element
    // is initialized
    Ok(unsafe { mark_initialized(result) })
}

/// Build an array with a function that creates elements based on their value, short-circuiting if
/// any index returns a `None`
///
/// ```
/// # use unarray::*;
/// let success = build_array_option(|i| Some(i * 2));
/// assert_eq!(success, Some([0, 2, 4]));
/// ```
pub fn build_array_option<T, const N: usize, F: FnMut(usize) -> Option<T>>(
    mut f: F,
) -> Option<[T; N]> {
    let actual_f = |i: usize| -> Result<T, ()> { f(i).ok_or(()) };

    match build_array_result(actual_f) {
        Ok(array) => Some(array),
        Err(()) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_array() {
        let array = build_array(|i| i * 2);
        assert_eq!(array, [0, 2, 4]);
    }

    #[test]
    fn test_build_array_option() {
        let array = build_array_option(|i| Some(i * 2));
        assert_eq!(array, Some([0, 2, 4]));
    }

    #[test]
    fn test_build_array_result() {
        let array = build_array_result(|i| Ok::<usize, ()>(i * 2));
        assert_eq!(array, Ok([0, 2, 4]));
    }
}
