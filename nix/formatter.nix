{ pkgs }:
pkgs.writeShellApplication {
  name = "vessel-format";
  runtimeInputs = [
    pkgs.nixfmt
    pkgs.rustfmt
  ];
  text = ''
    paths=("$@")
    if [ "''${#paths[@]}" -eq 0 ]; then
      paths=(.)
    fi

    for path in "''${paths[@]}"; do
      if [ -f "$path" ] && [[ "$path" == *.nix ]]; then
        nixfmt "$path"
      elif [ -d "$path" ]; then
        while IFS= read -r -d "" file; do
          nixfmt "$file"
        done < <(find "$path" -type f -name '*.nix' -print0)
      fi
    done

    cargo fmt --all
  '';
}
