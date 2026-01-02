#!/bin/bash
# ArangoDB Multi-Model Database Examples
# Prerequisites: docker-compose up arangodb
# Web UI: http://localhost:8529
# Credentials: root / rootpassword

set -e

ARANGO_URL="http://localhost:8529"
API_URL="http://localhost:8080/api/arangodb"  # zerg-api endpoint
AUTH="root:rootpassword"
DB="_system"  # Using system database for examples

echo "=== ArangoDB Direct API Examples ==="

# Helper function for AQL queries
aql_query() {
  curl -s -X POST "$ARANGO_URL/_db/$DB/_api/cursor" \
    -u "$AUTH" \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"$1\"}"
}

# 1. Health check
echo -e "\n--- Health Check ---"
curl -s "$ARANGO_URL/_api/version" -u "$AUTH" | jq .

# 2. Create a document collection
echo -e "\n--- Create 'users' Collection ---"
curl -s -X POST "$ARANGO_URL/_db/$DB/_api/collection" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"name": "users", "type": 2}' | jq .

# 3. Create an edge collection
echo -e "\n--- Create 'friendships' Edge Collection ---"
curl -s -X POST "$ARANGO_URL/_db/$DB/_api/collection" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"name": "friendships", "type": 3}' | jq .

# 4. Create documents
echo -e "\n--- Create User Documents ---"
curl -s -X POST "$ARANGO_URL/_db/$DB/_api/document/users" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"_key": "alice", "name": "Alice", "age": 30, "city": "NYC"}' | jq .

curl -s -X POST "$ARANGO_URL/_db/$DB/_api/document/users" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"_key": "bob", "name": "Bob", "age": 35, "city": "LA"}' | jq .

curl -s -X POST "$ARANGO_URL/_db/$DB/_api/document/users" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"_key": "charlie", "name": "Charlie", "age": 28, "city": "NYC"}' | jq .

# 5. Create edges (friendships)
echo -e "\n--- Create Friendship Edges ---"
curl -s -X POST "$ARANGO_URL/_db/$DB/_api/document/friendships" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"_from": "users/alice", "_to": "users/bob", "since": 2020}' | jq .

curl -s -X POST "$ARANGO_URL/_db/$DB/_api/document/friendships" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"_from": "users/bob", "_to": "users/charlie", "since": 2021}' | jq .

curl -s -X POST "$ARANGO_URL/_db/$DB/_api/document/friendships" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"_from": "users/alice", "_to": "users/charlie", "since": 2022}' | jq .

# 6. Create a graph
echo -e "\n--- Create 'social' Graph ---"
curl -s -X POST "$ARANGO_URL/_db/$DB/_api/gharial" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "social",
    "edgeDefinitions": [
      {
        "collection": "friendships",
        "from": ["users"],
        "to": ["users"]
      }
    ]
  }' | jq .

# 7. AQL: Query all users
echo -e "\n--- AQL: All Users ---"
aql_query "FOR u IN users RETURN u" | jq .

# 8. AQL: Filter by city
echo -e "\n--- AQL: Users in NYC ---"
aql_query "FOR u IN users FILTER u.city == 'NYC' RETURN u" | jq .

# 9. AQL: Join documents
echo -e "\n--- AQL: Friendships with Names ---"
aql_query "FOR f IN friendships LET from = DOCUMENT(f._from) LET to = DOCUMENT(f._to) RETURN {from: from.name, to: to.name, since: f.since}" | jq .

# 10. Graph Traversal: Alice's friends
echo -e "\n--- Graph: Alice's Friends (1 hop) ---"
aql_query "FOR v, e, p IN 1..1 OUTBOUND 'users/alice' GRAPH 'social' RETURN {friend: v.name, since: e.since}" | jq .

# 11. Graph Traversal: Friends of friends
echo -e "\n--- Graph: Alice's Friends of Friends (2 hops) ---"
aql_query "FOR v, e, p IN 1..2 OUTBOUND 'users/alice' GRAPH 'social' RETURN DISTINCT {name: v.name, distance: LENGTH(p.edges)}" | jq .

# 12. AQL: Aggregation
echo -e "\n--- AQL: Users by City ---"
aql_query "FOR u IN users COLLECT city = u.city WITH COUNT INTO count RETURN {city, count}" | jq .

# 13. Full-text search (if available)
echo -e "\n--- AQL: Search by Name Pattern ---"
aql_query "FOR u IN users FILTER CONTAINS(LOWER(u.name), 'ali') RETURN u" | jq .

# 14. Update a document
echo -e "\n--- Update Alice's Age ---"
curl -s -X PATCH "$ARANGO_URL/_db/$DB/_api/document/users/alice" \
  -u "$AUTH" \
  -H "Content-Type: application/json" \
  -d '{"age": 31}' | jq .

# 15. Get document
echo -e "\n--- Get Alice ---"
curl -s "$ARANGO_URL/_db/$DB/_api/document/users/alice" -u "$AUTH" | jq .

# 16. Cleanup
echo -e "\n--- Cleanup ---"
curl -s -X DELETE "$ARANGO_URL/_db/$DB/_api/gharial/social?dropCollections=false" -u "$AUTH" | jq .
curl -s -X DELETE "$ARANGO_URL/_db/$DB/_api/collection/friendships" -u "$AUTH" | jq .
curl -s -X DELETE "$ARANGO_URL/_db/$DB/_api/collection/users" -u "$AUTH" | jq .

echo -e "\n=== ArangoDB Examples Complete ==="

# --- Using zerg-api endpoints ---
echo -e "\n\n=== Zerg API ArangoDB Examples ==="
echo "Make sure zerg-api is running with ARANGO_URL, ARANGO_USER, ARANGO_PASSWORD, ARANGO_DATABASE"

cat << 'EOF'

# Health check via zerg-api
curl -s "$API_URL/health" | jq .

# List collections via zerg-api
curl -s "$API_URL/collections" | jq .

# Create collection via zerg-api
curl -s -X POST "$API_URL/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "products",
    "collection_type": "document"
  }' | jq .

# Create document via zerg-api
curl -s -X POST "$API_URL/collections/products/documents" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Laptop",
    "price": 999.99,
    "category": "electronics"
  }' | jq .

# Get document via zerg-api
curl -s "$API_URL/collections/products/documents/laptop-key" | jq .

# Execute AQL via zerg-api
curl -s -X POST "$API_URL/aql" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "FOR p IN products FILTER p.price > @minPrice RETURN p",
    "bind_vars": {"minPrice": 500}
  }' | jq .

# Graph traversal via zerg-api
curl -s -X POST "$API_URL/traverse" \
  -H "Content-Type: application/json" \
  -d '{
    "start_vertex": "users/alice",
    "graph": "social",
    "direction": "outbound",
    "min_depth": 1,
    "max_depth": 2,
    "limit": 10
  }' | jq .

# Create edge via zerg-api
curl -s -X POST "$API_URL/collections/friendships/edges" \
  -H "Content-Type: application/json" \
  -d '{
    "from": "users/alice",
    "to": "users/bob",
    "label": "FRIENDS_WITH",
    "since": 2020
  }' | jq .
EOF
