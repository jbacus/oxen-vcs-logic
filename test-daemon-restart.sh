#!/bin/bash

set -e

echo "=== Daemon Restart Flow Test ==="
echo

# Get initial daemon PID
INITIAL_PID=$(ps aux | grep -i auxin-daemon | grep -v grep | awk '{print $2}')
echo "1. Initial daemon PID: $INITIAL_PID"

# Clear logs
> /tmp/auxin-restart-button.log
> /tmp/com.auxin.daemon.stderr

echo "2. Triggering restart via Swift test client..."

# Create a test Swift script that calls the XPC restart method
cat > /tmp/test-restart.swift << 'EOF'
import Foundation

@objc protocol OxenDaemonXPCProtocol {
    func restartDaemon(withReply reply: @escaping (Bool, String?) -> Void)
    func ping(withReply reply: @escaping (Bool) -> Void)
}

class TestClient {
    let connection: NSXPCConnection

    init() {
        connection = NSXPCConnection(machServiceName: "com.auxin.daemon.xpc", options: [])
        connection.remoteObjectInterface = NSXPCInterface(with: OxenDaemonXPCProtocol.self)
        connection.resume()
    }

    func testRestart() {
        let semaphore = DispatchSemaphore(value: 0)

        guard let proxy = connection.remoteObjectProxyWithErrorHandler({ error in
            print("XPC Error: \(error)")
            semaphore.signal()
        }) as? OxenDaemonXPCProtocol else {
            print("Failed to get proxy")
            exit(1)
        }

        print("Calling restartDaemon...")
        let startTime = Date()

        proxy.restartDaemon(withReply: { success, error in
            let elapsed = Date().timeIntervalSince(startTime)
            print("Got reply after \(String(format: "%.2f", elapsed))s: success=\(success), error=\(error ?? "nil")")

            // Wait for daemon to restart
            print("Waiting 3 seconds for daemon to restart...")
            sleep(3)

            // Test ping
            proxy.ping(withReply: { isAlive in
                print("Ping after restart: \(isAlive)")
                semaphore.signal()
            })
        })

        semaphore.wait()
        connection.invalidate()
    }
}

let client = TestClient()
client.testRestart()
EOF

# Compile and run the test
echo "3. Compiling test client..."
swiftc /tmp/test-restart.swift -o /tmp/test-restart 2>&1 | grep -v "warning:" || true

echo "4. Running restart test..."
/tmp/test-restart

# Wait a moment for the daemon to fully restart
sleep 2

# Get new daemon PID
NEW_PID=$(ps aux | grep -i auxin-daemon | grep -v grep | awk '{print $2}')
echo
echo "5. New daemon PID: $NEW_PID"

# Check if PID changed
if [ "$INITIAL_PID" != "$NEW_PID" ]; then
    echo "✅ SUCCESS: Daemon restarted (PID changed from $INITIAL_PID to $NEW_PID)"
else
    echo "❌ FAILURE: Daemon did not restart (PID still $INITIAL_PID)"
fi

# Check logs
echo
echo "6. Checking restart logs..."
if [ -f /tmp/auxin-restart-button.log ]; then
    echo "--- Restart button log ---"
    tail -20 /tmp/auxin-restart-button.log
fi

echo
echo "7. Checking daemon stderr..."
tail -10 /tmp/com.auxin.daemon.stderr

# Check how many daemon instances are running
echo
echo "8. Checking for duplicate daemons..."
DAEMON_COUNT=$(ps aux | grep -i auxin-daemon | grep -v grep | wc -l)
if [ "$DAEMON_COUNT" -eq 1 ]; then
    echo "✅ SUCCESS: Only one daemon instance running"
else
    echo "❌ WARNING: $DAEMON_COUNT daemon instances running:"
    ps aux | grep -i auxin-daemon | grep -v grep
fi

echo
echo "=== Test Complete ==="
