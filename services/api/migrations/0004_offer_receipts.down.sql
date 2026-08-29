DROP INDEX IF EXISTS email_outbox_offer_idx;
DROP INDEX IF EXISTS seat_offers_workspace_receipts_idx;
ALTER TABLE email_outbox DROP COLUMN seat_offer_id;
ALTER TABLE seat_offers DROP COLUMN delivery_status;
ALTER TABLE seat_offers DROP COLUMN token_encrypted;
