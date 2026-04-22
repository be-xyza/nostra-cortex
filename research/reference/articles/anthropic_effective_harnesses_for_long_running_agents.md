# Effective Harnesses for Long-Running Agents (Anthropic, Nov 2025)

## Core Concept
AI agents struggle with "context window amnesia" on long-running tasks. Instead of relying purely on larger context windows or more capable models, the solution is **effective scaffolding and external memory**.

## The Two-Part Harness Solution

Anthropic's Claude Agent SDK implemented a two-part solution for long-running projects:

### 1. Initializer Agent (The Planner)
Runs only once at the start of a project:
- Sets up the environment (directories, dependencies)
- Generates a comprehensive feature/task list
- Creates a structured progress document (e.g., `claude-progress.txt`)
- Makes the initial Git commit to establish the baseline

### 2. Coding Agent (The Executor)
Runs repeatedly for all subsequent sessions:
- Wakes up with "amnesia" (fresh context)
- Reads the external memory (`claude-progress.txt` and Git logs) to orient itself
- Selects a *single* unfinished feature from the list
- Implements, tests, and commits the feature
- Updates `claude-progress.txt` with the results
- Exits (to be spun up again for the next task)

## Key Takeaway
To make agents disciplined over long timeframes, you must provide them with **structural external memory** (files, git history) that they read at boot and write to before exiting, rather than trying to keep a single massive context window alive indefinitely.
