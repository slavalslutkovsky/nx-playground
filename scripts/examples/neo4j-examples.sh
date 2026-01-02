#!/bin/bash
# Neo4j Graph Database Examples
# Prerequisites: docker-compose up neo4j
# Browser UI: http://localhost:7474
# Bolt: bolt://localhost:7687
# Credentials: neo4j / password123

set -e

NEO4J_URL="http://localhost:7474"
API_URL="http://localhost:8080/api/neo4j"  # zerg-api endpoint
AUTH="neo4j:password123"

echo "=== Neo4j Direct API Examples ==="

# Helper function for Cypher queries
cypher_query() {
  curl -s -X POST "$NEO4J_URL/db/neo4j/tx/commit" \
    -u "$AUTH" \
    -H "Content-Type: application/json" \
    -d "{\"statements\": [{\"statement\": \"$1\"}]}"
}

# 1. Health check
echo -e "\n--- Health Check ---"
curl -s "$NEO4J_URL" | head -5

# 2. Create nodes (People)
echo -e "\n--- Create Person Nodes ---"
cypher_query "CREATE (p:Person {id: 'p1', name: 'Alice', age: 30, role: 'Engineer'}) RETURN p" | jq .
cypher_query "CREATE (p:Person {id: 'p2', name: 'Bob', age: 35, role: 'Manager'}) RETURN p" | jq .
cypher_query "CREATE (p:Person {id: 'p3', name: 'Charlie', age: 28, role: 'Designer'}) RETURN p" | jq .

# 3. Create nodes (Companies)
echo -e "\n--- Create Company Nodes ---"
cypher_query "CREATE (c:Company {id: 'c1', name: 'TechCorp', industry: 'Technology'}) RETURN c" | jq .
cypher_query "CREATE (c:Company {id: 'c2', name: 'DesignHub', industry: 'Design'}) RETURN c" | jq .

# 4. Create relationships
echo -e "\n--- Create Relationships ---"
cypher_query "MATCH (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'}) CREATE (a)-[:KNOWS {since: 2020}]->(b) RETURN a, b" | jq .
cypher_query "MATCH (a:Person {name: 'Bob'}), (c:Person {name: 'Charlie'}) CREATE (a)-[:MANAGES]->(c) RETURN a, c" | jq .
cypher_query "MATCH (p:Person {name: 'Alice'}), (c:Company {name: 'TechCorp'}) CREATE (p)-[:WORKS_AT {since: 2019}]->(c) RETURN p, c" | jq .
cypher_query "MATCH (p:Person {name: 'Bob'}), (c:Company {name: 'TechCorp'}) CREATE (p)-[:WORKS_AT {since: 2018}]->(c) RETURN p, c" | jq .
cypher_query "MATCH (p:Person {name: 'Charlie'}), (c:Company {name: 'DesignHub'}) CREATE (p)-[:WORKS_AT {since: 2021}]->(c) RETURN p, c" | jq .

# 5. Query: Find all people
echo -e "\n--- Find All People ---"
cypher_query "MATCH (p:Person) RETURN p.name, p.role, p.age" | jq .

# 6. Query: Find relationships
echo -e "\n--- Find Who Knows Who ---"
cypher_query "MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN a.name, r.since, b.name" | jq .

# 7. Query: Graph traversal (who works where)
echo -e "\n--- Who Works Where ---"
cypher_query "MATCH (p:Person)-[r:WORKS_AT]->(c:Company) RETURN p.name, c.name, r.since" | jq .

# 8. Query: Find colleagues (2-hop)
echo -e "\n--- Find Alice's Colleagues (via Company) ---"
cypher_query "MATCH (alice:Person {name: 'Alice'})-[:WORKS_AT]->(c:Company)<-[:WORKS_AT]-(colleague:Person) WHERE colleague <> alice RETURN colleague.name, c.name" | jq .

# 9. Query: Shortest path
echo -e "\n--- Shortest Path Alice to Charlie ---"
cypher_query "MATCH path = shortestPath((a:Person {name: 'Alice'})-[*]-(c:Person {name: 'Charlie'})) RETURN [n in nodes(path) | n.name] as path" | jq .

# 10. GraphRAG-style query: Get context around an entity
echo -e "\n--- GraphRAG: Context Around 'TechCorp' (2 hops) ---"
cypher_query "MATCH (start:Company {name: 'TechCorp'})-[*1..2]-(related) RETURN DISTINCT labels(related)[0] as type, related.name as name LIMIT 10" | jq .

# 11. Cleanup
echo -e "\n--- Cleanup (Delete All) ---"
cypher_query "MATCH (n) DETACH DELETE n" | jq .

echo -e "\n=== Neo4j Examples Complete ==="

# --- Using zerg-api endpoints ---
echo -e "\n\n=== Zerg API Neo4j Examples ==="
echo "Make sure zerg-api is running with NEO4J_URI, NEO4J_USER, NEO4J_PASSWORD"

cat << 'EOF'

# Health check via zerg-api
curl -s "$API_URL/health" | jq .

# Create node via zerg-api
curl -s -X POST "$API_URL/nodes" \
  -H "Content-Type: application/json" \
  -d '{
    "labels": ["Person", "Developer"],
    "properties": {
      "name": "Alice",
      "age": 30,
      "skills": "rust,python"
    }
  }' | jq .

# Create relationship via zerg-api
curl -s -X POST "$API_URL/relationships" \
  -H "Content-Type: application/json" \
  -d '{
    "from_id": "node-uuid-1",
    "to_id": "node-uuid-2",
    "relationship_type": "KNOWS",
    "properties": {"since": 2020}
  }' | jq .

# Execute Cypher query via zerg-api
curl -s -X POST "$API_URL/cypher" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "MATCH (p:Person)-[:WORKS_AT]->(c:Company) RETURN p.name, c.name",
    "params": {}
  }' | jq .

# GraphRAG query via zerg-api
curl -s -X POST "$API_URL/graphrag" \
  -H "Content-Type: application/json" \
  -d '{
    "entity": "TechCorp",
    "depth": 2,
    "relationship_types": ["WORKS_AT", "KNOWS"],
    "limit": 10
  }' | jq .
EOF
