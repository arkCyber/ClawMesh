// Build script for lemmy_email
// Using simplified translations module instead of rosetta-i18n generated translations

fn main() {
  // No-op: We use translations_simple.rs instead of rosetta-i18n generated translations
  println!("cargo:rerun-if-changed=src/translations_simple.rs");
}
