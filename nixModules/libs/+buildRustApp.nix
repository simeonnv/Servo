{
  inputs,
  lib,
  self,
  ...
}: {
  flake.lib.buildRustApp = {
    pkgs,
    lib,
    ...
  }: name: let
    naersk-lib = pkgs.callPackage inputs.naersk {};
    deps = self.lib.getDeps pkgs lib;
  in
    naersk-lib.buildPackage ({
        pname = name;
        version = "0.1.0";
        src = ./../..;
        SQLX_OFFLINE = "true";
        SQLX_OFFLINE_DIR = ./../.. + "/.sqlx";

        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.libclang.lib}/lib/clang/${lib.getVersion pkgs.clang}/include";

        preBuild = ''
          export SET_MAKE_JOBS=$NIX_BUILD_CORES
        '';

        meta = {
          mainProgram = name;
        };

        cargoBuildOptions = x: x ++ ["-p" name];
        cargoTestOptions = x: x ++ ["-p" name];
      }
      // deps);
}
