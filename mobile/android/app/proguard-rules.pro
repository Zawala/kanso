# The native library finds these by name through JNI, which shrinking cannot
# see.
-keep class app.kanso.kanso.KansoNative { *; }
-keep class app.kanso.kanso.KansoSecrets { *; }
# The certificate verifier's Kotlin component, called from Rust.
-keep, includedescriptorclasses class org.rustls.platformverifier.** { *; }
# The share activity and the widget are started by the system by name; the
# manifest keeps them, and captureInbox is covered by KansoNative's rule above.
