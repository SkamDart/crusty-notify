with import <nixpkgs> {};

let
  crust = (rustChannels.nightly.rust.override { extensions = [ "rust-src" ]; });
in
stdenv.mkDerivation {
  name = "oxidize";
  buildInputs = [ crust rustracer ];
  RUST_SRC_PATH= "${crust}/lib/rustlib/src/rust/src";
}
