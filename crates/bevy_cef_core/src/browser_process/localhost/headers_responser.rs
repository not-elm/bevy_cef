use crate::prelude::CefResponse;

#[derive(Clone, Default, Debug)]
pub struct HeadersResponser {
    pub mime_type: String,
    pub status_code: u32,
    pub headers: Vec<(String, String)>,
    pub response_length: usize,
}

impl HeadersResponser {
    pub fn prepare(&mut self, cef_response: &CefResponse, range: &Option<(usize, Option<usize>)>) {
        self.mime_type = cef_response.mime_type.clone();
        self.status_code = if range.is_some() {
            206 // Partial Content
        } else {
            cef_response.status_code
        };
        self.headers.clear();
        self.response_length = obtain_response_length(&cef_response.data, range);
        self.headers
            .push(("Access-Control-Allow-Origin".to_string(), "*".to_string()));
        self.headers
            .push(("Access-Control-Allow-Methods".to_string(), "*".to_string()));
        self.headers
            .push(("Access-Control-Allow-Headers".to_string(), "*".to_string()));
        if let Some(content_range) = content_range_header_value(&cef_response.data, range) {
            self.headers
                .push(("Content-Range".to_string(), content_range));
            self.headers
                .push(("Accept-Ranges".to_string(), "bytes".to_string()));
        }
    }
}

/// Create a `Content-Range` header value based on the provided data and range.
///
/// If the range is `None`, since the request type is not a range request, it returns `None`.
///
/// ## Reference
///
/// - [206 Partial Content](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status/206)
fn content_range_header_value(
    data: &[u8],
    range: &Option<(usize, Option<usize>)>,
) -> Option<String> {
    let (start, end) = range.as_ref()?;
    Some(format!(
        "bytes {}-{}/{}",
        start,
        end.unwrap_or(data.len()),
        data.len()
    ))
}

fn obtain_response_length(data: &[u8], range: &Option<(usize, Option<usize>)>) -> usize {
    match range {
        Some((start, end)) => end.unwrap_or(data.len()) - start,
        None => data.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::utils::default;

    #[test]
    fn test_obtain_response_length_no_range() {
        let data = b"Hello, World!";
        let result = obtain_response_length(data, &None);
        assert_eq!(result, 13);
    }

    #[test]
    fn test_obtain_response_length_empty_data_no_range() {
        let data = b"";
        let result = obtain_response_length(data, &None);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_obtain_response_length_range_with_end() {
        let data = b"Hello, World!";
        let result = obtain_response_length(data, &Some((0, Some(5))));
        assert_eq!(result, 5);
    }

    #[test]
    fn test_obtain_response_length_range_partial() {
        let data = b"Hello, World!";
        let result = obtain_response_length(data, &Some((7, Some(12))));
        assert_eq!(result, 5);
    }

    #[test]
    fn test_obtain_response_length_range_without_end() {
        let data = b"Hello, World!";
        let result = obtain_response_length(data, &Some((7, None)));
        assert_eq!(result, 6);
    }

    #[test]
    fn test_obtain_response_length_range_from_start() {
        let data = b"Hello, World!";
        let result = obtain_response_length(data, &Some((0, None)));
        assert_eq!(result, 13);
    }

    #[test]
    fn test_obtain_response_length_range_zero_length() {
        let data = b"Hello, World!";
        let result = obtain_response_length(data, &Some((5, Some(5))));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_obtain_response_length_range_end_equals_data_len() {
        let data = b"Hello, World!";
        let result = obtain_response_length(data, &Some((0, Some(13))));
        assert_eq!(result, 13);
    }

    #[test]
    fn test_obtain_response_length_empty_data_with_range() {
        let data = b"";
        let result = obtain_response_length(data, &Some((0, None)));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_obtain_response_length_large_data() {
        let data = vec![0u8; 1024];
        let result = obtain_response_length(&data, &None);
        assert_eq!(result, 1024);
    }

    #[test]
    fn test_obtain_response_length_large_data_with_range() {
        let data = vec![0u8; 1024];
        let result = obtain_response_length(&data, &Some((100, Some(200))));
        assert_eq!(result, 100);
    }

    #[test]
    fn test_content_range_header_value_no_range() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_content_range_header_value_range_with_end() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &Some((0, Some(5))));
        assert_eq!(result, Some("bytes 0-5/13".to_string()));
    }

    #[test]
    fn test_content_range_header_value_range_without_end() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &Some((7, None)));
        assert_eq!(result, Some("bytes 7-13/13".to_string()));
    }

    #[test]
    fn test_content_range_header_value_range_from_start() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &Some((0, None)));
        assert_eq!(result, Some("bytes 0-13/13".to_string()));
    }

    #[test]
    fn test_content_range_header_value_range_partial() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &Some((7, Some(12))));
        assert_eq!(result, Some("bytes 7-12/13".to_string()));
    }

    #[test]
    fn test_content_range_header_value_range_single_byte() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &Some((5, Some(6))));
        assert_eq!(result, Some("bytes 5-6/13".to_string()));
    }

    #[test]
    fn test_content_range_header_value_range_last_byte() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &Some((12, Some(13))));
        assert_eq!(result, Some("bytes 12-13/13".to_string()));
    }

    #[test]
    fn test_content_range_header_value_single_byte_data() {
        let data = b"a";
        let result = content_range_header_value(data, &Some((0, None)));
        assert_eq!(result, Some("bytes 0-1/1".to_string()));
    }

    #[test]
    fn test_content_range_header_value_large_data() {
        let data = vec![0u8; 1024];
        let result = content_range_header_value(&data, &Some((100, Some(200))));
        assert_eq!(result, Some("bytes 100-200/1024".to_string()));
    }

    #[test]
    fn test_content_range_header_value_large_data_no_end() {
        let data = vec![0u8; 1024];
        let result = content_range_header_value(&data, &Some((500, None)));
        assert_eq!(result, Some("bytes 500-1024/1024".to_string()));
    }

    #[test]
    fn test_content_range_header_value_zero_start() {
        let data = b"test";
        let result = content_range_header_value(data, &Some((0, Some(2))));
        assert_eq!(result, Some("bytes 0-2/4".to_string()));
    }

    #[test]
    fn test_content_range_header_value_range_end_equals_data_len() {
        let data = b"Hello, World!";
        let result = content_range_header_value(data, &Some((0, Some(13))));
        assert_eq!(result, Some("bytes 0-13/13".to_string()));
    }

    #[test]
    fn status_code_is_206_for_partial_content() {
        let data = b"Hello, World!";
        let mut headers_responser = HeadersResponser::default();
        headers_responser.prepare(
            &CefResponse {
                data: data.to_vec(),
                ..default()
            },
            &Some((0, Some(5))),
        );
        assert_eq!(headers_responser.status_code, 206);
    }
}
