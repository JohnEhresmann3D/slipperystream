# Capability Vocabulary

Skills declare what they need in terms of **capabilities**, never concrete
tools. Each project's `MANIFEST.md` maps these names to whatever is actually
connected (built-in tools, MCP servers, or "not available"). This indirection
is what lets a skill written for one project run unchanged in another with a
completely different tool set.

**Rules:**

- A skill's `capabilities_needed` list may only use names defined here. If a
  new skill needs a capability that doesn't exist yet, add it to this file
  first — with a definition general enough that at least two different
  concrete tools could satisfy it.
- If the manifest maps a capability to nothing, a skill that needs it must say
  so and degrade honestly (ask the human for the information, or state that
  step was skipped) rather than pretending the step happened.

---

## Vocabulary

| Capability | Definition | Example bindings |
|---|---|---|
| `web_research` | Search and read sources on the public web | WebSearch/WebFetch (Claude Code), web search (claude.ai), Brave MCP |
| `file_read` | Read files in the project workspace | Read (Claude Code), project knowledge (Claude Project), caller-supplied context (API) |
| `file_write` | Create or modify files in the project workspace | Write/Edit (Claude Code); **unavailable** in a plain Claude Project |
| `codebase_search` | Search project contents by pattern or keyword | Grep/Glob (Claude Code), repository MCP |
| `code_execution` | Run code or shell commands and observe results | Bash (Claude Code), code-execution tool (API) |
| `version_control` | Inspect history, diffs, branches | git via shell, GitHub MCP |
| `issue_tracking` | Read and write tickets/tasks in a tracker | Linear MCP, Jira MCP, GitHub Issues |
| `design_lookup` | Retrieve design artifacts (mocks, specs, boards) | Figma MCP, attached design docs |
| `data_query` | Query structured data stores | database MCP, BigQuery MCP |
| `communication` | Send messages to humans outside this conversation | Slack MCP, email MCP — **always behind the §2 irreversible-action trigger** |

## Adding to the Vocabulary

A good capability name describes an *ability*, not a product. `issue_tracking`
is a capability; `linear` is a binding. If you can't imagine a second tool
that could satisfy the name, it's a binding wearing a capability's clothes —
generalize it or don't add it.
