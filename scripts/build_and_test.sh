#!/bin/bash

# Build and test script for flatten-rust
# This script handles building, testing, and benchmarking the project

set -e  # Exit on any error

echo "🚀 Starting build and test process for flatten-rust..."

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

# Check if Rust is installed
check_rust() {
    print_status "Checking Rust installation..."
    
    # Try to activate cargo environment
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    rust_version=$(rustc --version)
    print_success "Rust found: $rust_version"
}

# Clean previous builds
clean_build() {
    print_status "Cleaning previous builds..."
    cargo clean
    print_success "Clean completed"
}

# Build the project
build_project() {
    print_status "Building project..."
    
    # Debug build
    print_status "Building debug version..."
    cargo build
    
    # Release build
    print_status "Building release version..."
    cargo build --release
    
    print_success "Build completed successfully"
}

# Run tests
run_tests() {
    print_status "Running unit tests..."
    cargo test --lib
    
    print_status "Running integration tests..."
    cargo test --test integration_tests
    
    print_success "All tests passed"
}

# Run clippy for linting
run_clippy() {
    print_status "Running Clippy linting..."
    cargo clippy -- -D warnings
    print_success "Clippy checks passed"
}

# Check code formatting
check_format() {
    print_status "Checking code formatting..."
    if cargo fmt --check; then
        print_success "Code formatting is correct"
    else
        print_warning "Code formatting issues found. Running cargo fmt..."
        cargo fmt
        print_success "Code formatted"
    fi
}

# Run benchmarks
run_benchmarks() {
    print_status "Running performance benchmarks..."
    cargo bench
    print_success "Benchmarks completed"
}

# Test the binary with sample data
test_binary() {
    print_status "Testing binary functionality..."
    
    # Create test directory structure
    TEST_DIR="/tmp/flatten_test_$$"
    mkdir -p "$TEST_DIR"/{src,tests,node_modules}
    
    # Create test files
    echo 'fn main() { println!("Hello from Rust!"); }' > "$TEST_DIR/src/main.rs"
    echo 'pub fn helper() { "helper function" }' > "$TEST_DIR/src/lib.rs"
    echo '#[test] fn test_helper() {}' > "$TEST_DIR/tests/integration.rs"
    echo 'console.log("hello");' > "$TEST_DIR/node_modules/index.js"
    echo '# Test Project' > "$TEST_DIR/README.md"
    
    # Run the binary
    OUTPUT_FILE="/tmp/flatten_output_$$"
    ./target/release/flatten-rust --folders "$TEST_DIR" --output "$OUTPUT_FILE" --skip-folders node_modules
    
    # Check output
    if [ -f "$OUTPUT_FILE" ] && grep -q "FOLDER STRUCTURE" "$OUTPUT_FILE"; then
        print_success "Binary test passed"
    else
        print_error "Binary test failed"
        cat "$OUTPUT_FILE" 2>/dev/null || true
        exit 1
    fi
    
    # Cleanup
    rm -rf "$TEST_DIR" "$OUTPUT_FILE"
}

# Create distribution package
create_package() {
    print_status "Creating distribution package..."
    
    VERSION=$(cargo metadata --no-deps --format-version 1 | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)
    PACKAGE_NAME="flatten-rust-$VERSION"
    
    mkdir -p "dist/$PACKAGE_NAME"
    
    # Copy binary
    cp target/release/flatten-rust "dist/$PACKAGE_NAME/"
    
    # Copy documentation
    cp README.md "dist/$PACKAGE_NAME/"
    cp -r examples "dist/$PACKAGE_NAME/"
    
    # Copy license if it exists
    if [ -f LICENSE ]; then
        cp LICENSE "dist/$PACKAGE_NAME/"
    fi
    
    # Create archive
    cd dist
    tar -czf "$PACKAGE_NAME.tar.gz" "$PACKAGE_NAME"
    cd ..
    
    print_success "Package created: dist/$PACKAGE_NAME.tar.gz"
}

# Main execution
main() {
    check_rust
    
    # Parse command line arguments
    SKIP_TESTS=false
    SKIP_BENCH=false
    SKIP_PACKAGE=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --skip-bench)
                SKIP_BENCH=true
                shift
                ;;
            --skip-package)
                SKIP_PACKAGE=true
                shift
                ;;
            --clean-only)
                clean_build
                exit 0
                ;;
            --help)
                echo "Usage: $0 [options]"
                echo "Options:"
                echo "  --skip-tests    Skip running tests"
                echo "  --skip-bench     Skip running benchmarks"
                echo "  --skip-package   Skip creating distribution package"
                echo "  --clean-only     Only clean the project"
                echo "  --help           Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    clean_build
    build_project
    check_format
    run_clippy
    
    if [ "$SKIP_TESTS" = false ]; then
        run_tests
        test_binary
    else
        print_warning "Skipping tests as requested"
    fi
    
    if [ "$SKIP_BENCH" = false ]; then
        run_benchmarks
    else
        print_warning "Skipping benchmarks as requested"
    fi
    
    if [ "$SKIP_PACKAGE" = false ]; then
        create_package
    else
        print_warning "Skipping package creation as requested"
    fi
    
    print_success "🎉 Build and test process completed successfully!"
    print_status "Binary location: ./target/release/flatten-rust"
    print_status "Usage: ./target/release/flatten-rust --folders <path> --output output.md"
}

# Run main function with all arguments
main "$@"