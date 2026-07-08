use std::{env, path::PathBuf};

use cxx_qt_build::{CxxQtBuilder, PluginType, QmlModule};

fn main() {
    let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();

    let builder =
        CxxQtBuilder::new_qml_module(QmlModule::new("bunpro").plugin_type(PluginType::Dynamic))
            .files(["src/lib.rs", "src/qtlogging.rs"])
            .cpp_files(["src/qtlogging.cpp"])
            .include_dir(manifest_dir.join("includes/"));

    builder.build();

    // https://github.com/KDAB/cxx-qt/issues/1433
    let version_script = manifest_dir.join("qt-plugin.version");
    println!(
        "cargo::rustc-link-arg-cdylib=-Wl,--version-script={}",
        version_script.display()
    );
}
