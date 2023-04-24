{ lib, pkgs, stdenv }:

let
  pname = "spin";
  version = "1.1.0";
  tmp_folder = "${pname}-${version}";
  # other OS are not supported by spin
  os = if stdenv.isDarwin then "macos" else "linux";
  # other arch is not supported by spin
  arch = if stdenv.isAarch64 then "aarch64" else "amd64";
  ref = https://github.com/fermyon/spin;
in
stdenv.mkDerivation rec {
  inherit pname version;

  # https://nixos.wiki/wiki/Packaging/Binaries
  src = pkgs.fetchurl {
    url = "${ref}/releases/download/v${version}/${pname}-v${version}-${os}-${arch}.tar.gz";
    sha256 = "sha256-pTv1MXUc01LdQS5mLHzMteUMW8ZEsMUu/eWa1ETCY9c=";
  };

  # see https://stackoverflow.com/questions/57225745/how-to-disable-unpack-phase-to-prevent-error-do-not-know-how-to-unpack-source-a
  phases = [ "unpackPhase" "installPhase" ];

  unpackPhase = ''
    runHook preUnpack
    mkdir ${tmp_folder}
    tar -C ${tmp_folder} -xzf $src
    runHook postUnpack
  '';

  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    cp ${tmp_folder}/spin $out/bin
    runHook postInstall
  '';

  meta = with lib; {
    homepage = ref;
    platforms = [ "x86_64-darwin" "aarch64-darwin" "x86_64-linux" "aarch64-linux" ];
  };
}

