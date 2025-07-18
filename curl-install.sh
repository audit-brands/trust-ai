#!/bin/bash
# Trust AI Beta - One-line curl installer
# Usage: curl -sSL https://raw.githubusercontent.com/audit-brands/trust-ai/main/curl-install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Trust AI Beta - Source Installation${NC}"
echo -e "${YELLOW}This will download and build Trust AI from source${NC}"
echo ""

# Download and execute the full installer
curl -sSL https://raw.githubusercontent.com/audit-brands/trust-ai/main/install-beta.sh | bash