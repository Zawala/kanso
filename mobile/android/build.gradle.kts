allprojects {
    repositories {
        google()
        mavenCentral()
        // The certificate verifier's Kotlin component (the Rust crate
        // rustls-platform-verifier needs it on Android). The crate's README
        // says to fetch it from this repository; app/build.gradle.kts picks
        // the version that Cargo.lock has.
        maven {
            url =
                uri(
                    "https://github.com/rustls/rustls-platform-verifier/raw/maven-archive/android-release-support/maven/",
                )
            content { includeGroup("org.rustls") }
        }
    }
}

val newBuildDir: Directory =
    rootProject.layout.buildDirectory
        .dir("../../build")
        .get()
rootProject.layout.buildDirectory.value(newBuildDir)

subprojects {
    val newSubprojectBuildDir: Directory = newBuildDir.dir(project.name)
    project.layout.buildDirectory.value(newSubprojectBuildDir)
}
subprojects {
    project.evaluationDependsOn(":app")
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
