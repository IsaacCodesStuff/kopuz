{ lib
, rustPlatform
, pkg-config
, wrapGAppsHook3
, makeWrapper
, webkitgtk_4_1
, gtk3
, libsoup_3
, glib-networking
, alsa-lib
, xdotool
, openssl
, nodejs
, src
}:

let
  pname = "rusic";
  version = "0.1.1";
in
rustPlatform.buildRustPackage {
  inherit pname version src;

  cargoLock = {
    lockFile = ./../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook3
    makeWrapper
    nodejs
  ];

  buildInputs = [
    webkitgtk_4_1
    gtk3
    libsoup_3
    glib-networking
    alsa-lib
    xdotool
    openssl
  ];

  cargoBuildFlags = [ "--package" "rusic" ];

  preBuild = ''
    # Build Tailwind CSS
    npm install
    npx @tailwindcss/cli -i ./tailwind.css -o ./rusic/assets/tailwind.css \
      --content './rusic/**/*.rs,./components/**/*.rs,./pages/**/*.rs,./hooks/**/*.rs,./player/**/*.rs,./reader/**/*.rs'
  '';

  postInstall = ''
    # Create assets directory next to binary
    mkdir -p $out/share/rusic/assets
    cp -r rusic/assets/* $out/share/rusic/assets/
    
    # Wrap binary to run from assets directory
    wrapProgram $out/bin/rusic \
      --chdir $out/share/rusic \
      --set GIO_MODULE_DIR "${glib-networking}/lib/gio/modules/"
    
    # Desktop file and icon
    mkdir -p $out/share/applications
    mkdir -p $out/share/icons/hicolor/scalable/apps
    cp data/com.temidaradev.rusic.desktop $out/share/applications/
    cp rusic/assets/logo.png $out/share/icons/hicolor/scalable/apps/com.temidaradev.rusic.png
    
    substituteInPlace $out/share/applications/com.temidaradev.rusic.desktop \
      --replace "Exec=rusic" "Exec=$out/bin/rusic"
  '';

  meta = with lib; {
    description = "A modern music player built with Dioxus";
    homepage = "https://github.com/temidaradev/rusic";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.linux;
    mainProgram = "rusic";
  };
}
