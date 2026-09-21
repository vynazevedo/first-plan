# Evidence-backed change preparation

The v1.5 workflow answers three questions before a change: what can be reused,
which references might be affected, and where validation evidence exists.

```bash
fpe context --query "validate email" --budget 8000 --json
fpe multi register --name web --path ../web
fpe impact
fpe contracts snapshot --out baseline.json
# After making a change:
fpe contracts diff --before baseline.json --fail-on-breaking --json
```

Context output distinguishes reusable symbol candidates, test references,
documented conventions and other references. Each item has a relative path,
line and xxh3 content hash. `revision` identifies HEAD; hashes identify the actual
working-tree content, including uncommitted edits. xxh3 is a freshness check,
not a cryptographic authenticity guarantee. A fresh document can still be wrong.

Git repositories use tracked and untracked non-ignored files. Without Git, a
bounded directory walk is used. Known build/dependency directories, `.env*`,
key files, binaries, files over 256KB and symlinks escaping the root are excluded.
This is not a general secret scanner. Review provider inputs before sending
private repository content to an external model.

The budget limits selected content and labels in characters, not tokenizer
tokens or the JSON envelope. Retrieval is lexical, using the existing symbol
extractors and identifier tokenization; it is not a complete call graph.
Generated convention documents with recorded source hashes are excluded if
those sources changed. Documents without provenance remain documented claims,
not verified facts. Re-run init with the desired layers to refresh stale claims.

## Preserve team instructions

`fpe generate --tool all` preserves text outside its `first-plan:begin/end`
managed block. Re-running updates that block. Malformed or duplicate markers
fail without replacing that file; target files are preflighted per adapter.
Legacy files without markers are preserved and a block is appended. Review
duplicate legacy generated prose once during migration. Existing Cursor MDC
frontmatter is preserved; newly created MDC files retain frontmatter at the top.

## Contracts and strict gates

OpenAPI snapshots now include operation and inherited path parameters,
request bodies, responses, security requirements and resolved local `$ref`
values. Endpoints are identified by spec file, method and path. Renaming a spec
therefore appears as removal/addition and needs review.

Required input additions, response removals, type changes and incompatible enum
directions are detected. Optional parameter additions and documentation edits
are non-breaking. Unrecognized schema changes are conservatively marked
breaking for review; this is not a complete OpenAPI compatibility solver.

`--fail-on-breaking` also rejects incomplete analysis: old snapshots without
details, unresolved/external/recursive references, unsupported versions,
Protobuf/GraphQL contracts, and skipped/missing cross-repo baselines. JSON and
Markdown expose warnings even without the strict flag. Rebuild old baselines
from the baseline commit, not from the changed checkout.

`fpe impact` matches source references to registered APIs' literal paths or
operation IDs, retaining producer, consumer, method, contract file and source
location. `candidate` means a textual reference was found, not proof that this
consumer invokes that endpoint or that every consumer has been found.

## Deployment evidence

Git tags establish release history only. After a deployment succeeds, its
pipeline can record an observation:

```bash
fpe deployment record --environment production \
  --commit FULL_COMMIT_SHA --source https://ci.example/runs/123
fpe deployment status
```

The commit must exist locally. `--observed-at` accepts an RFC3339 timestamp;
otherwise the current time is recorded. One latest observation per environment
is stored in `.first-plan/deployments.json`. Missing evidence is `unknown`.
Observations are `observed_not_live_verified`; records older than 24 hours have
a stale warning. A supplied source is attribution, not an authenticated query
to that source. Rollbacks and later deployments require a new observation.

## MCP integration

Configure a client to launch:

```json
{
  "mcpServers": {
    "first-plan": {
      "command": "fpe",
      "args": ["mcp", "--root", "/absolute/project/path"]
    }
  }
}
```

The server supports MCP 2025-11-25 stdio with `context`, `impact` and
`deployment_status` tools. The client cannot override the root. Impact can read
repositories explicitly listed by the operator in `.first-plan/multi.yaml`.
No model calls or commands from retrieved source text are executed. stdout is
reserved for JSON-RPC; diagnostics go to stderr. Tool results include explicit
limitations. HTTP transport and resources/prompts are not exposed.

Protocol references: [stdio transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports),
[lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle),
[tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools).
