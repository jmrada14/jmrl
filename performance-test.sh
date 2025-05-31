#!/bin/bash

# JMRL Blog Performance Testing Script
# Tests various endpoints for performance, load, and response times

set -euo pipefail

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
CONCURRENT_USERS="${CONCURRENT_USERS:-10}"
TEST_DURATION="${TEST_DURATION:-30s}"
OUTPUT_DIR="performance-results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if server is running
    if ! curl -s "$BASE_URL" > /dev/null; then
        log_error "Server is not running at $BASE_URL"
        exit 1
    fi
    
    # Check for required tools
    local tools=("curl" "ab" "jq")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_warning "$tool is not installed. Some tests may be skipped."
        fi
    done
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    
    log_success "Prerequisites check passed"
}

# Test individual endpoint response time
test_endpoint_response_time() {
    local endpoint="$1"
    local name="$2"
    
    log_info "Testing response time for $name..."
    
    local result
    result=$(curl -w "@-" -s -o /dev/null "$BASE_URL$endpoint" <<< '
    {
        "time_total": "%{time_total}",
        "time_namelookup": "%{time_namelookup}",
        "time_connect": "%{time_connect}",
        "time_appconnect": "%{time_appconnect}",
        "time_pretransfer": "%{time_pretransfer}",
        "time_starttransfer": "%{time_starttransfer}",
        "time_redirect": "%{time_redirect}",
        "size_download": "%{size_download}",
        "speed_download": "%{speed_download}",
        "http_code": "%{http_code}"
    }')
    
    local http_code time_total size_download
    http_code=$(echo "$result" | jq -r '.http_code')
    time_total=$(echo "$result" | jq -r '.time_total')
    size_download=$(echo "$result" | jq -r '.size_download')
    
    if [[ "$http_code" == "200" ]]; then
        log_success "$name: ${time_total}s (${size_download} bytes)"
        echo "$endpoint,$name,$time_total,$size_download,$http_code" >> "$OUTPUT_DIR/response_times_$TIMESTAMP.csv"
    else
        log_error "$name: HTTP $http_code"
    fi
}

# Apache Bench load testing
run_ab_test() {
    local endpoint="$1"
    local name="$2"
    
    if ! command -v ab &> /dev/null; then
        log_warning "Apache Bench (ab) not found, skipping load test for $name"
        return
    fi
    
    log_info "Running load test for $name (${CONCURRENT_USERS} concurrent users, ${TEST_DURATION})..."
    
    local requests_per_duration
    requests_per_duration=$((${TEST_DURATION%s} * 10)) # Approximate requests based on duration
    
    ab -n "$requests_per_duration" -c "$CONCURRENT_USERS" \
       -g "$OUTPUT_DIR/ab_${name// /_}_$TIMESTAMP.dat" \
       "$BASE_URL$endpoint" > "$OUTPUT_DIR/ab_${name// /_}_$TIMESTAMP.txt" 2>&1
    
    # Extract key metrics
    local rps mean_time
    if [[ -f "$OUTPUT_DIR/ab_${name// /_}_$TIMESTAMP.txt" ]]; then
        rps=$(grep "Requests per second" "$OUTPUT_DIR/ab_${name// /_}_$TIMESTAMP.txt" | awk '{print $4}')
        mean_time=$(grep "Time per request" "$OUTPUT_DIR/ab_${name// /_}_$TIMESTAMP.txt" | head -1 | awk '{print $4}')
        
        log_success "$name Load Test: ${rps} req/s, ${mean_time}ms avg"
    fi
}

# Memory and CPU usage monitoring
monitor_system_resources() {
    log_info "Monitoring system resources during tests..."
    
    # Find the jmrl process
    local pid
    pid=$(pgrep -f "jmrl" || echo "")
    
    if [[ -n "$pid" ]]; then
        # Monitor for 30 seconds
        for i in {1..30}; do
            local cpu_usage memory_usage
            if command -v ps &> /dev/null; then
                # macOS compatible ps command
                cpu_usage=$(ps -p "$pid" -o %cpu | tail -1 | tr -d ' ' || echo "0")
                memory_usage=$(ps -p "$pid" -o %mem | tail -1 | tr -d ' ' || echo "0")
                echo "$i,$cpu_usage,$memory_usage" >> "$OUTPUT_DIR/system_resources_$TIMESTAMP.csv"
            fi
            sleep 1
        done &
        
        log_info "Resource monitoring started (PID: $pid)"
    else
        log_warning "Could not find jmrl process for monitoring"
    fi
}

# Test concurrent connections
test_concurrent_connections() {
    log_info "Testing concurrent connections..."
    
    local pids=()
    local start_time
    start_time=$(date +%s)
    
    # Start concurrent requests
    for i in $(seq 1 "$CONCURRENT_USERS"); do
        {
            local response_time
            response_time=$(curl -w "%{time_total}" -s -o /dev/null "$BASE_URL/")
            echo "Thread $i: ${response_time}s"
        } &
        pids+=($!)
    done
    
    # Wait for all to complete
    for pid in "${pids[@]}"; do
        wait "$pid"
    done
    
    local end_time duration
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    
    log_success "Concurrent connections test completed in ${duration}s"
}

# Test different content types
test_content_types() {
    log_info "Testing different content types..."
    
    # Test endpoints (based on actual routes)
    local endpoints=(
        "/ Homepage"
        "/blog Blog Index"
        "/feed.xml RSS Feed"
        "/sitemap.xml Sitemap"
        "/robots.txt Robots"
        "/manifest.json Manifest"
    )
    
    # Initialize CSV
    echo "endpoint,name,response_time,size_bytes,http_code" > "$OUTPUT_DIR/response_times_$TIMESTAMP.csv"
    
    for endpoint_info in "${endpoints[@]}"; do
        local endpoint name
        endpoint=$(echo "$endpoint_info" | cut -d' ' -f1)
        name=$(echo "$endpoint_info" | cut -d' ' -f2-)
        
        test_endpoint_response_time "$endpoint" "$name"
    done
}

# Generate performance report
generate_report() {
    log_info "Generating performance report..."
    
    local report_file="$OUTPUT_DIR/performance_report_$TIMESTAMP.md"
    
    cat > "$report_file" << EOF
# JMRL Blog Performance Test Report

**Test Date:** $(date)
**Base URL:** $BASE_URL
**Concurrent Users:** $CONCURRENT_USERS
**Test Duration:** $TEST_DURATION

## Summary

This report contains performance test results for the JMRL blog application.

## Response Times

EOF
    
    if [[ -f "$OUTPUT_DIR/response_times_$TIMESTAMP.csv" ]]; then
        echo "| Endpoint | Name | Response Time (s) | Size (bytes) | Status |" >> "$report_file"
        echo "|----------|------|-------------------|--------------|---------|" >> "$report_file"
        
        while IFS=, read -r endpoint name time size status; do
            if [[ "$endpoint" != "endpoint" ]]; then # Skip header
                echo "| $endpoint | $name | $time | $size | $status |" >> "$report_file"
            fi
        done < "$OUTPUT_DIR/response_times_$TIMESTAMP.csv"
    fi
    
    cat >> "$report_file" << EOF

## Load Test Results

Load test results can be found in the following files:
- Apache Bench results: \`ab_*_$TIMESTAMP.txt\`
- Raw data: \`ab_*_$TIMESTAMP.dat\`

## System Resources

System resource monitoring data can be found in:
- \`system_resources_$TIMESTAMP.csv\`

## Recommendations

1. **Response Times**: All endpoints should respond within 200ms for optimal user experience
2. **Load Capacity**: The application should handle at least 100 concurrent users
3. **Resource Usage**: Monitor CPU and memory usage during peak loads
4. **Caching**: Implement caching for static content and frequently accessed pages

## Files Generated

EOF
    
    for file in "$OUTPUT_DIR"/*"$TIMESTAMP"*; do
        if [[ -f "$file" ]]; then
            echo "- $(basename "$file")" >> "$report_file"
        fi
    done
    
    log_success "Performance report generated: $report_file"
}

# Main execution
main() {
    echo "JMRL Blog Performance Testing"
    echo "============================="
    echo
    
    check_prerequisites
    monitor_system_resources
    test_content_types
    test_concurrent_connections
    
    # Run load tests for key endpoints
    run_ab_test "/" "Homepage"
    run_ab_test "/blog" "Blog Index"
    
    generate_report
    
    log_success "Performance testing completed! Results saved in $OUTPUT_DIR/"
}

# Run if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
