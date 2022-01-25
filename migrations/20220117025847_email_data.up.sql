CREATE TYPE valid_status AS ENUM ('running', 'completed', 'stopped');
CREATE TABLE bulk_jobs (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_records INTEGER NOT NULL,
    job_status VALID_STATUS NOT NULL DEFAULT VALID_STATUS 'running'
);
CREATE TABLE email_results (
    job_id INTEGER,
    email_id TEXT,
    result JSONB,
    PRIMARY KEY(job_id, email_id),
    FOREIGN KEY (job_id) REFERENCES blk_vrfy_job(id)
);