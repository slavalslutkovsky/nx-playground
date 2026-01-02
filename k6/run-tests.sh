#!/bin/bash
# k6 Load Test Runner
# Usage:
#   ./run-tests.sh local [test-file]     - Run tests locally with Docker
#   ./run-tests.sh cluster [test-file]   - Run tests in Kubernetes cluster
#   ./run-tests.sh apply                 - Apply k6 ConfigMap to cluster

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
K8S_DIR="$SCRIPT_DIR/k8s"
TESTS_DIR="$SCRIPT_DIR/tests"

# Default URLs for local testing
export ZERG_API_URL="${ZERG_API_URL:-http://localhost:3000}"
export ZERG_MONGO_API_URL="${ZERG_MONGO_API_URL:-http://localhost:3001}"
export PRODUCTS_API_URL="${PRODUCTS_API_URL:-http://localhost:3003}"

usage() {
    echo "k6 Load Test Runner"
    echo ""
    echo "Usage:"
    echo "  $0 local [test-file]     Run tests locally with Docker"
    echo "  $0 cluster [test-file]   Run tests in Kubernetes cluster"
    echo "  $0 apply                 Apply k6 ConfigMap to cluster"
    echo "  $0 status                Check running k6 jobs in cluster"
    echo "  $0 logs [job-name]       View logs of k6 job"
    echo "  $0 clean                 Clean up completed k6 jobs"
    echo ""
    echo "Test files:"
    echo "  zerg-api          Test Zerg API"
    echo "  zerg-mongo-api    Test Zerg MongoDB API"
    echo "  products-api      Test Products API"
    echo "  all-apis          Test all APIs"
    echo ""
    echo "Examples:"
    echo "  $0 local zerg-api"
    echo "  $0 cluster all-apis"
    echo "  $0 apply && $0 cluster zerg-api"
}

run_local() {
    local test_file="${1:-zerg-api}"
    local test_path="$TESTS_DIR/${test_file}.js"

    if [[ ! -f "$test_path" ]]; then
        echo "Error: Test file not found: $test_path"
        exit 1
    fi

    echo "Running k6 tests locally: $test_file"
    echo "Target URLs:"
    echo "  ZERG_API_URL: $ZERG_API_URL"
    echo "  ZERG_MONGO_API_URL: $ZERG_MONGO_API_URL"
    echo "  PRODUCTS_API_URL: $PRODUCTS_API_URL"
    echo ""

    # Check if k6 is installed locally
    if command -v k6 &> /dev/null; then
        k6 run \
            -e ZERG_API_URL="$ZERG_API_URL" \
            -e ZERG_MONGO_API_URL="$ZERG_MONGO_API_URL" \
            -e PRODUCTS_API_URL="$PRODUCTS_API_URL" \
            "$test_path"
    else
        # Use Docker
        docker run --rm -i \
            --network host \
            -v "$SCRIPT_DIR:/scripts:ro" \
            -e ZERG_API_URL="$ZERG_API_URL" \
            -e ZERG_MONGO_API_URL="$ZERG_MONGO_API_URL" \
            -e PRODUCTS_API_URL="$PRODUCTS_API_URL" \
            grafana/k6:latest run "/scripts/tests/${test_file}.js"
    fi
}

apply_configmap() {
    echo "Applying k6 ConfigMap to cluster..."
    kubectl apply -f "$K8S_DIR/k6-configmap.yaml"
    echo "ConfigMap applied successfully!"
}

run_cluster() {
    local test_file="${1:-zerg-api}"
    local job_name="k6-${test_file}-$(date +%s)"

    echo "Running k6 tests in cluster: $test_file"
    echo "Job name: $job_name"

    # Ensure ConfigMap exists
    if ! kubectl get configmap k6-tests &>/dev/null; then
        echo "ConfigMap not found. Applying..."
        apply_configmap
    fi

    # Create job from template
    cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: $job_name
  namespace: default
  labels:
    app: k6
    test: $test_file
spec:
  ttlSecondsAfterFinished: 3600
  backoffLimit: 0
  template:
    metadata:
      labels:
        app: k6
        test: $test_file
    spec:
      restartPolicy: Never
      containers:
        - name: k6
          image: grafana/k6:latest
          command: ["k6", "run", "/scripts/${test_file}.js"]
          env:
            - name: ZERG_API_URL
              value: "http://zerg-api.default.svc.cluster.local:3000"
            - name: ZERG_MONGO_API_URL
              value: "http://zerg-mongo-api.default.svc.cluster.local:3001"
            - name: PRODUCTS_API_URL
              value: "http://products-api.default.svc.cluster.local:3003"
          volumeMounts:
            - name: k6-scripts
              mountPath: /scripts
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
      volumes:
        - name: k6-scripts
          configMap:
            name: k6-tests
EOF

    echo ""
    echo "Job created! Monitor with:"
    echo "  kubectl logs -f job/$job_name"
    echo ""
    echo "Waiting for pod to start..."
    kubectl wait --for=condition=ready pod -l job-name=$job_name --timeout=60s 2>/dev/null || true

    echo "Streaming logs..."
    kubectl logs -f job/$job_name
}

check_status() {
    echo "k6 Jobs:"
    kubectl get jobs -l app=k6
    echo ""
    echo "k6 Pods:"
    kubectl get pods -l app=k6
}

view_logs() {
    local job_name="${1:-k6-load-test}"
    kubectl logs -f job/$job_name
}

clean_jobs() {
    echo "Cleaning up completed k6 jobs..."
    kubectl delete jobs -l app=k6 --field-selector status.successful=1 2>/dev/null || true
    kubectl delete jobs -l app=k6 --field-selector status.failed=1 2>/dev/null || true
    echo "Cleanup complete!"
}

case "${1:-}" in
    local)
        run_local "$2"
        ;;
    cluster)
        run_cluster "$2"
        ;;
    apply)
        apply_configmap
        ;;
    status)
        check_status
        ;;
    logs)
        view_logs "$2"
        ;;
    clean)
        clean_jobs
        ;;
    *)
        usage
        ;;
esac
