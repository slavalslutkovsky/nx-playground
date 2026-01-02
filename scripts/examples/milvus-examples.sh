#!/bin/bash
# Milvus Vector Database Examples
# Prerequisites: docker-compose up milvus milvus-etcd milvus-minio
# REST API: http://localhost:19121 (or via SDK on port 19530)

set -e

# Milvus 2.x REST API endpoint
MILVUS_URL="http://localhost:19121/v2/vectordb"
API_URL="http://localhost:8080/api/milvus"  # zerg-api endpoint

echo "=== Milvus Direct API Examples ==="

# Note: Milvus REST API availability depends on version
# These examples use the v2 REST API

# 1. Health check
echo -e "\n--- Health Check ---"
curl -s "$MILVUS_URL/collections/list" | jq . || echo "Note: REST API may need different endpoint"

# 2. Create a collection
echo -e "\n--- Create Collection 'embeddings' ---"
curl -s -X POST "$MILVUS_URL/collections/create" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "embeddings",
    "dimension": 128,
    "metricType": "COSINE",
    "primaryFieldName": "id",
    "vectorFieldName": "vector"
  }' | jq .

# 3. List collections
echo -e "\n--- List Collections ---"
curl -s "$MILVUS_URL/collections/list" | jq .

# 4. Describe collection
echo -e "\n--- Describe Collection ---"
curl -s -X POST "$MILVUS_URL/collections/describe" \
  -H "Content-Type: application/json" \
  -d '{"collectionName": "embeddings"}' | jq .

# 5. Insert vectors
echo -e "\n--- Insert Vectors ---"
# Generate random 128-dim vectors using Python
VECTOR1=$(python3 -c "import random; print([round(random.uniform(-1, 1), 4) for _ in range(128)])")
VECTOR2=$(python3 -c "import random; print([round(random.uniform(-1, 1), 4) for _ in range(128)])")
VECTOR3=$(python3 -c "import random; print([round(random.uniform(-1, 1), 4) for _ in range(128)])")

curl -s -X POST "$MILVUS_URL/entities/insert" \
  -H "Content-Type: application/json" \
  -d "{
    \"collectionName\": \"embeddings\",
    \"data\": [
      {\"id\": \"doc1\", \"vector\": $VECTOR1, \"content\": \"Machine learning basics\"},
      {\"id\": \"doc2\", \"vector\": $VECTOR2, \"content\": \"Deep neural networks\"},
      {\"id\": \"doc3\", \"vector\": $VECTOR3, \"content\": \"Natural language processing\"}
    ]
  }" | jq .

# 6. Create index (for better search performance)
echo -e "\n--- Create Index ---"
curl -s -X POST "$MILVUS_URL/indexes/create" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "embeddings",
    "indexParams": {
      "metricType": "COSINE",
      "indexType": "IVF_FLAT",
      "params": {"nlist": 128}
    }
  }' | jq .

# 7. Load collection (required before search)
echo -e "\n--- Load Collection ---"
curl -s -X POST "$MILVUS_URL/collections/load" \
  -H "Content-Type: application/json" \
  -d '{"collectionName": "embeddings"}' | jq .

# 8. Search for similar vectors
echo -e "\n--- Vector Search ---"
QUERY_VECTOR=$(python3 -c "import random; print([round(random.uniform(-1, 1), 4) for _ in range(128)])")
curl -s -X POST "$MILVUS_URL/entities/search" \
  -H "Content-Type: application/json" \
  -d "{
    \"collectionName\": \"embeddings\",
    \"data\": [$QUERY_VECTOR],
    \"limit\": 3,
    \"outputFields\": [\"content\"]
  }" | jq .

# 9. Query by filter (scalar filtering)
echo -e "\n--- Query with Filter ---"
curl -s -X POST "$MILVUS_URL/entities/query" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "embeddings",
    "filter": "id in [\"doc1\", \"doc2\"]",
    "outputFields": ["id", "content"],
    "limit": 10
  }' | jq .

# 10. Get entity by ID
echo -e "\n--- Get Entity ---"
curl -s -X POST "$MILVUS_URL/entities/get" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "embeddings",
    "ids": ["doc1"],
    "outputFields": ["id", "content"]
  }' | jq .

# 11. Delete entities
echo -e "\n--- Delete Entity ---"
curl -s -X POST "$MILVUS_URL/entities/delete" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "embeddings",
    "filter": "id == \"doc3\""
  }' | jq .

# 12. Release collection (free memory)
echo -e "\n--- Release Collection ---"
curl -s -X POST "$MILVUS_URL/collections/release" \
  -H "Content-Type: application/json" \
  -d '{"collectionName": "embeddings"}' | jq .

# 13. Drop collection (cleanup)
echo -e "\n--- Drop Collection ---"
curl -s -X POST "$MILVUS_URL/collections/drop" \
  -H "Content-Type: application/json" \
  -d '{"collectionName": "embeddings"}' | jq .

echo -e "\n=== Milvus Examples Complete ==="

# --- Using zerg-api endpoints ---
echo -e "\n\n=== Zerg API Milvus Examples ==="
echo "Make sure zerg-api is running with MILVUS_URL=http://localhost:19121"

cat << 'EOF'

# Health check via zerg-api
curl -s "$API_URL/health" | jq .

# List collections via zerg-api
curl -s "$API_URL/collections" | jq .

# Create collection via zerg-api
curl -s -X POST "$API_URL/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "rag_embeddings",
    "dimension": 1536,
    "metricType": "COSINE"
  }' | jq .

# Get collection info via zerg-api
curl -s "$API_URL/collections/rag_embeddings" | jq .

# Insert vectors via zerg-api
curl -s -X POST "$API_URL/vectors" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "rag_embeddings",
    "data": [
      {"id": "doc1", "vector": [0.1, 0.2, ...], "content": "Document text"},
      {"id": "doc2", "vector": [0.2, 0.3, ...], "content": "Another document"}
    ]
  }' | jq .

# Search vectors via zerg-api
curl -s -X POST "$API_URL/search" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "rag_embeddings",
    "data": [[0.1, 0.2, ...]],
    "limit": 5,
    "outputFields": ["content"]
  }' | jq .

# Query by filter via zerg-api
curl -s -X POST "$API_URL/query" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "rag_embeddings",
    "filter": "id in [\"doc1\", \"doc2\"]",
    "outputFields": ["id", "content"],
    "limit": 10
  }' | jq .

# Delete entities via zerg-api
curl -s -X POST "$API_URL/delete" \
  -H "Content-Type: application/json" \
  -d '{
    "collectionName": "rag_embeddings",
    "ids": ["doc1"]
  }' | jq .

# Delete collection via zerg-api
curl -s -X DELETE "$API_URL/collections/rag_embeddings" | jq .
EOF
