use std::time::Duration;

pub fn backoff_strategy(attempt: u32, base_delay_ms: u64, max_delay_ms: u64) -> Duration {
    let exponent = attempt as u32;
    let delay = base_delay_ms.saturating_mul(2u64.saturating_pow(exponent));
    Duration::from_millis(delay.min(max_delay_ms))
}

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn humanize_job_type(raw: &str) -> String {
    let parts: Vec<String> = raw
        .split(';')
        .map(|part| {
            let part = part.trim().to_uppercase();
            match part.as_str() {
                "FLEX_TIME" => "Flex".to_string(),
                "FULL_TIME" => "Full".to_string(),
                "PART_TIME" => "Part".to_string(),
                "SEASONAL" => "Seasonal".to_string(),
                "REDUCED_TIME" => "Reduced".to_string(),
                _ => part
                    .split('_')
                    .map(|word| {
                        let word = word.to_lowercase();
                        let mut chars = word.chars();
                        match chars.next() {
                            Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                            None => "".to_string(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        })
        .collect();

    if parts.is_empty() {
        "Unknown".to_string()
    } else {
        format!("{} time", parts.join("/ "))
    }
}