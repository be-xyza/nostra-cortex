const WebSocket = require('ws');

console.log('Connecting to ws://localhost:3000/ws...');
const ws = new WebSocket('ws://localhost:3000/ws');

ws.on('open', function open() {
    console.log('✓ Connected to Gateway');
    ws.send('Hello Cortex');
    console.log('✓ Sent message');

    // Give it a moment to receive echo/ack if implemented, then exit
    setTimeout(() => {
        console.log('Test Passed');
        ws.close();
        process.exit(0);
    }, 1000);
});

ws.on('message', function message(data) {
    console.log('received: %s', data);
});

ws.on('error', function error(err) {
    console.error('✗ Connection Error:', err.message);
    process.exit(1);
});
