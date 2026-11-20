use crate::model::{NewResponse, ResponseRecord};
use chrono::Utc;
use uuid::Uuid;

const CONSENT: [&str; 2] = ["Vâng", "Không Nha"];

pub fn validate(input: NewResponse) -> Result<ResponseRecord, String> {
    let consent = input.consent.trim();
    if !CONSENT.contains(&consent) {
        return Err(format!("consent phải là {}", CONSENT.join(" hoặc ")));
    }

    Ok(ResponseRecord {
        id: Uuid::new_v4().to_string(),
        consent: consent.to_string(),
        free_day: date(input.free_day)?,
        food: text(input.food, "food")?,
        drink: text(input.drink, "drink")?,
        place: text(input.place, "place")?,
        created_at: Utc::now().to_rfc3339(),
        visitor_id: meta(input.visitor_id, 80, "visitorId")?,
        user_agent: meta(input.user_agent, 240, "userAgent")?,
        language: meta(input.language, 40, "language")?,
        timezone: meta(input.timezone, 80, "timezone")?,
    })
}

fn date(value: Option<String>) -> Result<Option<String>, String> {
    let Some(raw) = value else { return Ok(None) };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(None);
    }
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() > 2 || parts[2].len() > 2 {
        return Err("freeDay phải có dạng YYYY-MM-DD".into());
    }
    if parts
        .iter()
        .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("freeDay phải có dạng YYYY-MM-DD".into());
    }
    let month: u32 = parts[1].parse().unwrap_or(0);
    let day: u32 = parts[2].parse().unwrap_or(0);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err("freeDay không hợp lệ".into());
    }
    Ok(Some(value.to_string()))
}

fn meta(value: Option<String>, max: usize, field: &str) -> Result<Option<String>, String> {
    let Some(raw) = value else { return Ok(None) };
    let value = raw.trim();
    if value.len() > max {
        return Err(format!("{field} quá dài"));
    }
    Ok(if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    })
}

fn text(value: Option<String>, field: &str) -> Result<Option<String>, String> {
    let Some(raw) = value else { return Ok(None) };
    let value = raw.trim();
    if value.len() > 100 {
        return Err(format!("{field} quá dài"));
    }
    Ok(if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(consent: &str) -> NewResponse {
        NewResponse {
            consent: consent.into(),
            free_day: None,
            food: None,
            drink: None,
            place: None,
            visitor_id: None,
            user_agent: None,
            language: None,
            timezone: None,
        }
    }

    #[test]
    fn accepts_valid_consent() {
        assert!(validate(body("Vâng")).is_ok());
        assert!(validate(body("Không Nha")).is_ok());
    }

    #[test]
    fn rejects_unknown_consent() {
        assert!(validate(body("Không")).is_err());
    }

    #[test]
    fn rejects_bad_date() {
        let mut input = body("Vâng");
        input.free_day = Some("2026-13-40".into());
        assert!(validate(input).is_err());
    }

    #[test]
    fn accepts_date_and_trims_text() {
        let mut input = body("Vâng");
        input.free_day = Some(" 2026-11-28 ".into());
        input.food = Some("  Cơm  ".into());
        let record = validate(input).expect("valid");
        assert_eq!(record.free_day.as_deref(), Some("2026-11-28"));
        assert_eq!(record.food.as_deref(), Some("Cơm"));
    }

    #[test]
    fn rejects_long_text() {
        let mut input = body("Vâng");
        input.place = Some("x".repeat(101));
        assert!(validate(input).is_err());
    }
}
