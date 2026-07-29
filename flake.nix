{
  description = "Quarry — Tauri development shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forEachSystem = fn: nixpkgs.lib.genAttrs systems (system: fn nixpkgs.legacyPackages.${system});
    in {
      devShells = forEachSystem (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # for camera permissions
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            gst_all_1.gst-plugins-good
            gst_all_1.gst-plugins-bad

            # Rust toolchain
            cargo
            rustc
            cargo-tauri

            # Node.js and pnpm
            nodejs_24
            pnpm

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
            alsa-lib
            tesseract
          ];

          shellHook = ''
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.glib.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.gtk3.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.cairo.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.pango.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.gdk-pixbuf.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.atk.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.webkitgtk_4_1.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.libsoup_3.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.openssl.dev}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.alsa-lib.dev}/lib/pkgconfig"

            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.libappindicator}/lib"

            export GST_PLUGIN_PATH="${pkgs.gst_all_1.gstreamer}/lib/gstreamer-1.0:${pkgs.gst_all_1.gst-plugins-base}/lib/gstreamer-1.0:${pkgs.gst_all_1.gst-plugins-good}/lib/gstreamer-1.0:${pkgs.gst_all_1.gst-plugins-bad}/lib/gstreamer-1.0"

            echo "Tauri development environment loaded!"
            echo "You can now run: pnpm tauri dev"
          '';
        };
      });
    };
}
