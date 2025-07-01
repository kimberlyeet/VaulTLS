ALTER TABLE user_certificates ADD COLUMN type INTEGER DEFAULT 0;
UPDATE user_certificates SET type = 0 where type IS NULL;