# Turso + Cloudflare Workers Setup Guide

Complete step-by-step guide to deploy your leaderboard backend.

---

## Part 1: Set Up Turso Database

### Step 1: Install Turso CLI

```bash
curl -sSfL https://get.tur.so/install.sh | bash

# Add to PATH (if not automatically added)
echo 'export PATH="$HOME/.turso:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify installation
turso --version
```

### Step 2: Authenticate with Turso

```bash
turso auth login
```

This will open your browser. Log in with your Turso account.

### Step 3: Create Database

```bash
# Create the database
turso db create dk-leaderboard

# You should see output like:
# Created database dk-leaderboard at [some-location]
# URL: libsql://dk-leaderboard-[your-org].turso.io
```

### Step 4: Get Database URL

```bash
turso db show dk-leaderboard
```

**Copy the URL** - it looks like:
```
libsql://dk-leaderboard-[your-org].turso.io
```

Save this! You'll need it for Cloudflare Workers.

### Step 5: Create Auth Token

```bash
turso db tokens create dk-leaderboard
```

**Copy the token** - it's a long string like:
```
eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...
```

Save this too! You'll need it for Cloudflare Workers.

### Step 6: Initialize Database Schema

```bash
# Navigate to your project directory (if not already there)
cd /home/james/dk

# Apply the schema
turso db shell dk-leaderboard < schema.sql
```

You should see:
```
(no output means success!)
```

### Step 7: Verify Schema

```bash
# Open the database shell
turso db shell dk-leaderboard

# In the shell, list tables:
.tables

# You should see:
# match_history  players

# Check the players table structure:
.schema players

# Exit the shell:
.exit
```

✅ **Turso database is ready!**

---

## Part 2: Set Up Cloudflare Workers

### Step 1: Install Node.js (if not installed)

```bash
# Check if Node.js is installed
node --version

# If not installed, use nvm:
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 20
nvm use 20
```

### Step 2: Install Wrangler CLI

```bash
npm install -g wrangler

# Verify installation
wrangler --version
```

### Step 3: Authenticate with Cloudflare

```bash
wrangler login
```

This will open your browser. Log in with Cloudflare (create free account if needed).

### Step 4: Create Worker Project

```bash
cd /home/james/dk/backend

# Install dependencies
npm install @libsql/client
```

### Step 5: Set Secrets (Database Credentials)

```bash
# Set database URL
wrangler secret put TURSO_DATABASE_URL
# When prompted, paste your Turso URL: libsql://dk-leaderboard-[your-org].turso.io

# Set auth token
wrangler secret put TURSO_AUTH_TOKEN
# When prompted, paste your Turso token: eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...
```

### Step 6: Deploy Worker

```bash
# Deploy to Cloudflare
wrangler deploy

# You should see output like:
# Published dk-leaderboard-api (X.XX sec)
#   https://dk-leaderboard-api.[your-subdomain].workers.dev
```

**Copy the worker URL!** It looks like:
```
https://dk-leaderboard-api.[your-subdomain].workers.dev
```

✅ **Backend API is live!**

---

## Part 3: Update Game Code with API URLs

### Step 1: Update Stats Submission URL

Open `src/game/leaderboard.rs` and find line 120:

```rust
// TODO: Replace with your deployed Cloudflare Worker URL
let api_url = "https://dk-leaderboard-api.your-subdomain.workers.dev/api/stats";
```

Replace with **YOUR** worker URL:

```rust
let api_url = "https://dk-leaderboard-api.[your-subdomain].workers.dev/api/stats";
```

### Step 2: Update Leaderboard Fetch URL

Open `src/ui/leaderboard.rs` and find line 122:

```rust
// TODO: Replace with your deployed Cloudflare Worker URL
let api_url = "https://dk-leaderboard-api.your-subdomain.workers.dev/api/leaderboard?sort=kd&limit=10";
```

Replace with **YOUR** worker URL:

```rust
let api_url = "https://dk-leaderboard-api.[your-subdomain].workers.dev/api/leaderboard?sort=kd&limit=10";
```

### Step 3: Rebuild the Game

```bash
cd /home/james/dk

# For WASM (production):
trunk build --release

# Or for desktop testing:
cargo build
```

✅ **Game is now connected to your live backend!**

---

## Part 4: Test the Setup

### Test 1: Check API is Live

```bash
# Test leaderboard endpoint (should return empty array at first)
curl https://dk-leaderboard-api.[your-subdomain].workers.dev/api/leaderboard?sort=kd&limit=10
```

Expected output:
```json
{"leaderboard":[]}
```

### Test 2: Submit Test Data

```bash
curl -X POST https://dk-leaderboard-api.[your-subdomain].workers.dev/api/stats \
  -H "Content-Type: application/json" \
  -d '{
    "player1_address": "kaspa:test123",
    "player2_address": "kaspa:test456",
    "player1_score": 10,
    "player2_score": 7,
    "session_seed": "test123"
  }'
```

Expected output:
```json
{"success":true,"match_id":"test123-1234567890"}
```

### Test 3: Check Leaderboard Again

```bash
curl https://dk-leaderboard-api.[your-subdomain].workers.dev/api/leaderboard?sort=kd&limit=10
```

Expected output (now with data):
```json
{
  "leaderboard": [
    {
      "kaspa_address": "kaspa:test123",
      "display_name": null,
      "total_kills": 10,
      "total_deaths": 7,
      "wins": 1,
      "losses": 0,
      "games_played": 1,
      "kd_ratio": 1.4285714285714286
    },
    {
      "kaspa_address": "kaspa:test456",
      "display_name": null,
      "total_kills": 7,
      "total_deaths": 10,
      "wins": 0,
      "losses": 1,
      "games_played": 1,
      "kd_ratio": 0.7
    }
  ]
}
```

✅ **Backend is working!**

### Test 4: Play a Real Game

```bash
# Run the game
trunk serve

# Or for desktop:
cargo run
```

1. Connect Kasware wallet
2. Enter lobby (leaderboard should show test data)
3. Start matchmaking
4. Play until 10 kills
5. Game ends → Stats submitted automatically
6. Return to lobby → **Leaderboard updates with your real stats!**

---

## Part 5: Maintenance & Monitoring

### View Database Data

```bash
# Open database shell
turso db shell dk-leaderboard

# View all players
SELECT * FROM players ORDER BY (CAST(total_kills AS REAL) / NULLIF(total_deaths, 0)) DESC LIMIT 10;

# View recent matches
SELECT * FROM match_history ORDER BY played_at DESC LIMIT 5;

# Exit
.exit
```

### View Worker Logs

```bash
# View real-time logs
wrangler tail

# Or check Cloudflare dashboard:
# https://dash.cloudflare.com → Workers & Pages → dk-leaderboard-api → Logs
```

### Update Worker Code

After making changes to `backend/worker.js`:

```bash
cd /home/james/dk/backend
wrangler deploy
```

### Reset Database (if needed)

```bash
# Drop and recreate
turso db shell dk-leaderboard

# In shell:
DROP TABLE IF EXISTS match_history;
DROP TABLE IF EXISTS players;
.exit

# Reapply schema
turso db shell dk-leaderboard < ../schema.sql
```

---

## Quick Reference

### Important URLs

Save these for reference:

| Service | URL | Purpose |
|---------|-----|---------|
| Turso Database | `libsql://dk-leaderboard-[your-org].turso.io` | Database connection |
| Worker API | `https://dk-leaderboard-api.[your-subdomain].workers.dev` | Backend API |
| Stats Endpoint | `/api/stats` | Submit game results |
| Leaderboard Endpoint | `/api/leaderboard?sort=kd&limit=10` | Get rankings |
| Player Endpoint | `/api/player/:address` | Get player stats |

### Common Commands

```bash
# Database
turso db list                          # List all databases
turso db show dk-leaderboard          # Show database info
turso db shell dk-leaderboard         # Open database shell

# Worker
wrangler deploy                        # Deploy changes
wrangler tail                          # View logs
wrangler secret list                   # List secrets (names only)
wrangler secret put TURSO_DATABASE_URL # Update secret

# Testing
curl https://[worker-url]/api/leaderboard  # Get leaderboard
curl -X POST https://[worker-url]/api/stats # Submit stats
```

---

## Troubleshooting

### Error: "Database not found"

```bash
# List your databases
turso db list

# Make sure dk-leaderboard exists
# If not, recreate:
turso db create dk-leaderboard
turso db shell dk-leaderboard < schema.sql
```

### Error: "Invalid auth token"

```bash
# Regenerate token
turso db tokens create dk-leaderboard

# Update worker secret
wrangler secret put TURSO_AUTH_TOKEN
```

### Error: "CORS error" in browser

Check `backend/worker.js` has CORS headers:

```javascript
const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
};
```

### Error: "Worker not deploying"

```bash
# Check you're logged in
wrangler whoami

# If not logged in:
wrangler login

# Try deploying again
wrangler deploy
```

### Leaderboard not updating in game

1. Check browser console for errors
2. Verify API URLs in code match your worker URL
3. Check worker logs: `wrangler tail`
4. Test API directly with curl

---

## Cost Estimate

### Free Tier Limits

**Turso (Free):**
- 9 GB storage
- 500 databases
- Unlimited reads
- $0/month

**Cloudflare Workers (Free):**
- 100,000 requests/day
- $0/month

**Total:** **$0/month** for most indie games! 🎉

### Paid Plans (if you exceed free tier)

**Turso Starter:** $29/month
- More storage and databases

**Cloudflare Workers Paid:** $5/month
- 10 million requests/month
- No daily limits

You'll likely never need paid plans unless your game goes viral!

---

## Summary Checklist

- [ ] Install Turso CLI
- [ ] Create Turso account & database
- [ ] Get database URL and auth token
- [ ] Apply schema.sql to database
- [ ] Install Wrangler CLI
- [ ] Login to Cloudflare
- [ ] Deploy worker with secrets
- [ ] Update API URLs in game code
- [ ] Rebuild game (trunk build --release)
- [ ] Test with curl
- [ ] Test with real game
- [ ] Celebrate! 🎉

---

## Need Help?

Common issues and solutions:
- Turso docs: https://docs.turso.tech
- Cloudflare Workers docs: https://developers.cloudflare.com/workers
- Your backend code: `/home/james/dk/backend/`
- Your schema: `/home/james/dk/schema.sql`

Your leaderboard should now be fully functional! 🚀
