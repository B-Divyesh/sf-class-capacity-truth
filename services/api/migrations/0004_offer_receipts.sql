ALTER TABLE seat_offers ADD COLUMN token_encrypted TEXT;
ALTER TABLE seat_offers ADD COLUMN delivery_status TEXT NOT NULL DEFAULT 'legacy_recorded';
ALTER TABLE email_outbox ADD COLUMN seat_offer_id TEXT REFERENCES seat_offers(id) ON DELETE CASCADE;

CREATE INDEX seat_offers_workspace_receipts_idx
  ON seat_offers(class_id, created_at DESC);
CREATE INDEX email_outbox_offer_idx ON email_outbox(seat_offer_id);
