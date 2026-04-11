# fsgdb Demo Narration

This follows the Duckcells pattern:

- pre-written narration, not improvised
- one narration segment per major scene
- visuals are edited first for pacing
- audio is written to the edited timeline
- slow operations can be sped up, but the narration must still match the edited footage

## Duckcells Patterns To Mirror

From the Duckcells demo, the useful structure to preserve is:

1. Each major scene has a clear label and one primary idea.
2. Narration is short, declarative, and tied to visible actions.
3. Segment boundaries match recording boundaries well enough to edit and align audio cleanly.
4. Long or variable-duration actions are handled in post by fitting narration to the edited video, not by forcing live pacing.

For fsgdb, the same rule applies:
- write the narration as scene-level segments
- keep each segment conceptually tight
- let editing handle long waits

---

## Scene List

1. General problem setup
2. Shared human-agent model
3. Clone Servo and build the graph
4. CLI proof of life
5. Open isolated VS Code session
6. Query Editor investigation
7. Graph Visualization investigation
8. Agent deep dive
9. Joint conclusion
10. Broader payoff

---

## Narration Segments

### Scene 1 - General Problem Setup

Text:

"Playwright already supports browser targets like Chromium, Edge, and Firefox. That works because those runtimes expose stable seams for launching, attaching, and driving them. The question is what it would take for another engine to fit that model."

Purpose:
- establish why browser automation support is a structural question
- explain why the case study matters

Notes:
- keep this over title card or clean setup visuals

---

### Scene 2 - Shared Human-Agent Model

Text:

"This is not a solo investigation. You inspect the codebase directly and visually. I reason over it structurally as the agent. fsgdb is the shared map between us."

Purpose:
- set up the two-person story
- make fsgdb the shared layer, not just a tool catalog

Notes:
- can be over a simple title card, split-screen, or setup sequence

---

### Scene 3 - Clone Servo and Build the Graph

Text:

"For the case study, we use Servo. We clone the repository and scan it once into a local graph database. From this point on, both of us work from the same structural model of the codebase."

Purpose:
- introduce Servo as the example
- establish the shared `graph.db`

Notes:
- show the clone command
- show the scan command clearly
- if scan time is long, speed up the waiting portion

Optional long-wait version:

"Servo is a large codebase, so this scan takes a bit. While it runs, fsgdb is extracting filesystem structure, code relationships, and git history into one local graph database."

---

### Scene 4 - CLI Proof of Life

Text:

"You start with the raw database. Before trusting any visualization or any agent answer, you inspect the graph directly from the command line. Stats and targeted queries give you a first pass at where startup, embedding, testing, and control surfaces might live."

Purpose:
- show the graph is real and queryable
- make the human start from source of truth

Notes:
- keep to one or two meaningful CLI queries
- avoid noisy terminal scrolling

---

### Scene 5 - Open Isolated VS Code Session

Text:

"Now we switch to the editor surface. The extension runs in an isolated VS Code session, but it points at the same graph database. This is the same structural model, just in a different interface."

Purpose:
- transition from terminal to editor
- reinforce same DB, different surface

Notes:
- this is a good place to visually show the isolated harness launch

---

### Scene 6 - Query Editor Investigation

Text:

"In the query editor, you move from the whole repository to a smaller set of candidate areas. You use Cypher to narrow the search around startup, embedding, testing, and other automation-relevant seams."

Purpose:
- show human-guided narrowing
- explain why the query editor matters

Notes:
- use queries that are readable on screen
- if a query is slow, speed up the wait and keep the result reveal at normal speed

Optional long-wait version:

"Some of these queries touch a broad part of the graph, so we speed through the wait. The goal here is not to admire the graph. It is to isolate the parts of Servo that might matter for automation."

---

### Scene 7 - Graph Visualization Investigation

Text:

"The graph view turns query results into structure. Now you can see whether likely control points are centralized, fragmented, or deeply coupled. This is where architectural intuition starts to form."

Purpose:
- convert rows into shape
- show why visual exploration helps the human

Notes:
- expand only one or two neighborhoods
- avoid overwhelming the viewer with a full dense graph

Optional long-wait version:

"Layout can take a moment when we load a denser neighborhood. We keep the trigger and the result in real time, and speed through the waiting in between."

---

### Scene 8 - Agent Deep Dive

Text:

"Once you've narrowed the search visually, I use the same graph as a structural reasoning layer. I trace likely entry points, dependencies, call paths, tests, and change risk. I'm not guessing from filenames. I'm following how the codebase is actually connected."

Purpose:
- explain what the agent is doing concretely
- make MCP feel rigorous rather than magical

Notes:
- show a few high-value questions and answers
- prefer one focused thread over a spray of unrelated tool calls

---

### Scene 9 - Joint Conclusion

Text:

"Now we can answer the real question with shared evidence. We know which seams look promising, which areas are more tightly coupled, and what we would validate next if we wanted to pursue a Playwright integration path."

Purpose:
- move from exploration to decision
- keep the conclusion engineering-specific

Notes:
- show a short summary card or concise list

---

### Scene 10 - Broader Payoff

Text:

"Servo is the example. The broader point is that fsgdb gives a developer and an agent the same map of a codebase. You can inspect it directly. I can reason over it structurally. And together we can evaluate unfamiliar systems with less guesswork."

Purpose:
- zoom back out from Servo
- end on the product thesis

Notes:
- keep the closing visual clean

---

## Timing Guidance

These are rough targets for spoken duration after editing.

| Scene | Target Duration |
|---|---|
| 1 | 9 to 12 seconds |
| 2 | 6 to 8 seconds |
| 3 | 10 to 16 seconds |
| 4 | 10 to 14 seconds |
| 5 | 5 to 7 seconds |
| 6 | 8 to 12 seconds |
| 7 | 8 to 12 seconds |
| 8 | 12 to 18 seconds |
| 9 | 8 to 12 seconds |
| 10 | 8 to 10 seconds |

Expected total:
- about 90 to 120 seconds depending on how much of the waiting is sped up

---

## Latency Rules For Narration

### Rule 1

If an operation is slow, either explain the wait or speed up the wait.

### Rule 2

If the visual is sped up, narration must be written to the sped-up edit, not the raw capture.

### Rule 3

Keep the trigger and the result reveal at normal speed whenever possible.

### Rule 4

Use long waits for explanation, not silence.

Good examples:
- explain what the scan is extracting
- explain what the query is trying to prove
- explain why the graph layout takes longer in a dense neighborhood

### Rule 5

Do not force narration to cover every second of footage.
Short silence is fine after a result lands if the visual needs a moment to breathe.

---

## Suggested Recording Segments

These should roughly align with edit groups, similar to Duckcells:

1. `seg_01_setup_and_scan`
2. `seg_02_cli`
3. `seg_03_vscode_query`
4. `seg_04_vscode_graph`
5. `seg_05_agent_and_conclusion`

This grouping is better than one segment per scene because:
- the surfaces stay coherent
- audio alignment remains manageable
- long waits can be edited within each surface segment

---

## Closing Guidance

The narration should always keep the emphasis here:

- not "look how many features fsgdb has"
- not "look how complicated Servo is"
- but "look how a human and an agent can work from the same structural truth"
