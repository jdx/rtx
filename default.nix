{ pkgs, lib, stdenv, fetchFromGitHub, rustPlatform, coreutils, bash, direnv, perl, installShellFiles }:
rustPlatform.buildRustPackage {
  pname = "rtx";
  version = "1.32.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ installShellFiles ];

  buildInputs = with pkgs; [
    coreutils
    bash
    direnv
    gnused
    git
    gawk
  ] ++ lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security;

  prePatch = ''
    substituteInPlace ./test/data/plugins/**/bin/* \
      --replace '#!/usr/bin/env bash' '#!${bash}/bin/bash'
    substituteInPlace ./src/fake_asdf.rs ./src/cli/reshim.rs \
      --replace '#!/bin/sh' '#!${bash}/bin/sh'
    substituteInPlace ./src/env_diff.rs \
      --replace '"bash"' '"${bash}/bin/bash"'
    substituteInPlace ./src/cli/direnv/exec.rs \
      --replace '"env"' '"${coreutils}/bin/env"' \
      --replace 'cmd!("direnv"' 'cmd!("${direnv}/bin/direnv"'
  '';

  # Skip the test_plugin_list_urls as it uses the .git folder, which
  # is excluded by default from Nix.
  checkPhase = ''
    RUST_BACKTRACE=full cargo test --all-features -- \
      --skip cli::plugins::ls::tests::test_plugin_list_urls
  '';

  postInstall = ''
      installManPage man/man1/rtx.1

      installShellCompletion --cmd rtx \
        --bash <($out/bin/rtx completion bash) \
        --fish <($out/bin/rtx completion fish) \
        --zsh <($out/bin/rtx completion zsh)
  '';

  # Need this to ensure openssl-src's build uses an available version of `perl`
  # https://github.com/alexcrichton/openssl-src-rs/issues/45
  OPENSSL_SRC_PERL = "${perl}/bin/perl";

  meta = with lib; {
    description = "Polyglot runtime manager (asdf rust clone)";
    homepage = "https://github.com/jdxcode/rtx";
    license = licenses.mit;
  };
}
