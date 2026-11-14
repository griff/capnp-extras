{ project, ... }:
project.crates.workspaceMembers.capnp-convert.build.override {
  runTests = true;
}
