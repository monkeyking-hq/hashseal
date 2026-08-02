// HashSeal Gradle plugin (composite: apply from this build or publish locally).
// Copyright (c) 2026 MonkeyKing.dev

plugins {
    `java-gradle-plugin`
    `maven-publish`
}

group = "ai.hashseal"
version = "0.1.0-SNAPSHOT"

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

repositories {
    mavenCentral()
}

gradlePlugin {
    plugins {
        create("hashseal") {
            id = "ai.hashseal"
            implementationClass = "ai.hashseal.gradle.HashsealPlugin"
            displayName = "HashSeal"
            description = "Thin HashSeal Gradle plugin (shells to hashseal CLI)"
        }
    }
}

// No runtime deps beyond Gradle API — CLI is external.
