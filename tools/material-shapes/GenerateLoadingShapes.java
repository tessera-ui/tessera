/*
 * Copyright 2024 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import androidx.graphics.shapes.*;
import java.lang.reflect.*;
import java.nio.*;
import java.nio.file.*;
import java.util.*;

public class GenerateLoadingShapes {
  record P(float x, float y, float r) {}

  interface Transform {
    float[] apply(float x, float y);
  }

  static RoundedPolygon transform(RoundedPolygon polygon, Transform action) {
    PointTransformer transformer =
        (PointTransformer)
            Proxy.newProxyInstance(
                PointTransformer.class.getClassLoader(),
                new Class<?>[] {PointTransformer.class},
                (proxy, method, args) -> {
                  float[] p = action.apply((Float) args[0], (Float) args[1]);
                  return ((long) Float.floatToRawIntBits(p[0]) << 32)
                      | (Float.floatToRawIntBits(p[1]) & 0xffffffffL);
                });
    return polygon.transformed(transformer);
  }

  static RoundedPolygon rotate(RoundedPolygon polygon, float angle) {
    float a = angle * (float) Math.PI / 180f, c = (float) Math.cos(a), s = (float) Math.sin(a);
    return transform(polygon, (x, y) -> new float[] {x * c - y * s, x * s + y * c});
  }

  static float radians(float angle) {
    return angle / 360f * 2 * (float) Math.PI;
  }

  static RoundedPolygon custom(P[] points, int reps, boolean mirrored) {
    List<P> out = new ArrayList<>();
    if (mirrored) {
      int actualReps = reps * 2;
      float section = 360f / actualReps;
      float[] angles = new float[points.length], distances = new float[points.length];
      for (int i = 0; i < points.length; i++) {
        float x = points[i].x - .5f, y = points[i].y - .5f;
        angles[i] = (float) Math.atan2(y, x) * 180f / (float) Math.PI;
        distances[i] = (float) Math.sqrt(x * x + y * y);
      }
      for (int repeat = 0; repeat < actualReps; repeat++)
        for (int index = 0; index < points.length; index++) {
          int i = repeat % 2 == 0 ? index : points.length - 1 - index;
          if (i > 0 || repeat % 2 == 0) {
            float angle =
                radians(
                    section * repeat
                        + (repeat % 2 == 0 ? angles[i] : section - angles[i] + 2 * angles[0]));
            out.add(
                new P(
                    (float) Math.cos(angle) * distances[i] + .5f,
                    (float) Math.sin(angle) * distances[i] + .5f,
                    points[i].r));
          }
        }
    } else
      for (int i = 0; i < points.length * reps; i++) {
        P p = points[i % points.length];
        float angle = radians((i / points.length) * 360f / reps),
            c = (float) Math.cos(angle),
            s = (float) Math.sin(angle),
            x = p.x - .5f,
            y = p.y - .5f;
        out.add(new P(x * c - y * s + .5f, x * s + y * c + .5f, p.r));
      }
    float[] vertices = new float[out.size() * 2];
    List<CornerRounding> rounding = new ArrayList<>();
    for (int i = 0; i < out.size(); i++) {
      P p = out.get(i);
      vertices[i * 2] = p.x;
      vertices[i * 2 + 1] = p.y;
      rounding.add(new CornerRounding(p.r, 0f));
    }
    return RoundedPolygonKt.RoundedPolygon(vertices, CornerRounding.Unrounded, rounding, .5f, .5f)
        .normalized();
  }

  static float[] points(Cubic c) {
    return new float[] {
      c.getAnchor0X(),
      c.getAnchor0Y(),
      c.getControl0X(),
      c.getControl0Y(),
      c.getControl1X(),
      c.getControl1Y(),
      c.getAnchor1X(),
      c.getAnchor1Y()
    };
  }

  static float scale(List<RoundedPolygon> polygons) {
    float factor = 1;
    for (RoundedPolygon p : polygons) {
      float[] b = p.calculateBounds(), m = p.calculateMaxBounds(new float[4]);
      factor =
          Math.min(factor, Math.max((b[2] - b[0]) / (m[2] - m[0]), (b[3] - b[1]) / (m[3] - m[1])));
    }
    return factor * 38f / 48f;
  }

  public static void main(String[] args) throws Exception {
    Locale.setDefault(Locale.ROOT);
    RoundedPolygon burst =
        custom(new P[] {new P(.193f, .277f, .053f), new P(.176f, .055f, .053f)}, 10, false);
    RoundedPolygon cookie9 =
        rotate(
                ShapesKt.star(
                    RoundedPolygon.Companion,
                    9,
                    1f,
                    .8f,
                    new CornerRounding(.5f, 0),
                    null,
                    null,
                    0,
                    0),
                -90)
            .normalized();
    RoundedPolygon pentagon =
        custom(
            new P[] {
              new P(.5f, -.009f, .172f), new P(1.030f, .365f, .164f), new P(.828f, .970f, .169f)
            },
            1,
            true);
    RoundedPolygon pill =
        custom(
            new P[] {new P(.961f, .039f, .426f), new P(1.001f, .428f, 0), new P(1f, .609f, 1f)},
            2,
            true);
    RoundedPolygon sunny =
        ShapesKt.star(
                RoundedPolygon.Companion, 8, 1f, .8f, new CornerRounding(.15f, 0), null, null, 0, 0)
            .normalized();
    RoundedPolygon cookie4 =
        custom(new P[] {new P(1.237f, 1.236f, .258f), new P(.5f, .918f, .233f)}, 4, false);
    RoundedPolygon oval =
        rotate(
                transform(
                    ShapesKt.circle(RoundedPolygon.Companion, 8, 1f, 0, 0),
                    (x, y) -> new float[] {x, y * .64f}),
                -45)
            .normalized();
    List<RoundedPolygon> shapes = List.of(burst, cookie9, pentagon, pill, sunny, cookie4, oval);
    RoundedPolygon circle =
        rotate(ShapesKt.circle(RoundedPolygon.Companion, 10, 1f, 0, 0).normalized(), 18);
    List<RoundedPolygon> determinate = List.of(circle, burst);
    List<Morph> morphs = new ArrayList<>();
    for (int i = 0; i < 7; i++)
      morphs.add(new Morph(shapes.get(i).normalized(), shapes.get((i + 1) % 7).normalized()));
    morphs.add(new Morph(circle.normalized(), burst.normalized()));
    int total = morphs.stream().mapToInt(m -> m.asCubics(0).size()).sum();
    ByteBuffer bytes = ByteBuffer.allocate(8 + 8 * 12 + total * 64).order(ByteOrder.LITTLE_ENDIAN);
    bytes.putInt(0x31494c4d).putInt(8);
    int offset = 0;
    for (int i = 0; i < 8; i++) {
      int count = morphs.get(i).asCubics(0).size();
      bytes.putInt(offset).putInt(count).putFloat(scale(i < 7 ? shapes : determinate));
      offset += count;
      System.out.printf(
          "sequence %d: %d cubics, scale %.9f%n", i, count, scale(i < 7 ? shapes : determinate));
    }
    for (Morph morph : morphs) {
      List<Cubic> start = morph.asCubics(0), end = morph.asCubics(1);
      if (start.size() != end.size()) throw new IllegalStateException("mismatched cubics");
      for (int i = 0; i < start.size(); i++) {
        for (float v : points(start.get(i))) bytes.putFloat(v);
        for (float v : points(end.get(i))) bytes.putFloat(v);
      }
    }
    Files.write(Path.of(args[0]), bytes.array());
    StringBuilder svg =
        new StringBuilder(
            "<svg xmlns='http://www.w3.org/2000/svg' width='640' height='256' viewBox='0 0 640"
                + " 256'><rect width='640' height='256' fill='white'/>");
    for (int seq = 0; seq < 8; seq++)
      for (int step = 0; step < 3; step++) {
        float p = step * .5f;
        List<Cubic> cubics = morphs.get(seq).asCubics(p);
        float minX = Float.POSITIVE_INFINITY,
            minY = minX,
            maxX = Float.NEGATIVE_INFINITY,
            maxY = maxX;
        for (Cubic c : cubics) {
          float[] a = points(c);
          for (int i = 0; i < 8; i += 2) {
            minX = Math.min(minX, a[i]);
            maxX = Math.max(maxX, a[i]);
            minY = Math.min(minY, a[i + 1]);
            maxY = Math.max(maxY, a[i + 1]);
          }
        }
        float cx = (minX + maxX) / 2,
            cy = (minY + maxY) / 2,
            s = 64 * scale(seq < 7 ? shapes : determinate);
        svg.append(
            String.format(
                "<g transform='translate(%d %d)'><path fill='#6750a4' transform='scale(%.9f)"
                    + " translate(%.9f %.9f)' d='M %.9f %.9f ",
                40 + seq * 80,
                40 + step * 80,
                s,
                -cx,
                -cy,
                cubics.get(0).getAnchor0X(),
                cubics.get(0).getAnchor0Y()));
        for (Cubic c : cubics)
          svg.append(
              String.format(
                  "C %.9f %.9f %.9f %.9f %.9f %.9f ",
                  c.getControl0X(),
                  c.getControl0Y(),
                  c.getControl1X(),
                  c.getControl1Y(),
                  c.getAnchor1X(),
                  c.getAnchor1Y()));
        svg.append("Z'/></g>");
      }
    Files.writeString(Path.of(args[1]), svg.append("</svg>").toString());
  }
}
