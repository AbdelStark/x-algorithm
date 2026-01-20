<roles>
| Role | Responsibility | Does NOT |
| --- | --- | --- |
| Coordinator | Scopes work, assigns tasks by component (home-mixer, thunder, phoenix, candidate-pipeline) | Implement code |
| Executor | Implements assigned changes and tests | Make architecture changes without approval |
| Reviewer | Checks correctness, risks, and tests | Write features |
</roles>

<delegation>
For any task request:
1. Investigation phase: identify affected components/files and draft an implementation plan.
2. Execution phase: implement, test, report blockers quickly.
</delegation>

<task_states>
todo -> inprogress -> inreview -> done
                 -> blocked
                 -> cancelled
</task_states>

<task_template>
## [Type]: [Title]

### Problem
[1-2 sentence problem statement]

### Context
- Relevant files/modules
- Dependencies

### Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Tests pass (if applicable)

### Scope Boundaries
- NOT included: [explicit exclusions]
</task_template>

<parallelization>
SAFE to parallelize:
- Tasks in different top-level components (home-mixer vs phoenix).
- Test writing alongside implementation when files do not overlap.

MUST serialize:
- Changes to the same module or shared pipeline wiring.
- Any change that affects public interfaces between components.

Conflict resolution:
1. Detect overlapping files early.
2. Pause the later task.
3. Complete and merge the first task.
4. Rebase or re-apply the second task.
5. Re-verify behavior.
</parallelization>

<escalation>
Escalate when:
- Blocked >30 minutes without progress
- Task requires scope/priority decision
- Security implications identified
- Breaking change required

Format:
## Escalation: [Title]
**Task**: [reference]
**Blocker**: [description]
**Options**:
1. [Option A] — [pros/cons]
2. [Option B] — [pros/cons]
**Recommendation**: [which and why]
</escalation>
