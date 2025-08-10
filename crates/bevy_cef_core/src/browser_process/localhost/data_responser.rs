use bevy::prelude::*;

#[derive(Default, Clone)]
pub struct DataResponser {
    data: Vec<u8>,
    offset: usize,
    end_offset: usize,
}

impl DataResponser {
    /// Prepares the data and headers for the response.
    ///
    /// The range header values only support the `bytes` range unit type and single range.
    /// TODO: Support multiple ranges.
    pub fn prepare(&mut self, data: Vec<u8>, range: &Option<(usize, Option<usize>)>) {
        if let Some((start, end)) = range {
            self.offset = *start;
            self.end_offset = end.unwrap_or(data.len()-1);
            self.data = data;
        } else {
            self.offset = 0;
            self.end_offset = data.len();
            self.data = data;
        }
    }

    pub fn read(&mut self, bytes_to_read: isize) -> Option<&[u8]> {
        if self.offset >= self.data.len() {
            return None;
        }
        let start = self.offset;
        let end = if bytes_to_read < 0 {
            self.data.len()
        } else {
            (self.offset as isize + bytes_to_read) as usize
        };
        let end = end.min(self.end_offset);

        if start >= end || start >= self.data.len() {
            return None;
        }

        let slice = &self.data[start..=end.min(self.data.len()-1)];
        self.offset += slice.len();
        Some(slice)
    }
}

pub fn parse_bytes_single_range(range_header_value: &str) -> Option<(usize, Option<usize>)> {
    let ranges = parse_bytes_range(range_header_value)?;
    ranges.first().cloned()
}

/// Parses the `Range` header value from a request and returns the start of the range.
///
/// ## Reference
///
/// - [`Range_requests`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Range_requests)
fn parse_bytes_range(range_header_value: &str) -> Option<Vec<(usize, Option<usize>)>> {
    if !range_header_value.starts_with("bytes=") {
        return None;
    }
    let mut ranges = Vec::new();
    let value = range_header_value.trim_start_matches("bytes=");
    // bytes=100-200,300-400 => ["100-200", "300-400"]
    let byte_ranges = value.split(",");
    for range in byte_ranges {
        // 100-200 => ["100", "200"]
        let mut split = range.split("-");
        let start = split.next()?;
        let end = split.next();
        let start = start.parse::<usize>().ok()?;
        let end = end.and_then(|e| e.parse::<usize>().ok());
        ranges.push((start, end));
    }
    Some(ranges)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn range_start_is_none_if_empty() {
        assert_eq!(parse_bytes_range(""), None);
    }

    #[test]
    fn range_only_start_offset() {
        assert_eq!(parse_bytes_range("bytes=100-"), Some(vec![(100, None)]));
    }

    #[test]
    fn range_one_bytes() {
        assert_eq!(
            parse_bytes_range("bytes=100-200"),
            Some(vec![(100, Some(200))])
        );
    }

    #[test]
    fn range_multiple_ranges() {
        assert_eq!(
            parse_bytes_range("bytes=100-200,300-400"),
            Some(vec![(100, Some(200)), (300, Some(400))])
        );
    }

    #[test]
    fn data_responser_new_with_start_and_end() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut responser = DataResponser::default();
        responser.prepare(data.clone(), &Some((2, Some(7))));
        assert_eq!(responser.data, data);
        assert_eq!(responser.offset, 2);
        assert_eq!(responser.end_offset, 7);
    }
}