# Agent Rules & Conventions

This document provides coding conventions, project structure, and common commands for AI coding agents working on this project.

## Project Overview

A self-hosted service for managing Cooklang recipe files. Cooklang is a markup language for recipes that makes them easy to read, write, and parse.

**Target Audience**: Users self-hosting for themselves and their family.

**Design Philosophy**: Keep it minimal. Use the official cooklang crate directly without unnecessary wrapper layers.

## Technology Stack

- **Language**: Rust (1.83+)
- **Parser**: Official `cooklang-rs` crate (v0.6)
- **Storage**: Git repository for recipe files + pluggable backends (DiskStorage, GitStorage)
- **Caching**: In-memory DashMap (rebuilt from storage on startup, no migrations needed)
- **API**: REST with Axum on Docker
- **Auth**: JWT framework (placeholder for future authentication)

## Architecture Decisions

- ✅ **Git as source of truth** - Recipe files stored in git; all API operations commit changes
- ✅ **Recipe ID**: SHA256 hash of git_path (first 12 hex chars) - deterministic, URL-friendly, changes on rename
- ✅ **Recipe Name/Title**: Derived from Cooklang YAML front matter `title` field. File names auto-generated and kept in sync with titles. Format: `---\ntitle: Recipe Name\n---`
- ✅ **Thread Safety**: Mutex-wrapped git operations for DiskStorage; atomic operations per backend
- ✅ **Cooklang Validation**: All recipe content must include YAML front matter with `title` field (enforced on create/update). Missing title → 400 Bad Request.

## Code Structure

```
/
├── src/
│   ├── api/          # REST API module (mod.rs, handlers.rs, models.rs, responses.rs)
│   ├── storage/      # Storage backends (disk.rs, git.rs)
│   ├── cache.rs      # In-memory cache (DashMap)
│   ├── parser.rs     # Cooklang parsing wrapper
│   └── main.rs       # Application entry point
├── tests/            # Integration tests
├── docs/             # API docs, OpenAPI, Postman, samples
├── scripts/          # Build and deployment scripts
└── config/           # Configuration files
```

## Common Commands

### Development
```bash
cargo build              # Build binary
cargo run              # Run dev server
cargo test             # Run all tests
cargo test -- --nocapture  # Show test output
cargo clippy           # Lint
cargo fmt              # Format
cargo check            # Check without building
```

### Production & Deployment
```bash
cargo build --release          # Build release binary
docker-compose up -d           # Run with Docker
docker-compose logs -f         # View logs
docker-compose down            # Stop
scripts/docker-test.sh         # Test Docker image
```

## Development Workflow

**BEFORE starting any work**:
1. Consult [PROJECT_PLAN.md](PROJECT_PLAN.md) for current phase, completed milestones, and technical debt
2. Review the relevant milestone checklist (e.g., `MILESTONE_CATEGORY_PATH_REFACTOR.md`) if one exists

**For each milestone**:
1. Create feature branch from `main`
2. Write tests first (TDD approach)
3. Implement feature
4. Run linter and tests: `cargo clippy && cargo test`
5. Update documentation (see Documentation Requirements below)
6. Create pull request

**CRITICAL: Review and Refine the Plan First**:
- Do NOT proceed to implementation until the plan is clear and detailed
- For each milestone, analyze:
  - Is it aligned with project goals and philosophy?
  - Are requirements specific enough (data structures, APIs, behaviors)?
  - Are edge cases and error handling covered?
  - Is the scope reasonable for one milestone?
- Propose revisions to PROJECT_PLAN.md with specific, actionable details
- Get explicit approval before writing code

**CRITICAL: Question Technology Choices Anytime**:
- For any major technology decision, ask:
  - "Is this actually needed?"
  - "What's the simplest solution?"
  - "Are there simpler alternatives?"
- Self-hosted family scenario: prefer simple, zero-config solutions
- When deciding to keep vs change: Update AGENTS.md "Architecture Decisions" with the reasoning

**CRITICAL: Use Checklists to Track Milestone Implementation**:
- For each milestone being implemented, create a detailed checklist BEFORE starting work
- Structure it like: Specification → Core Logic → API Layer → Documentation → Testing & Verification
- Check off items as you complete them - this keeps progress visible and prevents scope creep
- Once the milestone is complete, delete the checklist file (it served its purpose)

## Documentation Requirements

**API Documentation is MANDATORY**: When you modify ANY API endpoint, update these files in the same commit:
- `src/api/handlers.rs` - Endpoint implementation
- `docs/API.md` - Human-readable documentation with curl examples
- `docs/openapi.yaml` - OpenAPI 3.0 spec (validate: `python3 -c "import yaml; yaml.safe_load(open('docs/openapi.yaml'))"`)
- `docs/postman-collection.json` - Postman requests (validate: `python3 -m json.tool docs/postman-collection.json > /dev/null`)
- `docs/SAMPLE-RECIPES.md` - If adding new testing scenarios

**Documentation Maintenance**:
- **AGENTS.md**: Update when adding commands, coding patterns, or making architecture decisions
- **PROJECT_PLAN.md**: Update when completing milestones, making decisions, or identifying technical debt. Keep it concise.
- **README.md**: Update when changing project goals, features, or setup instructions
- **docs/TESTING.md**: Update when adding test cases or patterns
- **docs/API.md, openapi.yaml, postman-collection.json**: Update TOGETHER whenever API changes

**Important**: When completing significant work, consolidate implementation details into brief bullet-point summary. Keep only what's essential for future agents to understand current state. Avoid explicitly naming phases or detailed design rationale to prevent context bloat.

## Documentation Files Reference

| File | Purpose |
|------|---------|
| **PROJECT_PLAN.md** | Current project phase, milestones, architectural decisions, technical debt |
| **README.md** | Quick start, project status, features, deployment |
| **docs/API.md** | REST API endpoint documentation with examples |
| **docs/openapi.yaml** | Machine-readable OpenAPI 3.0 specification |
| **docs/postman-collection.json** | Postman collection with test requests |
| **docs/SAMPLE-RECIPES.md** | Sample recipes in Cooklang format with testing instructions |
| **docs/TESTING.md** | Test patterns, coverage targets, CI/CD info |
| **docs/DOCKER-TESTING.md** | Docker test script guide and debugging |

## Milestone Tracking & Documentation

**Directory Structure**: All milestone-related documents go in `milestones/{milestone-name}/`:
```
milestones/
├── category-path-refactor/
│   ├── milestone.md          # Main milestone spec and architecture
│   ├── phases/
│   │   ├── 01-api-spec.md    # Phase checklist
│   │   ├── 02-core-logic.md  # Phase checklist
│   │   └── 03-api-layer.md   # Phase checklist
│   └── tasks/
│       ├── TASK-extract-metadata.md
│       └── TASK-docker-tests.md
```

**When to Create Documents**:
- **Milestone.md**: Before starting any milestone work. Contains full spec, phases, and definition of done.
- **Phase.md**: When starting a new phase within the milestone. Contains detailed checklist and tasks for that phase.
- **Task.md**: When a phase is large (likely to exceed context window). Contains focused checklist for ONE task.

**Phase/Task Naming**:
- Phases: `{phase-number}-{description}.md` (e.g., `01-api-spec.md`)
- Tasks: `TASK-{name}.md` (e.g., `TASK-extract-metadata.md`)

**Checklist Format** (at top of phase/task file):
```markdown
# Phase {N}: {Description}

**Status**: ✅ COMPLETE | ⏳ IN PROGRESS | ❌ NOT STARTED
**Milestone**: {milestone-name}
**Phase**: {N} ({description})
**Branch**: {feature-branch-name}

---

## Task {N}.{N}: {Description}

- [x] Subtask 1
- [x] Subtask 2
- [ ] Subtask 3 (if in progress)
```

**Cleanup Rules** (must happen together in same commit):
1. When a phase is **100% complete**:
   - In `milestone.md`: Replace the detailed task list with a brief completion summary
   - Remove all `### Task X.Y:` sections and checklist items for that phase
   - Keep only: Phase heading with ✅ COMPLETE, **Completed** date, and bullet-point summary of key accomplishments
   - Example:
     ```markdown
     ## Phase 1: API Specification ✅ COMPLETE
     
     **Completed** (Nov 9, 2025):
     - Updated OpenAPI spec with new schemas
     - Added fallback lookup endpoints
     - Updated API.md documentation
     ```
   - Delete the phase checklist file from `phases/` directory
   - Commit both changes together:
   ```bash
   git add milestones/{milestone}/milestone.md
   git rm milestones/{milestone}/phases/{phase-number}-{description}.md
   git commit -m "[{milestone-name}] Phase {N} complete - delete phase checklist"
   ```

2. When a task is **100% complete**:
   - Update `milestone.md` to mark task as ✅ COMPLETE (if task-level tracking is used)
   - Delete the task checklist file from `tasks/` directory
   - Commit both changes together:
   ```bash
   git add milestones/{milestone}/milestone.md
   git rm milestones/{milestone}/tasks/TASK-{name}.md
   git commit -m "[{milestone-name}] Task {name} complete - delete task checklist"
   ```

3. Keep `milestone.md` in the repo as historical record (never delete - it shows what was accomplished in each phase)
4. CRITICAL: Do NOT update milestone.md and delete the checklist files in separate commits - they must be together

**Context Window Management**:
- If approaching context limit while working on a task:
  1. Commit your current work: `git add . && git commit -m "[{milestone-name}] Current progress on {task/phase}"`
  2. Add a note at top of phase/task file with last completed item and next action
  3. Next agent resumes from the note

**Before Handing Off**:
1. Update the phase/task file with current status
2. Add clear notes about what's done and what's next
3. Commit changes
4. Include the file path in the handoff message for next agent

## Code Conventions

- Write clean, readable code with meaningful variable names
- Keep functions small and focused on a single responsibility
- Write unit tests for all business logic
- Include integration tests for API endpoints
- Use dependency injection for testability
- Handle errors explicitly - no silent failures
- Validate all inputs and sanitize file paths to prevent directory traversal
- Aim for >80% code coverage
- Use descriptive test names that explain the scenario
