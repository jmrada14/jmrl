#!/bin/bash

# JMRL Blog Production Monitoring Script
# Monitors application health, performance, and logs

set -euo pipefail

# Configuration
SERVICE_NAME="${SERVICE_NAME:-jmrl}"
BASE_URL="${BASE_URL:-http://localhost:3000}"
CHECK_INTERVAL="${CHECK_INTERVAL:-30}"
LOG_FILE="${LOG_FILE:-/tmp/jmrl-monitor.log}"
ALERT_EMAIL="${ALERT_EMAIL:-}"
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"

# Thresholds
CPU_THRESHOLD="${CPU_THRESHOLD:-80}"
MEMORY_THRESHOLD="${MEMORY_THRESHOLD:-80}"
RESPONSE_TIME_THRESHOLD="${RESPONSE_TIME_THRESHOLD:-2.0}"
ERROR_RATE_THRESHOLD="${ERROR_RATE_THRESHOLD:-5}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $1"
    echo -e "${BLUE}$msg${NC}"
    echo "$msg" >> "$LOG_FILE"
}

log_success() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [SUCCESS] $1"
    echo -e "${GREEN}$msg${NC}"
    echo "$msg" >> "$LOG_FILE"
}

log_warning() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [WARNING] $1"
    echo -e "${YELLOW}$msg${NC}"
    echo "$msg" >> "$LOG_FILE"
}

log_error() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $1"
    echo -e "${RED}$msg${NC}"
    echo "$msg" >> "$LOG_FILE"
    send_alert "ERROR" "$1"
}

# Send alerts
send_alert() {
    local level="$1"
    local message="$2"
    
    # Email alert
    if [[ -n "$ALERT_EMAIL" ]] && command -v mail &> /dev/null; then
        echo "JMRL Blog Alert: $level - $message" | mail -s "JMRL Blog Alert" "$ALERT_EMAIL"
    fi
    
    # Slack alert
    if [[ -n "$SLACK_WEBHOOK" ]] && command -v curl &> /dev/null; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"🚨 JMRL Blog Alert: $level - $message\"}" \
            "$SLACK_WEBHOOK" > /dev/null 2>&1 || true
    fi
}

# Check if service is running
check_service_status() {
    local pid
    pid=$(pgrep -f "$SERVICE_NAME" || echo "")
    
    if [[ -n "$pid" ]]; then
        log_success "Service is running (PID: $pid)"
        return 0
    else
        log_error "Service is not running"
        return 1
    fi
}

# Check HTTP health
check_http_health() {
    local response_time http_code
    
    # Test main endpoint
    local curl_output
    curl_output=$(curl -w "%{http_code}:%{time_total}" -s -o /dev/null "$BASE_URL" 2>/dev/null || echo "000:999")
    
    http_code=$(echo "$curl_output" | cut -d':' -f1)
    response_time=$(echo "$curl_output" | cut -d':' -f2)
    
    if [[ "$http_code" == "200" ]]; then
        if (( $(echo "$response_time > $RESPONSE_TIME_THRESHOLD" | bc -l) )); then
            log_warning "HTTP OK but slow response: ${response_time}s (threshold: ${RESPONSE_TIME_THRESHOLD}s)"
        else
            log_success "HTTP health OK: ${response_time}s"
        fi
        return 0
    else
        log_error "HTTP health check failed: HTTP $http_code"
        return 1
    fi
}

# Monitor system resources
check_system_resources() {
    local pid
    pid=$(pgrep -f "$SERVICE_NAME" || echo "")
    
    if [[ -z "$pid" ]]; then
        return 1
    fi
    
    # Get CPU and memory usage (macOS compatible)
    local cpu_usage memory_usage
    if command -v ps &> /dev/null; then
        cpu_usage=$(ps -p "$pid" -o %cpu | tail -1 | tr -d ' ' | cut -d'.' -f1 || echo "0")
        memory_usage=$(ps -p "$pid" -o %mem | tail -1 | tr -d ' ' | cut -d'.' -f1 || echo "0")
    else
        cpu_usage=0
        memory_usage=0
    fi
    
    # Check thresholds
    if [[ "$cpu_usage" -gt "$CPU_THRESHOLD" ]]; then
        log_warning "High CPU usage: ${cpu_usage}% (threshold: ${CPU_THRESHOLD}%)"
    fi
    
    if [[ "$memory_usage" -gt "$MEMORY_THRESHOLD" ]]; then
        log_warning "High memory usage: ${memory_usage}% (threshold: ${MEMORY_THRESHOLD}%)"
    fi
    
    if [[ "$cpu_usage" -le "$CPU_THRESHOLD" ]] && [[ "$memory_usage" -le "$MEMORY_THRESHOLD" ]]; then
        log_success "System resources OK: CPU ${cpu_usage}%, Memory ${memory_usage}%"
    fi
}

# Check disk space
check_disk_space() {
    local disk_usage
    disk_usage=$(df / | tail -1 | awk '{print $5}' | sed 's/%//')
    
    if [[ "$disk_usage" -gt 80 ]]; then
        log_warning "High disk usage: ${disk_usage}%"
    else
        log_success "Disk space OK: ${disk_usage}%"
    fi
}

# Check log errors
check_log_errors() {
    # This is a placeholder - in production, you'd check actual application logs
    # For now, we'll simulate checking recent entries in our monitor log
    
    local recent_errors
    recent_errors=$(grep -c "ERROR" "$LOG_FILE" 2>/dev/null || echo "0")
    
    if [[ "$recent_errors" -gt 0 ]]; then
        log_info "Found $recent_errors recent errors in logs"
    else
        log_success "No recent errors in logs"
    fi
}

# Generate health report
generate_health_report() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    echo "========================================"
    echo "JMRL Blog Health Report - $timestamp"
    echo "========================================"
    echo
    
    # Service status
    if check_service_status > /dev/null 2>&1; then
        echo "✅ Service Status: Running"
    else
        echo "❌ Service Status: Not Running"
    fi
    
    # HTTP health
    if check_http_health > /dev/null 2>&1; then
        echo "✅ HTTP Health: OK"
    else
        echo "❌ HTTP Health: Failed"
    fi
    
    # System resources
    local pid
    pid=$(pgrep -f "$SERVICE_NAME" || echo "")
    if [[ -n "$pid" ]]; then
        local cpu_usage memory_usage
        cpu_usage=$(ps -p "$pid" -o %cpu | tail -1 | tr -d ' ' || echo "0")
        memory_usage=$(ps -p "$pid" -o %mem | tail -1 | tr -d ' ' || echo "0")
        echo "📊 CPU Usage: ${cpu_usage}%"
        echo "📊 Memory Usage: ${memory_usage}%"
    fi
    
    # Disk space
    local disk_usage
    disk_usage=$(df / | tail -1 | awk '{print $5}')
    echo "💾 Disk Usage: $disk_usage"
    
    echo
    echo "Last 5 log entries:"
    tail -5 "$LOG_FILE" 2>/dev/null || echo "No log entries found"
    echo
}

# Main monitoring loop
monitor_loop() {
    log_info "Starting JMRL Blog monitoring (interval: ${CHECK_INTERVAL}s)"
    
    while true; do
        echo
        log_info "Running health checks..."
        
        check_service_status
        check_http_health
        check_system_resources
        check_disk_space
        check_log_errors
        
        log_info "Health check cycle completed, sleeping for ${CHECK_INTERVAL}s"
        sleep "$CHECK_INTERVAL"
    done
}

# Handle signals
cleanup() {
    log_info "Monitoring stopped"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Main execution
case "${1:-monitor}" in
    "monitor")
        monitor_loop
        ;;
    "status")
        generate_health_report
        ;;
    "check")
        log_info "Running single health check..."
        check_service_status
        check_http_health
        check_system_resources
        check_disk_space
        ;;
    "help")
        echo "JMRL Blog Monitoring Script"
        echo
        echo "Usage: $0 [command]"
        echo
        echo "Commands:"
        echo "  monitor    Start continuous monitoring (default)"
        echo "  status     Show current health status"
        echo "  check      Run single health check"
        echo "  help       Show this help"
        echo
        echo "Environment Variables:"
        echo "  SERVICE_NAME              Service name to monitor (default: jmrl)"
        echo "  BASE_URL                 Base URL to check (default: http://localhost:3000)"
        echo "  CHECK_INTERVAL           Check interval in seconds (default: 30)"
        echo "  LOG_FILE                 Log file path (default: /tmp/jmrl-monitor.log)"
        echo "  ALERT_EMAIL             Email for alerts"
        echo "  SLACK_WEBHOOK           Slack webhook URL for alerts"
        echo "  CPU_THRESHOLD           CPU usage alert threshold % (default: 80)"
        echo "  MEMORY_THRESHOLD        Memory usage alert threshold % (default: 80)"
        echo "  RESPONSE_TIME_THRESHOLD Response time threshold in seconds (default: 2.0)"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac
