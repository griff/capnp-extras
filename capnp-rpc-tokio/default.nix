{ project, ... }:
project.crates.workspaceMembers.capnp-rpc-tokio.build.override {
  runTests = true;
}
