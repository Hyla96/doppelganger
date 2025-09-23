-- Database schema for behavior comparison results

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Table to store comparison requests and results
CREATE TABLE IF NOT EXISTS comparison_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    request_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Request details
    method VARCHAR(10) NOT NULL,
    path TEXT NOT NULL,
    request_headers JSONB,
    request_body TEXT,

    -- Primary service response
    primary_service_name VARCHAR(255) NOT NULL,
    primary_status_code INTEGER NOT NULL,
    primary_response_headers JSONB,
    primary_response_body TEXT,
    primary_response_time_ms BIGINT NOT NULL,

    -- Shadow service responses (as JSONB array)
    shadow_responses JSONB NOT NULL,

    -- Comparison results
    status_match BOOLEAN NOT NULL,
    body_match BOOLEAN NOT NULL,
    header_differences JSONB,
    response_time_diff_ms BIGINT,

    -- Additional metadata
    metadata JSONB,

    -- Indexes
    CONSTRAINT idx_request_id_created UNIQUE (request_id, created_at)
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_comparison_results_created_at ON comparison_results(created_at);
CREATE INDEX IF NOT EXISTS idx_comparison_results_request_id ON comparison_results(request_id);
CREATE INDEX IF NOT EXISTS idx_comparison_results_primary_service ON comparison_results(primary_service_name);
CREATE INDEX IF NOT EXISTS idx_comparison_results_status_match ON comparison_results(status_match);
CREATE INDEX IF NOT EXISTS idx_comparison_results_body_match ON comparison_results(body_match);

-- Table for storing aggregated comparison statistics
CREATE TABLE IF NOT EXISTS comparison_stats (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    date DATE NOT NULL,
    primary_service_name VARCHAR(255) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    status_matches BIGINT NOT NULL DEFAULT 0,
    body_matches BIGINT NOT NULL DEFAULT 0,
    avg_response_time_diff_ms BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT idx_stats_date_service UNIQUE (date, primary_service_name)
);

CREATE INDEX IF NOT EXISTS idx_comparison_stats_date ON comparison_stats(date);
CREATE INDEX IF NOT EXISTS idx_comparison_stats_service ON comparison_stats(primary_service_name);