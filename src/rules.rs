pub fn wrap_decr(value: usize, max: usize) -> usize {
    (value as isize - 1).rem_euclid(max as isize) as usize
}

pub fn wrap_incr(value: usize, max: usize) -> usize {
    (value + 1) % max
}
