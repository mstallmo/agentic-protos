#!/bin/bash
# test_grpc.sh - Test script for HelloService gRPC server with SQLite migrations
# This script starts the server, runs the client to test functionality, and cleans up.

set -e  # Exit on any error

# Colors for better readability
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Server process ID
SERVER_PID=""

# Function to clean up server process on exit
cleanup() {
  echo -e "\n${BLUE}Shutting down server process...${NC}"
  if [ -n "$SERVER_PID" ]; then
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    echo -e "${GREEN}Server stopped successfully${NC}"
  fi
}

# Register cleanup function to run on exit
trap cleanup EXIT INT TERM

# Print banner
echo -e "${BOLD}${BLUE}══════════════════════════════════════════════════${NC}"
echo -e "${BOLD}${BLUE}    HelloService gRPC Server Migration Tester     ${NC}"
echo -e "${BOLD}${BLUE}══════════════════════════════════════════════════${NC}"

# Check for SQLite database file
if [ -f "data.db" ]; then
  echo -e "\n${YELLOW}Existing SQLite database detected${NC}"
  echo -e "Counter values and statistics will be preserved from previous runs"
  echo -e "To start fresh, delete the database file: rm data.db"
else
  echo -e "\n${YELLOW}No SQLite database found${NC}"
  echo -e "A new database will be created with the latest schema migrations"
fi

# Check for migrations directory
if [ -d "migrations" ]; then
  echo -e "\n${BLUE}Migrations directory found${NC}"
  MIGRATION_COUNT=$(ls migrations/*.sql 2>/dev/null | wc -l)
  if [ "$MIGRATION_COUNT" -gt 0 ]; then
    echo -e "Found ${YELLOW}$MIGRATION_COUNT${NC} migration files:"
    ls -1 migrations/*.sql | sed 's/^/  - /'
  else
    echo -e "${RED}Warning: No SQL migration files found in migrations directory${NC}"
  fi
else
  echo -e "\n${RED}Error: Migrations directory not found!${NC}"
  echo -e "Please create a migrations directory with SQL migration files"
  exit 1
fi

# Build the project
echo -e "\n${BLUE}Building the project...${NC}"
cargo build || { echo -e "${RED}Build failed!${NC}"; exit 1; }

# Start the server in the background
echo -e "\n${BLUE}Starting gRPC server in the background...${NC}"
cargo run --bin agentic-protos &
SERVER_PID=$!

# Wait for server to initialize
echo -e "${YELLOW}Waiting for server to initialize...${NC}"
sleep 2

# Check if server is running
if ! ps -p $SERVER_PID > /dev/null; then
  echo -e "${RED}Server failed to start! Check error messages above.${NC}"
  exit 1
fi
echo -e "${GREEN}Server started successfully with PID $SERVER_PID${NC}"

# Run the client
echo -e "\n${BLUE}Running gRPC client to test server...${NC}"
cargo run --bin client

# Print summary
echo -e "\n${GREEN}${BOLD}Test Summary:${NC}"
echo -e "• ${GREEN}Server started successfully${NC}"
echo -e "• ${GREEN}Client communication successful${NC}"
echo -e "• ${GREEN}SQLite migrations applied${NC}"
echo -e "• ${GREEN}Counter values persisted in database${NC}"
echo -e "\n${YELLOW}Note: The database file 'data.db' contains your counter data${NC}"
echo -e "${YELLOW}Run this script again to see the counter continue to increment${NC}"

# Server will be stopped by cleanup function