import org.gradle.api.provider.ValueSource
import org.gradle.api.provider.ValueSourceParameters

plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
    id("org.jetbrains.kotlin.plugin.compose")
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

    // The JVM unit tests run with TestNG (see the dependencies below).
    testOptions {
        unitTests.all { it.useTestNG() }
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
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.getByName("debug")
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
    // The home-screen widget. Jetpack Glance, Apache-2.0.
    implementation("androidx.glance:glance-appwidget:1.1.1")

    // JVM unit tests for the drop writer, run with TestNG (Apache-2.0) and
    // kotlin.test's assertions (Apache-2.0); neither is part of the app.
    testImplementation("org.testng:testng:7.10.2")
    testImplementation("org.jetbrains.kotlin:kotlin-test-testng:2.3.20")
    // org.json for the JVM (the platform's stubs do nothing in unit tests); Apache-2.0.
    testImplementation("com.vaadin.external.google:android-json:0.0.20131108.vaadin1")
}

flutter {
    source = "../.."
}
