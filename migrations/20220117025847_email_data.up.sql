CREATE TYPE valid_status AS ENUM ('pending', 'running', 'completed', 'stopped');
CREATE TABLE blk_vrfy_job (
    id SERIAL PRIMARY KEY,
    job_uuid UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_records INTEGER NOT NULL,
    total_processed INTEGER NOT NULL DEFAULT 0,
    summary_total_safe INTEGER NOT NULL DEFAULT 0,
    summary_total_invalid INTEGER NOT NULL DEFAULT 0,
    summary_total_risky INTEGER NOT NULL DEFAULT 0,
    summary_total_unknown INTEGER NOT NULL DEFAULT 0,
    job_status VALID_STATUS NOT NULL DEFAULT VALID_STATUS 'pending'
);
CREATE TABLE ema_vrfy_rec (
    job_id INTEGER,
    record_id INTEGER,
    status VALID_STATUS NOT NULL DEFAULT VALID_STATUS 'pending',
    email_id TEXT,
    PRIMARY KEY(job_id, record_id),
    FOREIGN KEY (job_id) REFERENCES blk_vrfy_job(id)
);