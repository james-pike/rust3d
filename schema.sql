-- Turso Database Schema for DK Leaderboard
-- Created: 2025-11-24

-- Players table - stores aggregate statistics
CREATE TABLE IF NOT EXISTS players (
    kaspa_address TEXT PRIMARY KEY,
    display_name TEXT,
    total_kills INTEGER DEFAULT 0,
    total_deaths INTEGER DEFAULT 0,
    games_played INTEGER DEFAULT 0,
    wins INTEGER DEFAULT 0,
    losses INTEGER DEFAULT 0,
    created_at INTEGER DEFAULT (unixepoch()),
    updated_at INTEGER DEFAULT (unixepoch())
);

-- Match history table - stores individual game results
CREATE TABLE IF NOT EXISTS match_history (
    match_id TEXT PRIMARY KEY,
    player1_address TEXT NOT NULL,
    player2_address TEXT NOT NULL,
    player1_score INTEGER NOT NULL,
    player2_score INTEGER NOT NULL,
    winner_address TEXT,
    session_seed TEXT,
    played_at INTEGER DEFAULT (unixepoch()),
    FOREIGN KEY (player1_address) REFERENCES players(kaspa_address),
    FOREIGN KEY (player2_address) REFERENCES players(kaspa_address)
);

-- Index for fast leaderboard queries ordered by K/D ratio
CREATE INDEX IF NOT EXISTS idx_kd_ratio ON players (
    (CAST(total_kills AS REAL) / NULLIF(total_deaths, 0)) DESC
);

-- Index for fast leaderboard queries ordered by wins
CREATE INDEX IF NOT EXISTS idx_wins ON players (wins DESC);

-- Index for looking up player match history
CREATE INDEX IF NOT EXISTS idx_match_player1 ON match_history(player1_address, played_at DESC);
CREATE INDEX IF NOT EXISTS idx_match_player2 ON match_history(player2_address, played_at DESC);
