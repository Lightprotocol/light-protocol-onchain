pub mod close_account;

const CHUNK_SIZE: usize = 32;

pub fn change_endianness<const SIZE: usize>(bytes: &[u8; SIZE]) -> [u8; SIZE] {
    let mut arr = [0u8; SIZE];
    for (i, b) in bytes.chunks(CHUNK_SIZE).enumerate() {
        for (j, byte) in b.iter().rev().enumerate() {
            arr[i * CHUNK_SIZE + j] = *byte;
        }
    }
    arr
}

/// Truncates the given 32-byte array, replacing the least important element
/// with 0, making it fit into Fr modulo field.
///
/// # Safety
///
/// This function is used mostly for truncating hashes (i.e. SHA-256) which are
/// not constrainted by any modulo space. At the same time, we can't (yet) use
/// any ZK-friendly function in one transaction. Truncating hashes to 31 should
/// be generally safe, but please make sure that it's appropriate in your case.
///
/// # Examples
///
/// ```
/// let original: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
///                            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
///                            29, 30, 31, 32];
/// let truncated: [u8; 32] = truncate_function(&original);
/// assert_eq!(truncated, [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
///                        18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
/// ```
pub fn truncate_to_circuit(bytes: &[u8; 32]) -> [u8; 32] {
    let mut truncated = [0; 32];
    truncated[1..].copy_from_slice(&bytes[1..]);
    truncated
}
