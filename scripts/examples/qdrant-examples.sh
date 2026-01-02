#!/bin/bash
# Qdrant Vector Database Examples
# Prerequisites: docker-compose up qdrant
# API runs on: http://localhost:6333

set -e

QDRANT_URL="http://localhost:6333"
API_URL="http://localhost:8080/api/qdrant"  # zerg-api endpoint

echo "=== Qdrant Direct API Examples ==="

# 1. Health check
echo -e "\n--- Health Check ---"
curl -s "$QDRANT_URL/healthz" | jq .

# 2. Create a collection
echo -e "\n--- Create Collection 'documents' ---"
curl -s -X PUT "$QDRANT_URL/collections/documents" \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    }
  }' | jq .

# 3. List collections
echo -e "\n--- List Collections ---"
curl -s "$QDRANT_URL/collections" | jq .

# 4. Insert vectors (upsert points)
echo -e "\n--- Upsert Points ---"
curl -s -X PUT "$QDRANT_URL/collections/documents/points?wait=true" \
  -H "Content-Type: application/json" \
  -d '{
    "points": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "vector": [0.1, 0.2, 0.3, '"$(python3 -c "print(','.join(['0.1']*381))")"'],
        "payload": {
          "content": "Introduction to machine learning",
          "category": "tech",
          "author": "John Doe"
        }
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "vector": [0.2, 0.3, 0.4, '"$(python3 -c "print(','.join(['0.2']*381))")"'],
        "payload": {
          "content": "Deep learning fundamentals",
          "category": "tech",
          "author": "Jane Smith"
        }
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440003",
        "vector": [0.9, 0.1, 0.1, '"$(python3 -c "print(','.join(['0.5']*381))")"'],
        "payload": {
          "content": "Cooking Italian pasta",
          "category": "food",
          "author": "Mario Rossi"
        }
      }
    ]
  }' | jq .

# 5. Search for similar vectors
echo -e "\n--- Vector Search (find tech articles) ---"
curl -s -X POST "$QDRANT_URL/collections/documents/points/search" \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.15, 0.25, 0.35, '"$(python3 -c "print(','.join(['0.15']*381))")"'],
    "limit": 3,
    "with_payload": true
  }' | jq .

# 6. Search with filter
echo -e "\n--- Search with Filter (category=tech) ---"
curl -s -X POST "$QDRANT_URL/collections/documents/points/search" \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.15, 0.25, 0.35, '"$(python3 -c "print(','.join(['0.15']*381))")"'],
    "limit": 3,
    "filter": {
      "must": [
        {"key": "category", "match": {"value": "tech"}}
      ]
    },
    "with_payload": true
  }' | jq .

# 7. Get collection info
echo -e "\n--- Collection Info ---"
curl -s "$QDRANT_URL/collections/documents" | jq .

# 8. Delete collection (cleanup)
echo -e "\n--- Delete Collection (cleanup) ---"
curl -s -X DELETE "$QDRANT_URL/collections/documents" | jq .

echo -e "\n=== Qdrant Examples Complete ==="

# --- Using zerg-api endpoints ---
echo -e "\n\n=== Zerg API Qdrant Examples ==="
echo "Make sure zerg-api is running with QDRANT_URL=http://localhost:6333"

cat << 'EOF'

# Health check via zerg-api
curl -s "$API_URL/health" | jq .

# Create collection via zerg-api
curl -s -X POST "$API_URL/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "rag_documents",
    "vector_size": 1536,
    "distance": "cosine"
  }' | jq .

# Upsert documents via zerg-api
curl -s -X POST "$API_URL/documents" \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "rag_documents",
    "documents": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "content": "Machine learning is a subset of AI",
        "metadata": {"source": "wiki", "topic": "ml"},
        "embedding": [0.1, 0.2, ...]
      }
    ]
  }' | jq .

# Search via zerg-api
curl -s -X POST "$API_URL/search" \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "rag_documents",
    "query_vector": [0.1, 0.2, ...],
    "limit": 5
  }' | jq .
EOF
