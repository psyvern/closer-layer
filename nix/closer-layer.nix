{
  inputs,
  lib,
  # Dependencies
  makeWrapper,
  glib,
  rustPlatform,
  atk,
  gobject-introspection,
  graphene,
  gtk4,
  gtk4-layer-shell,
  pkg-config,
  librsvg,
  rustfmt,
  cargo,
  rustc,
  lockFile,
  ...
}: let
  inherit (builtins) fromTOML readFile;

  cargoToml = fromTOML (readFile ../Cargo.toml);
  pname = cargoToml.package.name;
  version = cargoToml.package.version;
in
  rustPlatform.buildRustPackage {
    inherit pname version;
    src = builtins.path {
      path = lib.sources.cleanSource inputs.self;
      name = "${pname}-${version}";
    };

    strictDeps = true;

    cargoLock = {
      inherit lockFile;
    };

    nativeBuildInputs = [
      pkg-config
      makeWrapper
      rustfmt
      rustc
      cargo
    ];

    buildInputs = [
      graphene
      gobject-introspection
      glib
      atk
      gtk4
      librsvg
      gtk4-layer-shell
    ];

    doCheck = true;
    checkInputs = [cargo rustc];

    copyLibs = true;

    CARGO_BUILD_INCREMENTAL = "false";
    RUST_BACKTRACE = "full";

    postInstall = ''
      wrapProgram $out/bin/closer-layer \
        --set GDK_PIXBUF_MODULE_FILE "$(echo ${librsvg.out}/lib/gdk-pixbuf-2.0/*/loaders.cache)" \
    '';

    meta = {
      description = "Close layer.";
      homepage = "https://github.com/psyvern";
      license = [lib.licenses.gpl3];
      mainProgram = "closer-layer";
      maintainers = with lib.maintainers; [NotAShelf n3oney];
    };
  }
