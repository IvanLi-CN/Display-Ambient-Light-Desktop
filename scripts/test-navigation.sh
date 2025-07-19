#!/bin/bash

# Test script for navigation functionality
# This script tests both command line arguments and URL scheme navigation

set -e

echo "🧪 Testing Ambient Light Control Navigation Features"
echo "=================================================="

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

# Check if app is built
APP_PATH=""
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - try different possible locations
    APP_PATH="./src-tauri/target/release/ambient-light-control"
    if [ ! -f "$APP_PATH" ]; then
        APP_PATH="./src-tauri/target/debug/ambient-light-control"
    fi
    if [ ! -f "$APP_PATH" ]; then
        APP_PATH="./src-tauri/target/debug/bundle/macos/Ambient Light Control.app/Contents/MacOS/ambient-light-control"
    fi
    if [ ! -f "$APP_PATH" ]; then
        APP_PATH="./src-tauri/target/release/bundle/macos/Ambient Light Control.app/Contents/MacOS/ambient-light-control"
    fi
else
    print_error "This script currently only supports macOS"
    exit 1
fi

if [ ! -f "$APP_PATH" ]; then
    print_error "App not found. Please build the app first with 'pnpm tauri build' or 'pnpm tauri dev'"
    exit 1
fi

print_status "Found app at: $APP_PATH"

# Test pages
PAGES=("info" "led-strips-configuration" "white-balance" "led-strip-test" "led-data-sender-test" "settings")

echo ""
print_status "Testing command line arguments..."
echo "=================================="

for page in "${PAGES[@]}"; do
    print_status "Testing --page $page"
    
    # Launch app with page argument (in background)
    "$APP_PATH" --page "$page" &
    APP_PID=$!
    
    # Wait longer for app to start and navigation to complete
    sleep 4

    # Kill the app
    kill $APP_PID 2>/dev/null || true
    wait $APP_PID 2>/dev/null || true
    
    print_success "Command line test for page '$page' completed"
    
    # Small delay between tests
    sleep 1
done

echo ""
print_status "Testing command line arguments with display ID..."
echo "================================================="

# Test display-specific pages
DISPLAY_IDS=("1" "2" "3")
for display_id in "${DISPLAY_IDS[@]}"; do
    print_status "Testing LED config for display $display_id"

    # Launch app with page and display arguments (in background)
    "$APP_PATH" --page led-strips-configuration --display "$display_id" &
    APP_PID=$!

    # Wait longer for app to start and navigation to complete
    sleep 4

    # Kill the app
    kill $APP_PID 2>/dev/null || true
    wait $APP_PID 2>/dev/null || true

    print_success "Display-specific test for display '$display_id' completed"

    # Small delay between tests
    sleep 1
done

echo ""
print_status "Testing URL scheme navigation..."
echo "================================"

# Test URL scheme (this will only work if the app is registered as the handler)
for page in "${PAGES[@]}"; do
    url="ambient-light://navigate/$page"
    print_status "Testing URL: $url"

    # Try to open the URL
    if command -v open >/dev/null 2>&1; then
        # macOS
        open "$url" 2>/dev/null || print_warning "URL scheme test failed for $url (app may not be registered)"
    else
        print_warning "Cannot test URL scheme on this platform"
    fi

    # Small delay between tests
    sleep 1
done

echo ""
print_status "Testing URL scheme with display ID..."
echo "====================================="

# Test display-specific URL schemes
for display_id in "${DISPLAY_IDS[@]}"; do
    url="ambient-light://navigate/led-strips-configuration/display/$display_id"
    print_status "Testing display URL: $url"

    # Try to open the URL
    if command -v open >/dev/null 2>&1; then
        # macOS
        open "$url" 2>/dev/null || print_warning "URL scheme test failed for $url (app may not be registered)"
    else
        print_warning "Cannot test URL scheme on this platform"
    fi

    # Small delay between tests
    sleep 1
done

echo ""
print_success "Navigation testing completed!"
echo ""
print_status "Manual testing instructions:"
echo "1. Build and install the app: pnpm tauri build"
echo "2. Test command line: './path/to/app --page info'"
echo "3. Test URL scheme: open 'ambient-light://navigate/settings'"
echo "4. Check that the app opens to the correct page"
echo ""
print_status "Available pages: ${PAGES[*]}"
echo ""
print_status "Available URL schemes:"
for page in "${PAGES[@]}"; do
    echo "  ambient-light://navigate/$page"
done
