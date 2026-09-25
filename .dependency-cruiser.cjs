// Module boundaries: docs/design/02-architecture.md (dependencies point downward only).
/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "no-circular",
      severity: "error",
      from: {},
      to: { circular: true },
    },
    {
      name: "core-is-pure",
      comment: "domain / arbiter / skills are pure: no Node built-ins, no I/O modules.",
      severity: "error",
      from: { path: "^src/(domain|arbiter|skills)/" },
      to: { dependencyTypes: ["core"] },
    },
    {
      name: "core-does-not-import-io",
      severity: "error",
      from: { path: "^src/(domain|arbiter|skills)/" },
      to: { path: "^src/(reasoning|reflex|sensing|transport|loop|app|profiles)/" },
    },
    {
      name: "jev-http-only-in-reasoning-jev",
      comment:
        "Only src/reasoning/ may import jev.ts, the TypeSafe API client (docs/design/03-reasoning-layer.md).",
      severity: "error",
      from: { pathNot: "^src/reasoning/|^scripts/experiments/" },
      to: { path: "^src/reasoning/jev\\.ts$" },
    },
    {
      name: "tests-only-from-tests",
      severity: "error",
      from: { path: "^src/" },
      to: { path: "^tests/" },
    },
  ],
  options: {
    doNotFollow: { path: "node_modules" },
    tsPreCompilationDeps: true,
    tsConfig: { fileName: "tsconfig.json" },
  },
};
