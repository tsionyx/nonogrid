use std::cmp::Reverse;

pub fn pad(s: &mut String, max_size: usize, right: bool) {
    let s_len = s.chars().count();
    if max_size > s_len {
        let spaces = " ".repeat(max_size - s_len);
        if right {
            s.push_str(spaces.as_str())
        } else {
            s.insert_str(0, spaces.as_str())
        }
    }
}

pub fn pad_with<T: Clone>(v: &mut Vec<T>, el: T, max_size: usize, right: bool) {
    let v_len = v.len();
    if max_size > v_len {
        let plus = vec![el; max_size - v_len];

        if right {
            v.extend(plus);
        } else {
            v.splice(..0, plus);
        }
    }
}

pub fn transpose<T: Clone>(input: &[Vec<T>]) -> Result<Vec<Vec<T>>, String> {
    if input.is_empty() {
        return Ok(vec![]);
    }

    let sizes: Vec<usize> = input.iter().map(|row| row.len()).collect();
    let min_size = sizes.iter().min().unwrap_or(&0);
    let max_size = sizes.iter().max().unwrap_or(&0);

    if min_size != max_size {
        return Err(format!("Jagged matrix: {} vs {}", min_size, max_size));
    }

    Ok((0..input[0].len())
        .map(|j| input.iter().map(|row| row[j].clone()).collect())
        .collect())
}

pub fn replace<T>(vec: &mut Vec<T>, what: T, with_what: T)
where
    T: PartialEq + Clone,
{
    if what == with_what {
        return;
    }

    if !vec.contains(&what) {
        return;
    }

    let replaced_indexes: Vec<usize> = vec
        .iter()
        .enumerate()
        .filter_map(|(index, val)| if val == &what { Some(index) } else { None })
        .collect();

    vec.extend(vec![with_what; replaced_indexes.len()]);
    for index in replaced_indexes {
        vec.swap_remove(index);
    }
}

pub fn remove<T>(vec: &mut Vec<T>, what: T)
where
    T: PartialEq,
{
    if !vec.contains(&what) {
        return;
    }

    let mut removed_indexes: Vec<usize> = vec
        .iter()
        .enumerate()
        .filter_map(|(index, val)| if val == &what { Some(index) } else { None })
        .collect();

    removed_indexes.sort_by_key(|&n| Reverse(n));
    for index in removed_indexes {
        vec.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::{pad, pad_with, remove, replace, transpose};

    #[test]
    fn pad_vector_left() {
        let mut v = vec![1, 2];
        pad_with(&mut v, 0, 4, false);
        assert_eq!(v, [0, 0, 1, 2])
    }

    #[test]
    fn pad_vector_right() {
        let mut v = vec![1, 2];
        pad_with(&mut v, 0, 4, true);
        assert_eq!(v, [1, 2, 0, 0])
    }

    #[test]
    fn do_not_pad_vector_right() {
        let mut v = vec![1, 2, 3];
        pad_with(&mut v, 0, 2, true);
        assert_eq!(v, [1, 2, 3])
    }

    #[test]
    fn do_not_pad_vector_left() {
        let mut v = vec![1, 2, 3];
        pad_with(&mut v, 0, 2, false);
        assert_eq!(v, [1, 2, 3])
    }

    #[test]
    fn pad_string_left() {
        let mut s = "hello".to_string();
        pad(&mut s, 7, false);
        assert_eq!(s, "  hello")
    }

    #[test]
    fn pad_string_right() {
        let mut s = "world".to_string();
        pad(&mut s, 7, true);
        assert_eq!(s, "world  ")
    }

    #[test]
    fn do_not_pad_string_left() {
        let mut s = "hello".to_string();
        pad(&mut s, 4, false);
        assert_eq!(s, "hello")
    }

    #[test]
    fn do_not_pad_string_right() {
        let mut s = "world".to_string();
        pad(&mut s, 5, true);
        assert_eq!(s, "world")
    }

    #[test]
    fn pad_non_ascii_right() {
        let mut s = "Привет".to_string();
        pad(&mut s, 8, true);
        assert_eq!(s, "Привет  ")
    }

    #[test]
    fn transpose_empty() {
        let m: Vec<Vec<u8>> = vec![];
        assert_eq!(transpose(&m).unwrap(), Vec::<Vec<u8>>::new())
    }

    #[test]
    fn transpose_empty_rows() {
        let m: Vec<Vec<u8>> = vec![vec![], vec![], vec![]];
        assert_eq!(transpose(&m).unwrap(), Vec::<Vec<u8>>::new())
    }

    #[test]
    fn transpose_square() {
        let m = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        assert_eq!(
            transpose(&m).unwrap(),
            vec![vec![1, 4, 7], vec![2, 5, 8], vec![3, 6, 9]]
        )
    }

    #[test]
    fn transpose_jagged() {
        let m = vec![vec![1, 2, 3], vec![4, 5], vec![7, 8, 9]];
        assert_eq!(transpose(&m).unwrap_err(), "Jagged matrix: 2 vs 3")
    }

    #[test]
    fn replace_ints() {
        let mut v = vec![1, 2, 3, 2];
        replace(&mut v, 2, 5);

        assert_eq!(v, vec![1, 5, 3, 5]);
    }

    #[test]
    fn no_replacement() {
        let mut v = vec![1, 2, 3, 2];
        replace(&mut v, 5, 4);

        assert_eq!(v, vec![1, 2, 3, 2]);
    }

    #[test]
    fn remove_with_replace() {
        let mut v = vec![1, 2, 3, 2];
        remove(&mut v, 2);

        assert_eq!(v, vec![1, 3]);
    }
}
