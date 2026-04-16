# fsgdb Demo Plan

> Core thesis: **fsgdb is the shared map a developer and an agent use to understand a complex codebase together.**
>
> Case study: **Servo**
>
> Investigation question:
> **What would it take for a browser engine like Servo to become a viable Playwright automation target?**

## Opening Frame

Playwright already works with established browser targets like Chromium, Edge, and Firefox.

That only works because those runtimes expose enough control surface to:
- launch reliably
- attach to the right process
- drive browser behavior
- validate results through stable automation seams

So the question is not just "is Servo a browser engine?"
The real question is:

**Does Servo expose the kinds of seams a framework like Playwright would need?**

We use Servo as the example.
We use fsgdb to investigate it together.

## Story Structure

The demo is about collaboration:

- **You** inspect the codebase directly and visually.
- **I** reason over it structurally as the coding agent.
- **We** use the same graph database to make an engineering judgment together.

Servo is the case study in the middle.
fsgdb is the product story at the beginning and the end.

## Shared Artifacts

- Target repo: `/tmp/servo`
- Shared graph database: `/tmp/fsgdb-servo-demo/graph.db`

Every surface in the demo should point at that exact DB path.

## Operational Entry Point

The demo now has a BraceBalance-style operational runner:

```bash
python3 ./demo/scripts/run_demo.py --dry-run
```

That runner is responsible for:

- resetting the temp Servo clone and demo database
- building the release binary
- cloning and scanning Servo
- proving the CLI scene works
- proving the VS Code query and graph scenes work
- verifying Copilot MCP configuration and repo trust
- running a real non-editing Copilot dry run against the shared graph
- checking GUI launch prerequisites before a visible session

The visible agent session entrypoints are:

```bash
python3 ./demo/scripts/run_demo.py
python3 ./demo/scripts/run_demo.py --current-terminal
```

Supporting scripts live in `demo/scripts/`:

- `run_demo.py`
- `reset_demo_state.py`
- `start_copilot_session.py`
- `open_agent_terminal.py`
- `record_mcp_scene.py`

## Visible Surfaces

1. Terminal
2. VS Code extension
3. Agent/MCP analysis

The VS Code segment is launched and recorded with the existing harness patterns:
- `@vscode/test-electron`
- `@vscode/test-cli`
- isolated config flow from `VS_CODE.md`

Those tools are recording and control infrastructure for the demo, not part of the Servo investigation itself.

---

## Phase 0 - Set the Stage

- [ ] Introduce the general problem:
  "How do we evaluate whether a large unfamiliar codebase has the seams needed for automation?"
- [ ] Explain why Playwright's existing targets matter:
  Chromium, Edge, and Firefox are viable because they expose real launch, control, and test surfaces
- [ ] Introduce Servo as the case study, not the thesis
- [ ] Introduce fsgdb as the shared intelligence layer between developer and agent

Narration:
"Playwright can automate browsers like Chromium, Edge, and Firefox because those runtimes expose stable seams for launching, attaching, and driving them. We want to know what it would take for another engine, like Servo, to fit that model. To answer that, we need more than file search. We need a structural map of the codebase."

---

## Phase 1 - Build the Shared Graph

- [ ] Clone Servo into `/tmp/servo`
- [ ] Build fsgdb release binary with MCP support
- [ ] Scan Servo into `/tmp/fsgdb-servo-demo/graph.db`
- [ ] Show the database file exists
- [ ] Run `stats` to prove the graph is populated

Example commands:

```bash
git clone https://github.com/servo/servo /tmp/servo
./target/release/fsgdb --database /tmp/fsgdb-servo-demo/graph.db scan /tmp/servo --enable-code-analysis --enable-git-analysis
./target/release/fsgdb --database /tmp/fsgdb-servo-demo/graph.db stats
```

Narration:
"We scan Servo once into a local graph database. From here on, both of us work from the same structural model."

---

## Phase 2 - You Inspect the Graph Directly in CLI

- [ ] Run a few direct CLI queries against the shared DB
- [ ] Establish first-pass orientation:
  - likely startup surfaces
  - likely embedding or shell layers
  - likely test-related modules
  - likely automation-adjacent code
- [ ] Show the graph is queryable as a real database before opening any UI

Suggested goals for CLI inspection:
- identify major browser and runtime-related files
- find files or modules with names related to testing, webdriver, embedding, shell, startup
- inspect dependency paths between candidate areas
- get a sense of where automation-relevant logic might concentrate

Narration:
"You begin from the raw database. Before trusting any visualization or any agent answer, you inspect the graph directly."

---

## Phase 3 - You Explore Servo Visually in VS Code

- [ ] Build the extension
- [ ] Launch isolated VS Code session using the existing test harness
- [ ] Point the extension at `/tmp/fsgdb-servo-demo/graph.db`
- [ ] Record the VS Code window as its own target
- [ ] Open:
  - **Cypher Query Editor**
  - **Graph Visualization**

### Query Editor

- [ ] Run targeted Cypher queries around likely automation seams
- [ ] Narrow from whole-repo exploration to a small set of candidate areas
- [ ] Use results to identify modules worth visualizing

### Graph Visualization

- [ ] Visualize one or two candidate regions returned by the query editor
- [ ] Expand neighbors and dependency edges
- [ ] Inspect whether likely control points are centralized or fragmented
- [ ] Click through nodes and details to build architectural intuition

Harness note:
Use the existing isolated VS Code test flow from `VS_CODE.md` so the session is reproducible and recordable.

Narration:
"In VS Code, you move from rows to shape. The query editor lets you interrogate the graph directly, and the visualization shows how the likely automation-relevant parts of Servo fit together."

---

## Phase 4 - I Investigate Deeply as the Agent

- [ ] Start MCP against the same graph DB
- [ ] Use MCP tools to analyze feasibility in layers

### Layer 1: Find candidate surfaces

- [ ] `search_files`
- [ ] `search_symbols`
- [ ] `semantic_search`

Goal:
Find candidate code areas related to:
- process launch
- browser startup
- embedding
- runtime control
- test harnesses
- external control surfaces

### Layer 2: Understand module boundaries

- [ ] `get_file_structure`
- [ ] `explain_module`
- [ ] `find_dependencies`
- [ ] `find_path`

Goal:
Understand what each candidate module contains and how the relevant pieces connect.

### Layer 3: Trace behavior and coupling

- [ ] `get_call_graph`
- [ ] `find_references`
- [ ] `analyze_impact`
- [ ] `get_cross_language_calls`

Goal:
Find actual hook points, call paths, and blast radius for any integration strategy.

### Layer 4: Understand validation cost

- [ ] `get_tests_for_function`
- [ ] `get_affected_tests`
- [ ] `get_test_coverage`

Goal:
Determine whether likely integration areas are already testable and what would need to be validated.

### Layer 5: Understand risk and churn

- [ ] `get_hotspots`
- [ ] `get_commits`
- [ ] `get_function_history`

Goal:
Assess whether the likely seams are stable enough to build against.

Narration:
"Once you've narrowed the search visually, I use the graph as a structural reasoning layer. I'm not guessing from filenames. I'm tracing entry points, dependencies, call chains, tests, and change risk."

---

## Phase 5 - We Converge on the Engineering Answer

- [ ] Summarize what Playwright-like automation would need from a target
- [ ] Summarize what Servo appears to provide
- [ ] Identify likely hook points
- [ ] Identify missing seams or integration risks
- [ ] Identify the first validation steps we would take next
- [ ] Give a concrete feasibility judgment

Possible conclusion structure:
- promising entry points
- likely control and launch seam
- likely validation path
- main architectural risks
- recommended next experiment

Narration:
"You use the graph to form the hypotheses. I use the graph to validate them structurally. Together, we turn a large unfamiliar codebase into an actionable engineering decision."

---

## Phase 6 - Zoom Back Out

- [ ] Re-state that Servo was the example, not the thesis
- [ ] Generalize the pattern:
  developer + agent + shared graph + real engineering question
- [ ] Close on fsgdb's broader value

Narration:
"Servo is just the case study. The real point is that fsgdb gives both of us the same map of a complex codebase. You can inspect it directly. I can reason over it structurally. And we can make decisions from shared evidence instead of separate guesses."

---

## Recording Breakdown

### Terminal

Use for:
- clone
- scan
- stats
- first-pass CLI queries

### VS Code

Use for:
- extension-driven query editor
- graph visualization
- node exploration
- isolated, reproducible launch via `@vscode/test-cli` and `@vscode/test-electron`

### Agent

Use for:
- MCP-backed deep analysis
- structural answers
- final engineering conclusion

---

## Closing Line

**"fsgdb gives a developer and an agent the same map of a codebase, so they can evaluate unfamiliar systems together and make real engineering decisions with less guesswork."**

---

## Scene-by-Scene Shot List

This section translates the plan into a recordable sequence.

### Scene 1 - General Problem Setup

Visual:
- terminal or title card
- brief framing text for the problem

On-screen idea:
- Playwright already targets Chromium, Edge, and Firefox
- those targets work because the browser exposes stable automation seams
- we want to understand what it would take for another engine to fit that model

Narration:
"Playwright already supports browser targets like Chromium, Edge, and Firefox. That only works because those runtimes expose stable seams for launching, attaching, and driving them. The question is what it would take for another engine to fit that model."

### Scene 2 - Introduce the Collaboration Model

Visual:
- terminal or title card
- optional split framing of "You", "Me", and "Shared Graph"

On-screen idea:
- you inspect the codebase directly
- I reason over it structurally
- we share one graph database

Narration:
"This is not a solo investigation. You inspect the codebase directly and visually. I reason over it structurally as the agent. fsgdb is the shared map between us."

### Scene 3 - Clone Servo and Create the Graph

Visual:
- terminal
- clone Servo
- run scan into `/tmp/fsgdb-servo-demo/graph.db`

On-screen idea:
- show the scan command clearly
- show the database path clearly

Narration:
"For the case study, we use Servo. We scan it once into a local graph database. From here on, both of us work from the same structural model."

### Scene 4 - CLI Proof of Life

Visual:
- terminal
- `stats`
- one or two direct queries

On-screen idea:
- prove the graph is real
- show that raw inspection works before any UI

Narration:
"You begin from the raw database. Before trusting a UI or an agent answer, you inspect the graph directly."

### Scene 5 - Open VS Code in Isolated Harness

Visual:
- launch isolated VS Code session through the existing extension harness
- extension opens against the shared DB path

On-screen idea:
- this is a reproducible VS Code surface, not a hand-driven ad hoc session

Narration:
"Now we switch to the editor surface. The extension runs in an isolated VS Code session, using the same graph database."

### Scene 6 - Query Editor Investigation

Visual:
- VS Code extension
- Cypher Query Editor tab
- several targeted queries around startup, embedding, test, and automation-relevant areas

On-screen idea:
- use the query editor to narrow candidate areas

Narration:
"In the query editor, you move from the whole repo to a smaller set of candidate areas that might matter for automation."

### Scene 7 - Graph Visualization Investigation

Visual:
- VS Code extension
- Graph Visualization tab
- load one or two candidate regions
- expand neighbors
- inspect node details

On-screen idea:
- show architectural shape, not just rows

Narration:
"The graph view turns search results into structure. Now you can see whether likely control points are centralized, fragmented, or deeply coupled."

### Scene 8 - Agent Deep Dive

Visual:
- MCP or agent terminal/chat surface
- show questions and answers grounded in the graph

On-screen idea:
- candidate hook points
- dependency paths
- tests
- change risk

Narration:
"Once you've narrowed the search visually, I use the same graph as a structural reasoning layer. I trace entry points, dependencies, call chains, tests, and change risk."

### Scene 9 - Joint Conclusion

Visual:
- short summary card or terminal/editor recap
- maybe three to five bullets

On-screen idea:
- what Playwright-like automation needs
- what Servo appears to provide
- what looks promising
- what remains risky

Narration:
"Now we can answer the real question. Not perfectly, not magically, but with shared evidence. We know where the likely seams are, what looks promising, and what would need validation next."

### Scene 10 - Broader Payoff

Visual:
- simple closing card

Narration:
"Servo is the example. The real point is that fsgdb gives a developer and an agent the same map of a codebase, so they can evaluate unfamiliar systems together and make real engineering decisions with less guesswork."

---

## Latency Strategy

Some parts of this demo will be slow, especially on a codebase as large as Servo.
The recording plan should treat long latency as a first-class constraint.

### Rule 1 - Do Not Leave Dead Air Unexplained

If an action takes a long time, do one of two things:
- narrate what is happening and why it takes time, or
- speed up the video during the wait

Do not leave unexplained idle footage.

### Rule 2 - Use Speed Ramps for Real Waits

For operations like:
- full scan
- heavy Cypher query
- graph layout load
- extension initialization
- large graph visualization refresh

record the real action, then accelerate the waiting portion in post.

Recommended pattern:
1. show the command or action at normal speed
2. once the wait is obvious, ramp to 4x to 12x speed
3. return to 1x when useful output appears

### Rule 3 - Audio Must Match the Edited Visual

If the visual is sped up, narration must account for that edited timing.

That means either:
- the narration explicitly describes the wait while the sped-up footage plays, or
- the narration starts after the sped-up section ends

Do not lay natural-speed narration over a sped-up section unless it is intentionally written for that pacing.

### Rule 4 - Prefer Explanatory Audio During Long Waits

The best use of sped-up time is explanation.

Examples:
- during scan time, explain what fsgdb is extracting
- during query time, explain what the query is trying to prove
- during graph render time, explain what we expect to learn from the visualization

This turns waiting into useful context instead of filler.

### Rule 5 - Keep the Beginning and End of Each Slow Operation Real-Time

Even when accelerating a long wait, preserve:
- the user action that triggers it
- the moment meaningful results appear

Those moments should usually stay at 1x speed so the operation feels real and legible.

### Rule 6 - Call Out Expensive Operations Honestly

When a query or graph load is expensive, say so.

Examples:
- "This scan takes a bit because Servo is large and we're extracting code structure and git history."
- "This query touches a broad part of the graph, so we'll speed through the wait."
- "The layout takes a moment here because we're loading a denser neighborhood around the candidate module."

This keeps the demo honest.

---

## Editing Notes

- Keep setup actions short and readable.
- Preserve exact commands on screen long enough to read.
- Speed up only the waiting portions, not the trigger or the result.
- If a query returns too much output, cut to the meaningful section rather than leaving noisy scrolling in real time.
- Prefer short summary cards after dense scenes so the audience does not need to remember raw terminal output.
- Narration should never fight the visual pace. Write audio to the edited timeline, not the raw capture.
