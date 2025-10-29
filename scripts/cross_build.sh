#!/bin/bash

# Cross-platform build script for flatten-rust
# Builds the project for multiple target platforms

set -e

echo "🌍 Starting cross-platform build for flatten-rust..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if rustup is installed
check_rustup() {
    if ! command -v rustup &> /dev/null; then
        print_error "rustup is not installed. Please install it from https://rustup.rs/"
        exit 1
    fi
}

# Install target if not already installed
install_target() {
    local target=$1
    print_status "Installing target: $target"
    if rustup target list --installed | grep -q "$target"; then
        print_status "Target $target already installed"
    else
        rustup target add "$target"
        print_success "Target $target installed"
    fi
}

# Build for specific target
build_target() {
    local target=$1
    local output_name=$2
    
    print_status "Building for target: $target"
    
    if cargo build --release --target "$target"; then
        # Create output directory
        mkdir -p "dist/$output_name"
        
        # Copy binary
        if [ "$target" = "x86_64-pc-windows-gnu" ]; then
            cp "target/$target/release/flatten-rust.exe" "dist/$output_name/"
        else
            cp "target/$target/release/flatten-rust" "dist/$output_name/"
        fi
        
        # Copy documentation
        cp README.md "dist/$output_name/"
        cp -r examples "dist/$output_name/"
        if [ -f LICENSE ]; then
            cp LICENSE "dist/$output_name/"
        fi
        
        print_success "Build completed for $target"
        return 0
    else
        print_error "Build failed for $target"
        return 1
    fi
}

# Create archive for distribution
create_archive() {
    local output_name=$1
    
    print_status "Creating archive for $output_name"
    
    cd dist
    if command -v tar >/dev/null 2>&1; then
        if [[ "$output_name" == *"windows"* ]]; then
            if command -v zip >/dev/null 2>&1; then
                zip -r "$output_name.zip" "$output_name"
                print_success "Created: $output_name.zip"
            else
                print_warning "zip not found, skipping archive for $output_name"
            fi
        else
            tar -czf "$output_name.tar.gz" "$output_name"
            print_success "Created: $output_name.tar.gz"
        fi
    else
        print_warning "tar not found, skipping archive creation"
    fi
    cd ..
}

# Main build function
main() {
    check_rustup
    
    # Get version from Cargo.toml
    VERSION=$(cargo metadata --no-deps --format-version 1 | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)
    print_status "Building version: $VERSION"
    
    # Clean previous builds
    print_status "Cleaning previous builds..."
    cargo clean
    rm -rf dist/*
    
    # Define targets and their output names
    declare -A TARGETS=(
        ["x86_64-unknown-linux-gnu"]="flatten-rust-$VERSION-linux-x86_64"
        ["x86_64-pc-windows-gnu"]="flatten-rust-$VERSION-windows-x86_64"
        ["x86_64-apple-darwin"]="flatten-rust-$VERSION-macos-x86_64"
        ["aarch64-apple-darwin"]="flatten-rust-$VERSION-macos-arm64"
    )
    
    # Optional: Add more targets
    # ["armv7-unknown-linux-gnueabihf"]="flatten-rust-$VERSION-linux-armv7"
    # ["i686-pc-windows-gnu"]="flatten-rust-$VERSION-windows-x86"
    
    # Build for each target
    local failed_targets=()
    local successful_targets=()
    
    for target in "${!TARGETS[@]}"; do
        output_name="${TARGETS[$target]}"
        
        print_status "=========================================="
        print_status "Building for $target"
        print_status "Output: $output_name"
        print_status "=========================================="
        
        if install_target "$target"; then
            if build_target "$target" "$output_name"; then
                create_archive "$output_name"
                successful_targets+=("$target")
            else
                failed_targets+=("$target")
            fi
        else
            failed_targets+=("$target")
        fi
        
        echo ""
    done
    
    # Summary
    print_status "=========================================="
    print_status "BUILD SUMMARY"
    print_status "=========================================="
    
    if [ ${#successful_targets[@]} -gt 0 ]; then
        print_success "Successfully built for:"
        for target in "${successful_targets[@]}"; do
            echo "  ✓ $target"
        done
    fi
    
    if [ ${#failed_targets[@]} -gt 0 ]; then
        print_error "Failed to build for:"
        for target in "${failed_targets[@]}"; do
            echo "  ✗ $target"
        done
    fi
    
    # Show distribution files
    if [ -d "dist" ] && [ "$(ls -A dist)" ]; then
        print_status "Distribution files created:"
        ls -la dist/
    fi
    
    print_status "Cross-platform build completed!"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            shift
            if [ -n "$1" ]; then
                CUSTOM_TARGET="$1"
                shift
            else
                print_error "--target requires a value"
                exit 1
            fi
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --target <target>    Build only for specific target"
            echo "  --help               Show this help message"
            echo ""
            echo "Available targets:"
            echo "  x86_64-unknown-linux-gnu    Linux x86_64"
            echo "  x86_64-pc-windows-gnu      Windows x86_64"
            echo "  x86_64-apple-darwin        macOS x86_64"
            echo "  aarch64-apple-darwin       macOS ARM64"
            
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# If custom target is specified, build only for that target
if [ -n "$CUSTOM_TARGET" ]; then
    print_status "Building only for target: $CUSTOM_TARGET"
    
    VERSION=$(cargo metadata --no-deps --format-version 1 | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)
    output_name="flatten-rust-$VERSION-$CUSTOM_TARGET"
    
    if install_target "$CUSTOM_TARGET"; then
        if build_target "$CUSTOM_TARGET" "$output_name"; then
            create_archive "$output_name"
            print_success "Build completed for $CUSTOM_TARGET"
        else
            print_error "Build failed for $CUSTOM_TARGET"
            exit 1
        fi
    else
        print_error "Failed to install target $CUSTOM_TARGET"
        exit 1
    fi
else
    # Build for all targets
    main
fi