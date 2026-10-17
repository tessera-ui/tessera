# Loading shape data

`GenerateLoadingShapes.java` regenerates the loading indicator's matched cubic
curves. This is an offline asset tool. Building or running Tessera does not need
Java, Kotlin, Maven, or an SVG parser for these shapes.

The shape definitions follow [MaterialShapes.kt](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MaterialShapes.kt),
inspected on 2026-10-17. Shape normalization and feature matching use the
official AndroidX Graphics Shapes 1.1.0 library. The first seven records form the
repeating sequence: soft burst, nine-sided cookie, pentagon, pill, sunny,
four-sided cookie, and oval. Record eight morphs an 18-degree rotated ten-vertex
circle into the soft burst for determinate progress.

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
java -cp "/external/dependencies/*" tools/material-shapes/GenerateLoadingShapes.java tessera-components/assets/material_loading_shapes.bin /external/qa/loading-shapes-reference.svg
```

The SVG shows each sequence at progress zero, one half, and one, without display
rotation. It is a reference artifact and is not included in the library.

The committed binary's SHA-256 is
`3ff5fdb60f54e57e1782fd82d1a2f3c914462e64a3383cdd3cfef8d77c7c64a7`.
Regeneration with the pinned dependencies must reproduce it byte for byte.

## Binary format

All numbers are little-endian. The file starts with `MLI1`, a 32-bit sequence
count of eight, and eight 12-byte records containing a 32-bit cubic offset,
32-bit cubic count, and 32-bit floating-point drawing scale. The remaining data
contains 322 matched cubic pairs. Each pair stores eight starting coordinates
and eight ending coordinates as 32-bit floats, in anchor/control/control/anchor
order. The header occupies 104 bytes; each pair occupies 64 bytes.

The drawing scale includes the 38/48 active-size ratio and the rotation-safe
bounds factor. Runtime shape interpolation, rotation, and tessellation run on
the GPU. Instance buffers are reused across frames and separately retained for
each draw batch to preserve queued writes.

See [the asset notice](../../tessera-components/assets/material_loading_shapes.NOTICE)
and [Apache 2.0 license](../../tessera-components/assets/material_loading_shapes.LICENSE).
