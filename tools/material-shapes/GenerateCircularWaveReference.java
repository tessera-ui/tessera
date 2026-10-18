/*
 * Copyright 2025 The Android Open Source Project
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
import java.io.*;
import java.nio.*;
import java.nio.file.*;
import java.util.*;

public class GenerateCircularWaveReference {
  static void writeCurves(DataOutputStream output, List<Cubic> curves) throws IOException {
    output.writeInt(Integer.reverseBytes(curves.size()));
    for (Cubic c : curves) {
      for (float v : new float[] {
        c.getAnchor0X(), c.getAnchor0Y(), c.getControl0X(), c.getControl0Y(),
        c.getControl1X(), c.getControl1Y(), c.getAnchor1X(), c.getAnchor1Y()
      }) output.writeInt(Integer.reverseBytes(Float.floatToRawIntBits(v)));
    }
  }

  public static void main(String[] args) throws Exception {
    try (DataOutputStream output = new DataOutputStream(Files.newOutputStream(Path.of(args[0])))) {
      output.writeBytes("MCW1");
      int[] vertices = {5, 6, 9, 12, 24};
      output.writeInt(Integer.reverseBytes(vertices.length));
      for (int n : vertices) {
        RoundedPolygon circle = ShapesKt.circle(RoundedPolygon.Companion, n, 1f, 0, 0).normalized();
        RoundedPolygon star = ShapesKt.star(RoundedPolygon.Companion, n, 1f, .75f,
            new CornerRounding(.35f, .4f), new CornerRounding(.5f, 0), null, 0, 0).normalized();
        Morph morph = new Morph(circle, star);
        output.writeInt(Integer.reverseBytes(n));
        writeCurves(output, circle.getCubics());
        for (float amplitude : new float[] {0, .25f, .5f, .75f, 1}) {
          writeCurves(output, morph.asCubics(amplitude));
        }
      }
    }
  }
}
