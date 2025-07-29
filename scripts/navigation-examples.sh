#!/bin/bash

# Navigation examples script
# This script demonstrates how to use the navigation features

echo "🚀 Ambient Light Control - Navigation Examples"
echo "=============================================="

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_example() {
    echo -e "${GREEN}[EXAMPLE]${NC} $1"
}

print_note() {
    echo -e "${YELLOW}[NOTE]${NC} $1"
}

echo ""
print_info "Command Line Navigation Examples"
echo "================================"

print_example "Open info page:"
echo "  ./Ambient\\ Light\\ Control.app/Contents/MacOS/Ambient\\ Light\\ Control --page info"

print_example "Open LED configuration page:"
echo "  ./Ambient\\ Light\\ Control.app/Contents/MacOS/Ambient\\ Light\\ Control --page led-strips-configuration"

print_example "Open white balance page:"
echo "  ./Ambient\\ Light\\ Control.app/Contents/MacOS/Ambient\\ Light\\ Control --page white-balance"

print_example "Open LED test page:"
echo "  ./Ambient\\ Light\\ Control.app/Contents/MacOS/Ambient\\ Light\\ Control --page led-strip-test"


print_example "Open settings page:"
echo "  ./Ambient\\ Light\\ Control.app/Contents/MacOS/Ambient\\ Light\\ Control --page settings"

print_example "Open LED config for specific display:"
echo "  ./Ambient\\ Light\\ Control.app/Contents/MacOS/Ambient\\ Light\\ Control --page led-strips-configuration --display 3"

echo ""
print_info "URL Scheme Navigation Examples"
echo "=============================="

print_example "Open info page via URL scheme:"
echo "  open 'ambient-light://navigate/info'"

print_example "Open LED configuration page via URL scheme:"
echo "  open 'ambient-light://navigate/led-strips-configuration'"

print_example "Open white balance page via URL scheme:"
echo "  open 'ambient-light://navigate/white-balance'"

print_example "Open LED test page via URL scheme:"
echo "  open 'ambient-light://navigate/led-strip-test'"



print_example "Open settings page via URL scheme:"
echo "  open 'ambient-light://navigate/settings'"

print_example "Open LED config for specific display via URL scheme:"
echo "  open 'ambient-light://navigate/led-strips-configuration/display/3'"

echo ""
print_info "Development Mode Examples"
echo "========================="

print_example "Start dev server with specific page:"
echo "  pnpm tauri dev -- --page settings"

print_example "Test URL scheme while dev server is running:"
echo "  open 'ambient-light://navigate/led-strip-test'"

echo ""
print_info "Frontend Code Examples"
echo "======================"

print_example "Using NavigationService in TypeScript:"
cat << 'EOF'
  import { NavigationService, navigateToPage } from '../services/navigation-service';
  
  // Navigate using static methods
  await NavigationService.navigateToInfo();
  await NavigationService.navigateToSettings();
  
  // Navigate using convenience function
  await navigateToPage('led-strip-test');
  
  // Check if page is valid
  if (NavigationService.isValidPage('custom-page')) {
    await navigateToPage('custom-page');
  }
EOF

print_example "Using URL Scheme helper:"
cat << 'EOF'
  import { AmbientLightUrlScheme } from '../services/navigation-service';
  
  // Create URL
  const url = AmbientLightUrlScheme.createNavigationUrl('white-balance');
  console.log(url); // ambient-light://navigate/white-balance
  
  // Open page via URL scheme
  await AmbientLightUrlScheme.openPageViaUrlScheme('settings');
  
  // Get all available URLs
  const urls = AmbientLightUrlScheme.getAllNavigationUrls();
EOF

echo ""
print_info "Testing and Automation"
echo "======================"

print_example "Run automated tests:"
echo "  ./scripts/test-navigation.sh"

print_example "Quick test specific page:"
echo "  ./Ambient\\ Light\\ Control.app/Contents/MacOS/Ambient\\ Light\\ Control --page info &"
echo "  sleep 2"
echo "  pkill -f 'Ambient Light Control'"

echo ""
print_note "Make sure to build the app first: pnpm tauri build"
print_note "URL scheme registration happens during app installation"
print_note "Some features may require user permission on first use"

echo ""
print_info "Available pages: info, led-strips-configuration, white-balance, led-strip-test, led-data-sender-test, settings"
