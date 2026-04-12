import { Component, Container, Box, Text, Spacer, SelectList, TUI } from "@mariozechner/pi-tui";
import chalk from "chalk";
import { A2UINode } from "./mockPayload.js";

export function createTUIComponent(node: A2UINode, tui: TUI): Component | null {
    if (!node.type) {
        return null;
    }

    const props = node.componentProperties || {};
    let children: Component[] = [];
    
    if (node.children?.explicitList) {
        children = node.children.explicitList
            .map(child => createTUIComponent(child, tui))
            .filter((c): c is Component => c !== null);
    }

    switch (node.type) {
        case "Container": {
            const container = new Container();
            children.forEach(c => container.addChild(c));
            return container;
        }
        case "Box": {
            const paddingX = props.paddingX || 1;
            const paddingY = props.paddingY || 1;
            let bgFn;
            if (props.bg === "gray") {
                bgFn = (t: string) => chalk.bgGray(t);
            }
            const box = new Box(paddingX, paddingY, bgFn);
            children.forEach(c => box.addChild(c));
            return box;
        }
        case "Text": {
            let colorFn = (t: string) => t;
            if (props.color === "cyan") colorFn = (t: string) => chalk.cyan(t);
            if (props.color === "red") colorFn = (t: string) => chalk.red(t);
            
            return new Text(
                colorFn(props.content || ""),
                props.paddingX || 0,
                props.paddingY || 0
            );
        }
        case "Spacer": {
            return new Spacer(props.lines || 1);
        }
        case "SelectList": {
            const theme = {
                selectedPrefix: (t: string) => chalk.green("> "),
                selectedText: (t: string) => chalk.green(t),
                description: (t: string) => chalk.gray(t),
                scrollInfo: (t: string) => chalk.cyan(t),
                noMatch: (t: string) => chalk.red(t)
            };
            const list = new SelectList(
                props.items || [],
                props.maxVisible || 5,
                theme
            );
            
            // Allow user to press Enter to select an item (this prevents terminal hanging indefinitely and validates E2E input)
            list.onSelect = (item) => {
                tui.stop();
                console.log(`\nAction Selected: ${item.label}`);
                if (typeof item.value === "string" && item.value.startsWith("open_web:")) {
                    console.log(`Handoff URL: ${item.value.slice("open_web:".length)}`);
                }
                process.exit(0);
            };

            // Focus the interactive list so the TUI routes keyboard inputs to it!
            setTimeout(() => tui.setFocus(list), 0);
            
            return list;
        }
        default:
            return new Text(`[Unsupported A2UI Widget: ${node.type}]`, 0, 0, (t) => chalk.bgRed(t));
    }
}
