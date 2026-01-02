#!/bin/bash
# =============================================================================
# Dapr Development Helper Script
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DAPR_COMPONENTS_PATH="$PROJECT_ROOT/.dapr/components"
DAPR_CONFIG_PATH="$PROJECT_ROOT/.dapr/config.yaml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# =============================================================================
# Commands
# =============================================================================

cmd_init() {
    log_info "Initializing Dapr..."

    # Check if Dapr CLI is installed
    if ! command -v dapr &> /dev/null; then
        log_error "Dapr CLI not found. Installing..."
        curl -fsSL https://raw.githubusercontent.com/dapr/cli/master/install/install.sh | bash
    fi

    # Initialize Dapr
    dapr init

    log_success "Dapr initialized successfully!"
    dapr --version
}

cmd_init_slim() {
    log_info "Initializing Dapr in slim mode (no Docker)..."
    dapr init --slim
    log_success "Dapr slim mode initialized!"
}

cmd_start_infra() {
    log_info "Starting infrastructure services..."
    docker-compose -f "$PROJECT_ROOT/docker-compose.dapr.yml" up -d \
        redis postgres mongodb zipkin otel-collector placement

    log_info "Waiting for services to be healthy..."
    sleep 5

    log_success "Infrastructure services started!"
    docker-compose -f "$PROJECT_ROOT/docker-compose.dapr.yml" ps
}

cmd_run_app() {
    local app_name="$1"
    local app_port="$2"
    local dapr_http_port="$3"
    local app_command="$4"

    if [[ -z "$app_name" ]]; then
        log_error "Usage: $0 run <app-name> <app-port> <dapr-http-port> <command>"
        exit 1
    fi

    app_port="${app_port:-3000}"
    dapr_http_port="${dapr_http_port:-3500}"

    log_info "Starting $app_name with Dapr sidecar..."

    dapr run \
        --app-id "$app_name" \
        --app-port "$app_port" \
        --dapr-http-port "$dapr_http_port" \
        --dapr-grpc-port "$((dapr_http_port + 50000))" \
        --resources-path "$DAPR_COMPONENTS_PATH" \
        --config "$DAPR_CONFIG_PATH" \
        --log-level info \
        -- $app_command
}

cmd_run_zerg_api() {
    log_info "Building and running zerg-api with Dapr..."

    cd "$PROJECT_ROOT"
    cargo build -p zerg-api --release

    cmd_run_app "zerg-api" 3000 3500 "./target/release/zerg-api"
}

cmd_run_zerg_mongo_api() {
    log_info "Building and running zerg-mongo-api with Dapr..."

    cd "$PROJECT_ROOT"
    cargo build -p zerg-mongo-api --release

    cmd_run_app "zerg-mongo-api" 3001 3501 "./target/release/zerg-mongo-api"
}

cmd_run_zerg_tasks() {
    log_info "Building and running zerg-tasks with Dapr..."

    cd "$PROJECT_ROOT"
    cargo build -p zerg-tasks --release

    cmd_run_app "zerg-tasks" 3002 3502 "./target/release/zerg-tasks"
}

cmd_stop() {
    log_info "Stopping all Dapr applications..."
    dapr stop --app-id zerg-api 2>/dev/null || true
    dapr stop --app-id zerg-mongo-api 2>/dev/null || true
    dapr stop --app-id zerg-tasks 2>/dev/null || true

    log_info "Stopping infrastructure..."
    docker-compose -f "$PROJECT_ROOT/docker-compose.dapr.yml" down

    log_success "All services stopped!"
}

cmd_status() {
    log_info "Dapr applications status:"
    dapr list

    echo ""
    log_info "Docker services status:"
    docker-compose -f "$PROJECT_ROOT/docker-compose.dapr.yml" ps
}

cmd_dashboard() {
    log_info "Opening Dapr dashboard..."
    dapr dashboard -p 8081
}

cmd_test_state() {
    local dapr_port="${1:-3500}"
    local key="${2:-test-key}"
    local value="${3:-test-value}"

    log_info "Testing state store on port $dapr_port..."

    # Save state
    curl -X POST "http://localhost:$dapr_port/v1.0/state/statestore" \
        -H "Content-Type: application/json" \
        -d "[{\"key\": \"$key\", \"value\": \"$value\"}]"

    echo ""
    log_info "Retrieving state..."
    curl -s "http://localhost:$dapr_port/v1.0/state/statestore/$key"
    echo ""

    log_success "State store test completed!"
}

cmd_test_pubsub() {
    local dapr_port="${1:-3500}"
    local topic="${2:-test-topic}"

    log_info "Testing pub/sub on port $dapr_port..."

    curl -X POST "http://localhost:$dapr_port/v1.0/publish/pubsub/$topic" \
        -H "Content-Type: application/json" \
        -d '{"message": "Hello from Dapr!", "timestamp": "'$(date -Iseconds)'"}'

    log_success "Message published to $topic!"
}

cmd_invoke() {
    local target_app="$1"
    local method="$2"
    local dapr_port="${3:-3500}"

    if [[ -z "$target_app" || -z "$method" ]]; then
        log_error "Usage: $0 invoke <target-app> <method> [dapr-port]"
        exit 1
    fi

    log_info "Invoking $target_app/$method via Dapr..."

    curl -s "http://localhost:$dapr_port/v1.0/invoke/$target_app/method/$method"
    echo ""
}

cmd_help() {
    echo "Dapr Development Helper Script"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  init              Initialize Dapr with Docker"
    echo "  init-slim         Initialize Dapr without Docker (slim mode)"
    echo "  start-infra       Start infrastructure services (Redis, Postgres, etc.)"
    echo "  run <app> ...     Run an app with Dapr sidecar"
    echo "  run-zerg-api      Build and run zerg-api with Dapr"
    echo "  run-zerg-mongo    Build and run zerg-mongo-api with Dapr"
    echo "  run-zerg-tasks    Build and run zerg-tasks with Dapr"
    echo "  stop              Stop all Dapr apps and infrastructure"
    echo "  status            Show status of all services"
    echo "  dashboard         Open Dapr dashboard"
    echo "  test-state        Test state store operations"
    echo "  test-pubsub       Test pub/sub messaging"
    echo "  invoke            Invoke a service method via Dapr"
    echo "  help              Show this help message"
}

# =============================================================================
# Main
# =============================================================================

case "${1:-help}" in
    init)           cmd_init ;;
    init-slim)      cmd_init_slim ;;
    start-infra)    cmd_start_infra ;;
    run)            shift; cmd_run_app "$@" ;;
    run-zerg-api)   cmd_run_zerg_api ;;
    run-zerg-mongo) cmd_run_zerg_mongo_api ;;
    run-zerg-tasks) cmd_run_zerg_tasks ;;
    stop)           cmd_stop ;;
    status)         cmd_status ;;
    dashboard)      cmd_dashboard ;;
    test-state)     shift; cmd_test_state "$@" ;;
    test-pubsub)    shift; cmd_test_pubsub "$@" ;;
    invoke)         shift; cmd_invoke "$@" ;;
    help|--help|-h) cmd_help ;;
    *)              log_error "Unknown command: $1"; cmd_help; exit 1 ;;
esac
