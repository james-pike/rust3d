# DK Leaderboard Backend API

Backend API for managing player statistics and leaderboards using Cloudflare Workers + Turso.

## Setup

### 1. Install Dependencies

```bash
npm install wrangler @libsql/client
```

### 2. Create Turso Database

```bash
# Install Turso CLI
curl -sSfL https://get.tur.so/install.sh | bash

# Create database
turso db create dk-leaderboard

# Get connection details
turso db show dk-leaderboard

# Create auth token
turso db tokens create dk-leaderboard
```

### 3. Initialize Schema

```bash
# Connect to database
turso db shell dk-leaderboard

# Paste contents of ../schema.sql
# Or use:
cat ../schema.sql | turso db shell dk-leaderboard
```

### 4. Configure Cloudflare Worker

```bash
# Set secrets
wrangler secret put TURSO_DATABASE_URL
# Enter: libsql://your-db.turso.io

wrangler secret put TURSO_AUTH_TOKEN
# Enter: your-token-here
```

### 5. Deploy

```bash
wrangler deploy
```

Your API will be available at: `https://dk-leaderboard-api.your-subdomain.workers.dev`

## API Endpoints

### POST /api/stats
Submit game results and update player statistics.

**Request:**
```json
{
  "player1_address": "kaspa:qz123...",
  "player2_address": "kaspa:qz456...",
  "player1_score": 10,
  "player2_score": 7,
  "session_seed": "abc123xyz"
}
```

**Response:**
```json
{
  "success": true,
  "match_id": "abc123xyz-1732473600000"
}
```

### GET /api/leaderboard?sort=kd&limit=50
Get leaderboard rankings.

**Query Parameters:**
- `sort`: `kd` (K/D ratio) or `wins` (default: `kd`)
- `limit`: Number of players to return (max: 100, default: 50)

**Response:**
```json
{
  "leaderboard": [
    {
      "kaspa_address": "kaspa:qz123...",
      "display_name": "Player1",
      "total_kills": 45,
      "total_deaths": 20,
      "wins": 8,
      "losses": 2,
      "games_played": 10,
      "kd_ratio": 2.25
    }
  ]
}
```

### GET /api/player/:address
Get individual player statistics.

**Response:**
```json
{
  "player": {
    "kaspa_address": "kaspa:qz123...",
    "display_name": "Player1",
    "total_kills": 45,
    "total_deaths": 20,
    "wins": 8,
    "losses": 2,
    "games_played": 10,
    "kd_ratio": 2.25
  }
}
```

## Local Development

```bash
wrangler dev
```

Test locally at `http://localhost:8787`

## Environment Variables

Set via `wrangler secret put`:
- `TURSO_DATABASE_URL`: Your Turso database URL
- `TURSO_AUTH_TOKEN`: Your Turso auth token
