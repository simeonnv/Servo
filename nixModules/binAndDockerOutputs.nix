{
  inputs,
  lib,
  ...
}: let
  inherit (inputs.self.lib) buildRustApp mkRustDocker;
in {
  perSystem = {
    config,
    self',
    inputs',
    pkgs,
    system,
    ...
  }: let
    oxalateApps = {
      auth = buildRustApp {inherit pkgs lib;} "auth";
      servo = buildRustApp {inherit pkgs lib;} "servo";
    };

    images = {
      auth = mkRustDocker pkgs "auth-server" "auth" oxalateApps.auth ["8989"];
      servo = mkRustDocker pkgs "servo-server" "servo" oxalateApps.servo ["18888"];
    };
  in {
    packages = {
      auth-app = oxalateApps.auth;
      servo-app = oxalateApps.servo;

      auth-image = images.auth;
      servo-image = images.servo;

      default = oxalateApps.servo;
    };

    apps = {
      auth = {
        type = "app";
        program = lib.getExe oxalateApps.auth;
      };
      servo = {
        type = "app";
        program = lib.getExe oxalateApps.servo;
      };
    };
  };
}
