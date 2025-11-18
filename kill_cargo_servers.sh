#!/bin/bash
# Script to kill all cargo/rust processes running on port 8080

PORT=8080

echo "🔍 Finding cargo/rust processes on port $PORT..."

# Find PIDs listening on port 8080 that are cargo/rust related
PIDS=$(lsof -ti:$PORT 2>/dev/null | xargs ps -p 2>/dev/null | grep -E "(cargo|rust|wavelength)" | awk '{print $1}' | sort -u)

if [ -z "$PIDS" ]; then
    # Alternative: find all processes on port 8080 and check if they're cargo-related
    ALL_PIDS=$(lsof -ti:$PORT 2>/dev/null)
    
    if [ -z "$ALL_PIDS" ]; then
        echo "✅ No processes found on port $PORT"
        exit 0
    fi
    
    echo "Found processes on port $PORT:"
    lsof -i:$PORT
    
    echo ""
    read -p "Kill all processes on port $PORT? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled."
        exit 0
    fi
    
    for PID in $ALL_PIDS; do
        PROCESS_NAME=$(ps -p $PID -o comm= 2>/dev/null)
        echo "  Killing PID $PID ($PROCESS_NAME)..."
        kill -9 $PID 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "  ✅ Successfully killed PID $PID"
        else
            echo "  ❌ Failed to kill PID $PID (may require sudo)"
        fi
    done
else
    echo "Found cargo/rust processes:"
    lsof -i:$PORT | grep -E "(cargo|rust|wavelength)"
    
    echo ""
    echo "⚠️  Killing processes: $PIDS"
    
    for PID in $PIDS; do
        PROCESS_NAME=$(ps -p $PID -o comm= 2>/dev/null)
        echo "  Killing PID $PID ($PROCESS_NAME)..."
        kill -9 $PID 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "  ✅ Successfully killed PID $PID"
        else
            echo "  ❌ Failed to kill PID $PID (may require sudo)"
        fi
    done
fi

# Verify they're gone
sleep 1
REMAINING=$(lsof -ti:$PORT 2>/dev/null)
if [ -z "$REMAINING" ]; then
    echo ""
    echo "✅ All processes on port $PORT have been terminated"
else
    echo ""
    echo "⚠️  Some processes may still be running:"
    lsof -i:$PORT
    echo ""
    echo "Try running with sudo: sudo ./kill_cargo_servers.sh"
fi

