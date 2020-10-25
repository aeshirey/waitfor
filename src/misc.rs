use std::time::Duration;

pub fn parse_duration(duration: &str) -> Result<Duration, ()> {
    let mut chars = duration.chars().peekable();

    let mut total_delay = 0.;

    let mut acc = 0;
    while let Some(&c) = chars.peek() {
        match c {
            '0'..='9' => {
                acc *= 10;
                acc += chars.next().unwrap().to_digit(10).unwrap();
            }
            'h' => {
                total_delay += acc as f64 * 3600.;
                acc = 0;
                chars.next().unwrap();
            }
            'm' => {
                total_delay += acc as f64 * 60.;
                acc = 0;
                chars.next().unwrap();
            }
            's' => {
                total_delay += acc as f64;
                acc = 0;
                chars.next().unwrap();
            }
            _ => return Err(()),
        }
    }

    total_delay += acc as f64;

    let d = Duration::from_secs_f64(total_delay);

    Ok(d)
}

pub fn parse_http_get(urlarg: &str) -> Result<(u16, String), ()> {
    if urlarg.len() < 5 {
        // too short
        Err(())
    } else if urlarg.starts_with("http://") || urlarg.starts_with("https://") {
        Ok((200, urlarg.into()))
    } else {
        let urlbytes: Vec<char> = urlarg.chars().collect();

        // Format: 404,http://
        if urlbytes[3] == ',' && urlbytes[0..3].iter().all(|c| c.is_digit(10)) {
            let status = 100 * (urlbytes[0] as u16 - '0' as u16)
                + 10 * (urlbytes[1] as u16 - '0' as u16)
                + (urlbytes[2] as u16 - '0' as u16);

            let url: String = urlbytes[4..].iter().collect();
            return Ok((status, url));
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_get_pass() {
        assert_eq!(
            Ok((200, "http://google.com".to_string())),
            parse_http_get("200,http://google.com")
        );

        assert_eq!(
            Ok((200, "http://google.com".to_string())),
            parse_http_get("http://google.com")
        );

        assert_eq!(
            Ok((404, "http://does-not-exist.com".to_string())),
            parse_http_get("404,http://does-not-exist.com")
        );
    }

    #[test]
    fn parse_get_fail() {
        assert_eq!(Err(()), parse_http_get(""));
        assert_eq!(Err(()), parse_http_get("http"));
        assert_eq!(Err(()), parse_http_get("xxx,http://does-not-exist.com"));
        assert_eq!(Err(()), parse_http_get("ftp://does-not-exist.com"));
    }
}
