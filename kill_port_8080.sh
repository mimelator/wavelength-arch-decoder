#!/bin/bash
# Script to kill all processes running on port 8080

PORT=8080

echo "🔍 Finding processes on port $PORT..."

# Find PIDs listening on port 8080
PIDS=$(lsof -ti:$PORT)

if [ -z "$PIDS" ]; then
    echo "✅ No processes found on port $PORT"
    exit 0
fi

echo "Found processes:"
lsof -i:$PORT

echo ""
echo "⚠️  Killing processes: $PIDS"

# Kill the processes
for PID in $PIDS; do
    echo "  Killing PID $PID..."
    kill -9 $PID 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "  ✅ Successfully killed PID $PID"
    else
        echo "  ❌ Failed to kill PID $PID (may require sudo)"
    fi
done

# Verify they're gone
sleep 1
REMAINING=$(lsof -ti:$PORT)
if [ -z "$REMAINING" ]; then
    echo ""
    echo "✅ All processes on port $PORT have been terminated"
else
    echo ""
    echo "⚠️  Some processes may still be running. Try running with sudo:"
    echo "   sudo ./kill_port_8080.sh"
fi

