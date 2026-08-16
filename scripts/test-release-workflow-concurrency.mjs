import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const workflow = readFileSync(".github/workflows/release.yml", "utf8");

assert.match(workflow, /^on:\n  workflow_dispatch:/m);
assert.match(workflow, /^permissions:\n  contents: write\n  packages: write$/m);
assert.match(
  workflow,
  /^concurrency:\n  group: \$\{\{ github\.workflow \}\}-\$\{\{ github\.run_id \}\}\n  cancel-in-progress: false$/m,
);

const jobNames = [...workflow.matchAll(/^  ([a-z][a-z0-9-]+):$/gm)].map((match) => match[1]);
assert.deepEqual(jobNames, ["determine-version", "build", "create-release"]);

console.log("release workflow concurrency contract passed");
