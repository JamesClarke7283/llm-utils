use chrono::NaiveDate;
use fancy_regex::Regex;

pub fn parse_knowledge_cutoff(date_str: Option<String>) -> Option<i64> {
    if let Some(date) = date_str {
        if date.to_lowercase() == "online" {
            Some(0)
        } else if let Ok(epoch) = date.parse::<i64>() {
            Some(epoch)
        } else {
            let parts: Vec<&str> = date.split('/').collect();
            if parts.len() == 3 {
                let day = parts[0].parse::<u32>().unwrap_or(1);
                let month = parts[1].parse::<u32>().unwrap_or(1);
                let year = parts[2].parse::<i32>().unwrap_or(2000);
                let naive_date = NaiveDate::from_ymd_opt(year, month, day)
                    .unwrap_or(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
                let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
                Some(naive_datetime.and_utc().timestamp())
            } else {
                None
            }
        }
    } else {
        None
    }
}

pub fn format_knowledge_cutoff(epoch: i64) -> String {
    if epoch == 0 {
        "Online".to_string()
    } else {
        let datetime = chrono::DateTime::from_timestamp(epoch, 0).unwrap();
        datetime.format("%d/%m/%Y").to_string()
    }
}

pub fn format_context_length(context_length: Option<u32>) -> String {
    context_length
        .map(|c| {
            let re = Regex::new(r"(?<=\d)(?=(\d{3})+$)").unwrap();
            re.replace_all(&c.to_string(), ",").to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}