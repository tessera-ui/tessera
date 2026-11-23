# Maintaining generated shape assets

These offline tools regenerate shape assets used by Tessera components; they are
not needed to build or run Tessera.

The loading shapes follow [MaterialShapes.kt](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MaterialShapes.kt).
The source material is credited in the adjacent asset notices.

## Regeneration

Use JDK 21 and place these dependencies in a directory outside the repository:

| Filename | Download | SHA-256 |
| --- | --- | --- |
| `graphics-shapes.jar` | [Graphics Shapes Desktop 1.1.0](https://dl.google.com/dl/android/maven2/androidx/graphics/graphics-shapes-desktop/1.1.0/graphics-shapes-desktop-1.1.0.jar) | `8d7a4d3812cae2e3ef8272e97f6e9917ae45931f17cf1391f0246a92c955d9f4` |
| `kotlin-stdlib.jar` | [Kotlin Standard Library 2.0.21](https://repo.maven.apache.org/maven2/org/jetbrains/kotlin/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar) | `f31cc53f105a7e48c093683bbd5437561d1233920513774b470805641bedbc09` |
| `collection.jar` | [AndroidX Collection JVM 1.5.0](https://dl.google.com/dl/android/maven2/androidx/collection/collection-jvm/1.5.0/collection-jvm-1.5.0.jar) | `70b35924e4babcdffa37d0e575ee039c56a2d97123342624c48b603233704341` |
| `annotation.jar` | [AndroidX Annotation JVM 1.9.1](https://dl.google.com/dl/android/maven2/androidx/annotation/annotation-jvm/1.9.1/annotation-jvm-1.9.1.jar) | `1e343917ebf27ba96fe4dc52b1cad7fd32b738fbc6355bb6cd5b3b305d7212d0` |

From the repository root, replace both `/external/...` paths and run:

```sh
java -cp "/external/dependencies/*" tools/material-shapes/GenerateLoadingShapes.java crates/tessera-components/assets/material_loading_shapes.bin /external/qa/loading-shapes-reference.svg
```

The committed binary's SHA-256 is
`3ff5fdb60f54e57e1782fd82d1a2f3c914462e64a3383cdd3cfef8d77c7c64a7`.
Regeneration with the pinned dependencies must reproduce it byte for byte.

See [the asset notice](../../crates/tessera-components/assets/material_loading_shapes.NOTICE)
and [Apache 2.0 license](../../crates/tessera-components/assets/material_loading_shapes.LICENSE).

## Circular wave validation data

`GenerateCircularWaveReference.java` uses the same pinned dependencies.

```sh
java -cp "/external/dependencies/*" tools/material-shapes/GenerateCircularWaveReference.java crates/tessera-components/assets/circular_wave_reference.bin
```

The fixture's SHA-256 is
`2f3f39249b5ca45deaa4683f46c29411f21a8e592d3a047a63418918418881c3`.
Regeneration with the pinned dependencies must reproduce it byte for byte.

The Rust port's attribution is in
[shapes.NOTICE](../../crates/tessera-components/src/md3/wavy_progress/shapes.NOTICE).
