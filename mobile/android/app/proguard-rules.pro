# The native library finds these by name through JNI, which shrinking cannot
# see.
-keep class app.kanso.kanso.KansoNative { *; }
-keep class app.kanso.kanso.KansoSecrets { *; }
# The certificate verifier's Kotlin component, called from Rust.
-keep, includedescriptorclasses class org.rustls.platformverifier.** { *; }
