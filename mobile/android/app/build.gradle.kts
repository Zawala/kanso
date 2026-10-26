import java.util.Properties
import org.gradle.api.provider.ValueSource
import org.gradle.api.provider.ValueSourceParameters

plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

/**
 * The version of the Rust crate `rustls-platform-verifier-android` in
 * Cargo.lock. The Kotlin component must be exactly that version (the crate's
 * README), so Cargo.lock decides.
 */
abstract class RustlsVersion : ValueSource<String, RustlsVersion.Params> {
    interface Params : ValueSourceParameters {
        val lockFile: RegularFileProperty
    }

    companion object {
        const val CRATE_NAME = "rustls-platform-verifier-android"
    }

    override fun obtain(): String {
        val lines = parameters.lockFile.get().asFile.readLines()
        val name = lines.indexOfFirst { it.trim() == "name = \"$CRATE_NAME\"" }
        if (name < 0) {
            error("$CRATE_NAME not found in Cargo.lock")
        }
        return lines
            .drop(name + 1)
            .firstOrNull { it.trimStart().startsWith("version = ") }
            ?.substringAfter('"', "")
            ?.substringBefore('"', "")
            ?.takeIf { it.isNotEmpty() }
            ?: error("$CRATE_NAME has no version in Cargo.lock")
    }
}

/**
 * The upload key of the Play release, from android/key.properties (never
 * committed: .gitignore lists it). CI writes the file from the release
 * environment's secrets just before the build (docs/RELEASING-MOBILE.md); a
 * checkout without one signs release builds with the debug key, so
 * `flutter run --release` works for everyone.
 */
val uploadKeyFile = rootProject.file("key.properties")
val uploadKey =
    Properties().apply {
        if (uploadKeyFile.exists()) {
            uploadKeyFile.inputStream().use { load(it) }
        }
    }

val rustlsPlatformVerifierVersion =
    providers.of(RustlsVersion::class.java) {
        // mobile/android/app -> the repository's Cargo.lock.
        parameters.lockFile.set(layout.projectDirectory.file("../../../Cargo.lock"))
    }

configurations.configureEach {
    resolutionStrategy.eachDependency {
        if (requested.group == "org.rustls" && requested.name == "rustls-platform-verifier") {
            useVersion(rustlsPlatformVerifierVersion.get())
            because("it must be the version of ${RustlsVersion.CRATE_NAME}")
        }
    }
}

android {
    namespace = "app.kanso.kanso"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        applicationId = "app.kanso.kanso"
        minSdk = 29
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        if (uploadKeyFile.exists()) {
            create("upload") {
                // A key.properties that is missing a line fails here, by name,
                // rather than signing with the wrong key.
                fun line(name: String) =
                    uploadKey.getProperty(name)
                        ?: error("${uploadKeyFile.path} has no $name")
                keyAlias = line("keyAlias")
                keyPassword = line("keyPassword")
                storeFile = file(line("storeFile"))
                storePassword = line("storePassword")
            }
        }
    }

    buildTypes {
        release {
            // Shrinking removes what it can't see is used; the rules keep
            // what the native library calls by name.
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            signingConfig =
                signingConfigs.findByName("upload") ?: signingConfigs.getByName("debug")
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

dependencies {
    // The Rust crate rustls-platform-verifier has a Kotlin part; its version
    // comes from Cargo.lock (above).
    implementation("org.rustls:rustls-platform-verifier")
}

flutter {
    source = "../.."
}

// Push on Android needs the Firebase project's google-services.json in this
// folder (README.md). Without it the build is the same, and push is simply
// unavailable on the phone.
if (file("google-services.json").exists()) {
    apply(plugin = "com.google.gms.google-services")
}
