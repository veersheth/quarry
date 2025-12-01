{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    cargo-tauri

    # Node.js and pnpm
    nodejs_20
    pnpm_9

    # GTK and related libraries (runtime)
    glib
    gtk3
    webkitgtk_4_1
    libsoup_3
    libappindicator

    # Additional GTK dependencies
    gdk-pixbuf
    cairo
    pango
    harfbuzz
    librsvg
    atk
    at-spi2-atk
    gobject-introspection
    pkg-config
    openssl
  ];

  shellHook = ''
    # Make pkg-config aware of .dev outputs
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.glib.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.gtk3.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.cairo.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.pango.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.gdk-pixbuf.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.atk.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.webkitgtk_4_1.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.libsoup_3.dev}/lib/pkgconfig"
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.openssl.dev}/lib/pkgconfig"

    # Ensure runtime can find libappindicator
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.libappindicator}/lib"

    echo "Tauri development environment loaded!"
    echo "You can now run: pnpm tauri dev"
  '';
}

