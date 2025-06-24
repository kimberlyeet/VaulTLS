ALTER TABLE user_certificates ADD COLUMN pkcs12_password TEXT DEFAULT '';
UPDATE user_certificates SET pkcs12_password = '' where pkcs12_password IS NULL;