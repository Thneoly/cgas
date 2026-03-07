-- CGAS Alpha Environment Database Initialization
-- PostgreSQL 15

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create CGAS schema
CREATE SCHEMA IF NOT EXISTS cgas;

-- Grant permissions
GRANT ALL ON SCHEMA cgas TO cgas;
GRANT ALL ON ALL TABLES IN SCHEMA cgas TO cgas;
GRANT ALL ON ALL SEQUENCES IN SCHEMA cgas TO cgas;

-- Create basic tables (application will manage schema migrations)
-- These are placeholder tables for initial setup

CREATE TABLE IF NOT EXISTS cgas.system_info (
    id SERIAL PRIMARY KEY,
    key VARCHAR(255) UNIQUE NOT NULL,
    value TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial system info
INSERT INTO cgas.system_info (key, value) VALUES
    ('environment', 'alpha'),
    ('version', 'v3.0.0-alpha'),
    ('deployed_at', CURRENT_TIMESTAMP)
ON CONFLICT (key) DO NOTHING;

-- Create index for performance
CREATE INDEX IF NOT EXISTS idx_system_info_key ON cgas.system_info(key);

-- Log initialization
DO $$
BEGIN
    RAISE NOTICE 'CGAS Alpha database initialized successfully';
END $$;
