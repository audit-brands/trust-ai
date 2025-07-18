#!/bin/bash

# Code Generation Testing Script for trust-ai CLI
# This script sets up the environment and runs code generation tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check if we're in the correct directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Check if tests directory exists
    if [ ! -d "tests" ]; then
        print_error "Tests directory not found. Please ensure test files are present."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to run unit tests (always available)
run_unit_tests() {
    print_status "Running unit tests..."
    
    if cargo test --test test_simple_code_generation unit_tests --no-fail-fast; then
        print_success "Unit tests passed"
        return 0
    else
        print_error "Unit tests failed"
        return 1
    fi
}

# Function to run integration tests (require API access)
run_integration_tests() {
    print_status "Running integration tests..."
    
    # Set environment variable to enable tests
    export RUN_CODE_GENERATION_TESTS=1
    
    # Test basic code generation
    print_status "Testing basic file creation..."
    if cargo test --test test_simple_code_generation conditional_tests::test_basic_file_creation_conditional --no-fail-fast; then
        print_success "Basic file creation test passed"
    else
        print_warning "Basic file creation test failed"
        return 1
    fi
    
    # Test file modification
    print_status "Testing file modification..."
    if cargo test --test test_simple_code_generation conditional_tests::test_file_modification_conditional --no-fail-fast; then
        print_success "File modification test passed"
    else
        print_warning "File modification test failed"
        return 1
    fi
    
    # Test shell command execution
    print_status "Testing shell command execution..."
    if cargo test --test test_simple_code_generation conditional_tests::test_shell_command_execution_conditional --no-fail-fast; then
        print_success "Shell command execution test passed"
    else
        print_warning "Shell command execution test failed"
        return 1
    fi
    
    print_success "All integration tests passed"
    return 0
}

# Function to run comprehensive tests (if available)
run_comprehensive_tests() {
    print_status "Running comprehensive tests..."
    
    export RUN_CODE_GENERATION_TESTS=1
    
    if cargo test --test test_code_generation_tools --no-fail-fast; then
        print_success "Comprehensive tests passed"
        return 0
    else
        print_warning "Comprehensive tests failed or skipped"
        return 1
    fi
}

# Function to clean up test artifacts
cleanup() {
    print_status "Cleaning up test artifacts..."
    
    # Remove test directories
    if [ -d "target/test_basic_file" ]; then
        rm -rf target/test_basic_file
        print_status "Removed target/test_basic_file"
    fi
    
    if [ -d "target/test_file_mod" ]; then
        rm -rf target/test_file_mod
        print_status "Removed target/test_file_mod"
    fi
    
    if [ -d "target/test_shell_cmd" ]; then
        rm -rf target/test_shell_cmd
        print_status "Removed target/test_shell_cmd"
    fi
    
    # Remove any other test directories
    find target -name "test_*" -type d -exec rm -rf {} + 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --unit-only     Run only unit tests (no API required)"
    echo "  --integration   Run integration tests (requires API access)"
    echo "  --comprehensive Run all available tests including comprehensive suite"
    echo "  --cleanup       Clean up test artifacts and exit"
    echo "  --help          Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  RUN_CODE_GENERATION_TESTS=1  Enable integration tests"
    echo ""
    echo "Examples:"
    echo "  $0                           # Run unit tests only"
    echo "  $0 --integration            # Run integration tests"
    echo "  $0 --comprehensive          # Run all tests"
}

# Main execution
main() {
    local run_unit=true
    local run_integration=false
    local run_comprehensive=false
    local cleanup_only=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit-only)
                run_unit=true
                run_integration=false
                run_comprehensive=false
                shift
                ;;
            --integration)
                run_unit=true
                run_integration=true
                run_comprehensive=false
                shift
                ;;
            --comprehensive)
                run_unit=true
                run_integration=true
                run_comprehensive=true
                shift
                ;;
            --cleanup)
                cleanup_only=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Handle cleanup-only mode
    if [ "$cleanup_only" = true ]; then
        cleanup
        exit 0
    fi
    
    print_status "Starting trust-ai code generation tests..."
    
    # Check prerequisites
    check_prerequisites
    
    local test_failures=0
    
    # Run unit tests
    if [ "$run_unit" = true ]; then
        if ! run_unit_tests; then
            ((test_failures++))
        fi
    fi
    
    # Run integration tests
    if [ "$run_integration" = true ]; then
        print_status "Integration tests require API access and may take longer..."
        if ! run_integration_tests; then
            ((test_failures++))
        fi
    fi
    
    # Run comprehensive tests
    if [ "$run_comprehensive" = true ]; then
        print_status "Comprehensive tests require full API access..."
        if ! run_comprehensive_tests; then
            ((test_failures++))
        fi
    fi
    
    # Cleanup
    cleanup
    
    # Report results
    echo ""
    print_status "Test execution completed"
    
    if [ $test_failures -eq 0 ]; then
        print_success "All tests passed successfully!"
        exit 0
    else
        print_error "$test_failures test suite(s) failed"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"