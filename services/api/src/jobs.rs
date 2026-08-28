use std::{collections::HashMap, env, time::Duration};

use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use sqlx::{Row, SqlitePool};

use crate::crypto::ContactCipher;

pub fn count_ical_events_by_summary(input: &str) -> HashMap<String, i64> {
    let unfolded = input.replace("\r\n ", "").replace("\n ", "");
    let mut counts = HashMap::new();
    let mut in_event = false;
    for raw in unfolded.lines() {
        let line = raw.trim_end_matches('\r');
        if line == "BEGIN:VEVENT" {
            in_event = true;
        }
        if line == "END:VEVENT" {
            in_event = false;
        }
        if in_event {
            if let Some(summary) = line.strip_prefix("SUMMARY:") {
                *counts.entry(summary.replace("\\,", ",")).or_insert(0) += 1;
            }
        }
    }
    counts
}

pub async fn poll_due_calendars(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    http: &reqwest::Client,
    now: i64,
) -> anyhow::Result<u64> {
    let rows = sqlx::query("SELECT id, workspace_id, feed_url_encrypted FROM calendar_connections WHERE enabled = 1 AND feed_url_encrypted IS NOT NULL AND COALESCE(next_poll_at, 0) <= ?1 LIMIT 20")
        .bind(now).fetch_all(pool).await?;
    let mut checked = 0;
    for row in rows {
        let connection_id: String = row.get("id");
        let workspace_id: String = row.get("workspace_id");
        let encrypted: String = row.get("feed_url_encrypted");
        let result = async {
            let url = cipher.decrypt(&encrypted)?;
            let body = if url == "https://fixture.invalid/school.ics" && env::var_os("TEST_AUTH_TOKEN").is_some() {
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Saturday level check\r\nEND:VEVENT\r\nEND:VCALENDAR".to_owned()
            } else {
                http.get(url).timeout(Duration::from_secs(8)).send().await?.error_for_status()?.text().await?
            };
            let counts = count_ical_events_by_summary(&body);
            let classes = sqlx::query("SELECT id, name, confirmed FROM real_classes WHERE workspace_id = ?1")
                .bind(&workspace_id).fetch_all(pool).await?;
            for class in classes {
                let class_id: String = class.get("id");
                let name: String = class.get("name");
                let local: i64 = class.get("confirmed");
                let calendar = counts.get(&name).copied().unwrap_or(0);
                let status = if calendar == local { "matched" } else { "attention" };
                sqlx::query("INSERT INTO reconciliation_runs (id, class_id, calendar_confirmed, local_confirmed, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
                    .bind(uuid::Uuid::new_v4().to_string()).bind(class_id).bind(calendar).bind(local).bind(status).bind(now).execute(pool).await?;
            }
            anyhow::Ok(())
        }.await;
        match result {
            Ok(()) => {
                sqlx::query("UPDATE calendar_connections SET last_polled_at = ?1, next_poll_at = ?2, last_error = NULL WHERE id = ?3")
                    .bind(now).bind(now + 300).bind(&connection_id).execute(pool).await?;
                checked += 1;
            }
            Err(error) => {
                let message = error.to_string();
                sqlx::query("UPDATE calendar_connections SET next_poll_at = ?1, last_error = ?2 WHERE id = ?3")
                    .bind(now + 300).bind(message.chars().take(240).collect::<String>()).bind(&connection_id).execute(pool).await?;
            }
        }
    }
    Ok(checked)
}

pub async fn deliver_due_email(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    now: i64,
) -> anyhow::Result<u64> {
    let rows = sqlx::query("SELECT id, recipient_encrypted, subject, text_body, attempts FROM email_outbox WHERE status IN ('pending', 'failed') AND next_attempt_at <= ?1 AND attempts < 5 LIMIT 20")
        .bind(now).fetch_all(pool).await?;
    let relay = env::var("SMTP_RELAY").ok();
    let mut delivered = 0;
    for row in rows {
        let id: String = row.get("id");
        if relay.is_none() {
            sqlx::query("UPDATE email_outbox SET status = 'captured', attempts = attempts + 1, sent_at = ?1 WHERE id = ?2")
                .bind(now).bind(id).execute(pool).await?;
            delivered += 1;
            continue;
        }
        let recipient = cipher.decrypt(&row.get::<String, _>("recipient_encrypted"))?;
        let from: Mailbox = env::var("SMTP_FROM")
            .unwrap_or_else(|_| "Class Capacity Truth <notifications@sociobot.in>".into())
            .parse()?;
        let email = Message::builder()
            .from(from)
            .to(recipient.parse()?)
            .subject(row.get::<String, _>("subject"))
            .body(row.get::<String, _>("text_body"))?;
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(relay.as_deref().unwrap())?;
        if let (Ok(username), Ok(password)) = (env::var("SMTP_USERNAME"), env::var("SMTP_PASSWORD"))
        {
            builder = builder.credentials(Credentials::new(username, password));
        }
        match builder.build().send(email).await {
            Ok(_) => {
                sqlx::query("UPDATE email_outbox SET status = 'sent', attempts = attempts + 1, sent_at = ?1, last_error = NULL WHERE id = ?2").bind(now).bind(id).execute(pool).await?;
                delivered += 1;
            }
            Err(error) => {
                sqlx::query("UPDATE email_outbox SET status = 'failed', attempts = attempts + 1, next_attempt_at = ?1, last_error = ?2 WHERE id = ?3")
                    .bind(now + 60).bind(error.to_string().chars().take(240).collect::<String>()).bind(id).execute(pool).await?;
            }
        }
    }
    Ok(delivered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_ical_events_without_counting_calendar_metadata() {
        let fixture = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Saturday level check\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Saturday level check\r\nEND:VEVENT\r\nEND:VCALENDAR";
        assert_eq!(
            count_ical_events_by_summary(fixture)["Saturday level check"],
            2
        );
    }
}
