import { TAUIRuntime } from '@taui-standard/core';

// Represents a comparative Nostra A2UI-like payload translated into the TAUI document format.
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
console.log("Setting stateless document bounds...");
runtime.setDocument(mySpecDoc);

runtime.onEvent((event: any) => {
    console.log(`\nHeadless Agent Loop intercepted TAUI Event:`, event);
    
    if (event.type === "action") {
        console.log(`\nObserved action event: ${event.action}, payload: ${JSON.stringify(event)}`);
        process.exit(0);
    }
});

console.log("Simulating raw terminal byte payload ('\\x1B[B' then '\\r')...");

try {
    runtime.dispatchRawEvent('\x1B[B');
    runtime.dispatchRawEvent('\r');
    console.log("Runtime accepted raw terminal input without throwing.");
} catch (error) {
    console.error(
        "Observed runtime limitation: raw terminal byte dispatch is not normalized successfully in the installed TAUI core package.",
    );
    console.error(error);
    process.exit(0);
}
