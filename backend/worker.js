// Cloudflare Worker for DK Leaderboard API
// Deploy with: wrangler deploy

import { createClient } from '@libsql/client/web';

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // CORS headers for browser access
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    };

    // Handle CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    // Initialize Turso client
    const client = createClient({
      url: env.TURSO_DATABASE_URL,
      authToken: env.TURSO_AUTH_TOKEN,
    });

    try {
      // Route: POST /api/stats - Submit game results
      if (url.pathname === '/api/stats' && request.method === 'POST') {
        const data = await request.json();

        // Validate payload
        const { player1_address, player2_address, player1_score, player2_score, session_seed,
                player1_display_name, player2_display_name } = data;

        if (!player1_address || !player2_address ||
            typeof player1_score !== 'number' || typeof player2_score !== 'number') {
          return new Response(JSON.stringify({ error: 'Invalid payload' }), {
            status: 400,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          });
        }

        // Determine winner
        const winner_address = player1_score > player2_score ? player1_address : player2_address;
        const match_id = `${session_seed}-${Date.now()}`;

        // Start transaction - insert players BEFORE match history to avoid foreign key constraint
        await client.batch([
          // Update player1 stats FIRST
          {
            sql: `INSERT INTO players (kaspa_address, display_name, total_kills, total_deaths, games_played, wins, losses)
                  VALUES (?, ?, ?, ?, 1, ?, ?)
                  ON CONFLICT(kaspa_address) DO UPDATE SET
                    display_name = COALESCE(excluded.display_name, display_name),
                    total_kills = total_kills + excluded.total_kills,
                    total_deaths = total_deaths + excluded.total_deaths,
                    games_played = games_played + 1,
                    wins = wins + excluded.wins,
                    losses = losses + excluded.losses,
                    updated_at = unixepoch()`,
            args: [
              player1_address,
              player1_display_name || null,
              player1_score,
              player2_score,
              player1_score > player2_score ? 1 : 0,
              player1_score < player2_score ? 1 : 0,
            ],
          },
          // Update player2 stats SECOND
          {
            sql: `INSERT INTO players (kaspa_address, display_name, total_kills, total_deaths, games_played, wins, losses)
                  VALUES (?, ?, ?, ?, 1, ?, ?)
                  ON CONFLICT(kaspa_address) DO UPDATE SET
                    display_name = COALESCE(excluded.display_name, display_name),
                    total_kills = total_kills + excluded.total_kills,
                    total_deaths = total_deaths + excluded.total_deaths,
                    games_played = games_played + 1,
                    wins = wins + excluded.wins,
                    losses = losses + excluded.losses,
                    updated_at = unixepoch()`,
            args: [
              player2_address,
              player2_display_name || null,
              player2_score,
              player1_score,
              player2_score > player1_score ? 1 : 0,
              player2_score < player1_score ? 1 : 0,
            ],
          },
          // Insert match history LAST (after players exist)
          {
            sql: `INSERT INTO match_history (match_id, player1_address, player2_address,
                  player1_score, player2_score, winner_address, session_seed)
                  VALUES (?, ?, ?, ?, ?, ?, ?)`,
            args: [match_id, player1_address, player2_address, player1_score,
                   player2_score, winner_address, session_seed],
          },
        ]);

        return new Response(JSON.stringify({ success: true, match_id }), {
          status: 200,
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        });
      }

      // Route: GET /api/leaderboard?sort=kd&limit=50
      if (url.pathname === '/api/leaderboard' && request.method === 'GET') {
        const sort = url.searchParams.get('sort') || 'kd'; // 'kd' or 'wins'
        const limit = Math.min(parseInt(url.searchParams.get('limit') || '50'), 100);

        let query;
        if (sort === 'wins') {
          query = `
            SELECT kaspa_address, display_name, total_kills, total_deaths,
                   wins, losses, games_played,
                   CAST(total_kills AS REAL) / NULLIF(total_deaths, 0) as kd_ratio
            FROM players
            WHERE games_played > 0
            ORDER BY wins DESC, kd_ratio DESC
            LIMIT ?
          `;
        } else {
          query = `
            SELECT kaspa_address, display_name, total_kills, total_deaths,
                   wins, losses, games_played,
                   CAST(total_kills AS REAL) / NULLIF(total_deaths, 0) as kd_ratio
            FROM players
            WHERE total_deaths > 0
            ORDER BY kd_ratio DESC, wins DESC
            LIMIT ?
          `;
        }

        const result = await client.execute({ sql: query, args: [limit] });

        return new Response(JSON.stringify({ leaderboard: result.rows }), {
          status: 200,
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        });
      }

      // Route: GET /api/player/:address - Get player stats
      if (url.pathname.startsWith('/api/player/') && request.method === 'GET') {
        const address = url.pathname.split('/api/player/')[1];

        const result = await client.execute({
          sql: `SELECT kaspa_address, display_name, total_kills, total_deaths,
                       wins, losses, games_played,
                       CAST(total_kills AS REAL) / NULLIF(total_deaths, 0) as kd_ratio
                FROM players
                WHERE kaspa_address = ?`,
          args: [address],
        });

        if (result.rows.length === 0) {
          return new Response(JSON.stringify({ error: 'Player not found' }), {
            status: 404,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          });
        }

        return new Response(JSON.stringify({ player: result.rows[0] }), {
          status: 200,
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        });
      }

      // 404 for unknown routes
      return new Response(JSON.stringify({ error: 'Not found' }), {
        status: 404,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
      });

    } catch (error) {
      console.error('Error:', error);
      return new Response(JSON.stringify({ error: error.message }), {
        status: 500,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
      });
    }
  },
};
