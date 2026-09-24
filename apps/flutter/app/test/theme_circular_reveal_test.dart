import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:operit2/ui/theme/ThemeCircularRevealHost.dart';

/// Verifies the sidebar theme switch uses a circular hole to the bottom-right.
void main() {
  test(
    'max radius is the distance from the origin to the bottom-right corner',
    () {
      const origin = Offset(48, 72);
      const size = Size(400, 800);
      expect(
        ThemeCircularRevealHost.maxRadius(origin: origin, size: size),
        (Offset(size.width, size.height) - origin).distance,
      );
    },
  );

  test('animated radius overshoots the bottom-right by the feather width', () {
    const origin = Offset(40, 60);
    const size = Size(400, 800);
    final maxRadius = ThemeCircularRevealHost.maxRadius(
      origin: origin,
      size: size,
    );
    final feather = ThemeCircularRevealHost.featherFor(maxRadius);
    expect(feather, inInclusiveRange(72.0, 200.0));
    expect(
      ThemeCircularRevealHost.animatedRadius(
        progress: 0,
        maxRadius: maxRadius,
        feather: feather,
      ),
      0,
    );
    expect(
      ThemeCircularRevealHost.animatedRadius(
        progress: 1,
        maxRadius: maxRadius,
        feather: feather,
      ),
      maxRadius + feather,
    );
  });

  testWidgets('theme switch captures the frame and reveals from the button', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(400, 800);
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);

    var dark = false;
    final recorder = ui.PictureRecorder();
    final canvas = Canvas(recorder);
    canvas.drawColor(const Color(0xFFFFFFFF), BlendMode.src);
    final snapshot = recorder.endRecording().toImageSync(8, 8);
    ThemeCircularRevealHost.debugCaptureFrame = (boundary, pixelRatio) async =>
        snapshot;
    addTearDown(() {
      ThemeCircularRevealHost.debugCaptureFrame = null;
    });

    await tester.pumpWidget(
      MaterialApp(
        home: ThemeCircularRevealHost(
          child: StatefulBuilder(
            builder: (context, setState) {
              return ColoredBox(
                color: dark ? Colors.black : Colors.white,
                child: Align(
                  alignment: Alignment.topLeft,
                  child: Builder(
                    builder: (buttonContext) {
                      return IconButton(
                        tooltip: 'toggle',
                        onPressed: () {
                          unawaited(
                            ThemeCircularRevealHost.maybeOf(
                              buttonContext,
                            )!.switchTheme(
                              originContext: buttonContext,
                              applyTheme: () async {
                                setState(() {
                                  dark = true;
                                });
                              },
                            ),
                          );
                        },
                        icon: const Icon(Icons.dark_mode_outlined),
                      );
                    },
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );

    await tester.tap(find.byTooltip('toggle'));
    await tester.pump();
    expect(
      find.byKey(const ValueKey<String>('themeCircularRevealOverlay')),
      findsOneWidget,
    );
    expect(find.byType(ShaderMask), findsOneWidget);

    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey<String>('themeCircularRevealOverlay')),
      findsNothing,
    );
    expect(dark, isTrue);
  });
}
