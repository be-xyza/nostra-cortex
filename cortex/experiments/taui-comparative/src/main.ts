import { TAUIRuntime } from '@taui-standard/core';

// Represents a comparative Nostra A2UI JSON node structurally migrated into TAUI spec format.
const mySpecDoc = {
    version: "1.0",
    screen: {
        type: "Box",
        children: [
            {
                type: "Text",
                content: "System Initialized: Cortex Eudaemon Linked."
            },
            {
                type: "Text",
                content: "Executing TAUI payload render..."
            },
            {
                type: "Select",
                id: "action-selector",
                options: [
                    { label: "Approve Plan", value: "proceed" },
                    { label: "Reject Plan", value: "halt" },
                    { label: "Modify Params", value: "steer" }
                ]
            }
        ]
    }
};

const runtime = new TAUIRuntime();

console.log("Cortex TAUI Experiment Initializing...");
console.log("Setting Stateless Document bounds...");
runtime.setDocument(mySpecDoc);

runtime.onEvent((event: any) => {
    // Unlike A2UI over pi-tui, TAUI normalizes all events natively off the entire document.
    console.log(`\nHeadless Agent Loop intercepted TAUI Event:`, event);
    
    if (event.type === "action") {
        console.log(`\nTest Passed: Action registered without manual focus tracking! Action: ${event.action}, Event: ${JSON.stringify(event)}`);
        process.exit(0);
    }
});

// Simulate raw terminal CSI byte sequences coming from STDIN
console.log("Simulating raw terminal byte payload ('\\x1B[B' -> 'down' navigate then '\\r' -> select)...");
runtime.dispatchRawEvent('\x1B[B'); // Simulate DOWN arrow
runtime.dispatchRawEvent('\r');     // Simulate ENTER key
