#!/bin/bash
#
# Berg10.Agent - Project Setup & Run Script
# Creates virtual environment, installs dependencies, and runs berg10_agent
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Project root directory
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Berg10.Agent - Project Setup${NC}"
echo -e "${YELLOW}========================================${NC}"
echo ""

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo -e "${GREEN}Python version:${NC} $PYTHON_VERSION"

# Virtual environment directory
VENV_DIR="$PROJECT_DIR/.venv"
VENV_PYTHON="$VENV_DIR/bin/python"

# Check if virtual environment exists
if [ -d "$VENV_DIR" ]; then
    echo -e "${GREEN}Virtual environment found:${NC} $VENV_DIR"
else
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    python3 -m venv "$VENV_DIR"
    echo -e "${GREEN}Virtual environment created:${NC} $VENV_DIR"
fi

# Activate virtual environment
echo -e "${GREEN}Activating virtual environment...${NC}"
source "$VENV_DIR/bin/activate"

# Check if dependencies need reinstallation (timestamp-based)
INSTALL_MARKER="$VENV_DIR/.install_marker"
PYPROJECT="$PROJECT_DIR/pyproject.toml"

REINSTALL=false
if [ ! -f "$INSTALL_MARKER" ]; then
    REINSTALL=true
    echo -e "${YELLOW}No install marker found, will install dependencies...${NC}"
elif [ "$PYPROJECT" -nt "$INSTALL_MARKER" ]; then
    REINSTALL=true
    echo -e "${YELLOW}pyproject.toml has changed, will reinstall dependencies...${NC}"
fi

if [ "$REINSTALL" = true ]; then
    # Upgrade pip (only if needed)
    echo -e "${YELLOW}Upgrading pip...${NC}"
    pip install --upgrade pip --quiet

    # Install dependencies
    echo -e "${YELLOW}Installing dependencies from pyproject.toml...${NC}"
    pip install -e . --quiet
    echo -e "${GREEN}Dependencies installed successfully${NC}"

    # Create install marker
    touch "$INSTALL_MARKER"
    echo -e "${GREEN}Install marker updated: $INSTALL_MARKER${NC}"

    # Verify installation
    echo -e "${YELLOW}Verifying installation...${NC}"
    python3 -c "import pocketflow; import litellm; import fastapi; print('Dependencies verified.')"
else
    echo -e "${GREEN}Dependencies already up-to-date (fast path)${NC}"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Running berg10_agent...${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Run berg10_agent as a module
python3 -m berg10_agent

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Execution complete!${NC}"
echo -e "${GREEN}========================================${NC}"

# Deactivate virtual environment
deactivate
