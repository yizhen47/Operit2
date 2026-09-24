import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:operit2/data/preferences/UserPreferencesManager.dart';
import 'package:operit2/ui/main/components/DrawerContent.dart';
import 'package:operit2/ui/main/components/DrawerConversationState.dart';
import 'package:operit2/ui/main/layout/PhoneLayout.dart';
import 'package:operit2/ui/theme/OperitTheme.dart';

/// Verifies drawer transitions preserve the mounted conversation list and input.
void main() {
  for (final enableNavigationAnimation in <bool>[true, false]) {
    testWidgets(
      'retains content build layout and paint during drawer frames (effects: $enableNavigationAnimation)',
      (tester) async {
        tester.view.physicalSize = const Size(400, 800);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final drawerOpen = ValueNotifier<bool>(false);
        final conversations = ValueNotifier<DrawerConversationState>(
          const DrawerConversationState(loading: false),
        );
        addTearDown(drawerOpen.dispose);
        addTearDown(conversations.dispose);
        final counts = _ContentCounts();
        await tester.pumpWidget(
          OperitTheme(
            initialThemePreferenceSnapshot:
                UserPreferencesManager.defaultThemePreferenceSnapshot,
            initialThemeIsReady: false,
            unconfiguredChildEnabled: true,
            hostInteractionHostsEnabled: false,
            child: Scaffold(
              body: PhoneLayout(
                content: _ContentProbe(counts: counts),
                navigationEntries: const [],
                pluginSidebarEntries: const [],
                selectedRouteId: 'ai_chat',
                drawerConversationState: conversations,
                drawerWidth: 300,
                drawerOpenState: drawerOpen,
                enableNavigationAnimation: enableNavigationAnimation,
                onOpenDrawer: () => drawerOpen.value = true,
                onCloseDrawer: () => drawerOpen.value = false,
                onNavigationEntrySelected: (_) {},
                onConversationActivated: () {},
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        final baseline = (counts.builds, counts.layouts, counts.paints);
        expect(baseline.$1, greaterThan(0));
        expect(baseline.$2, greaterThan(0));
        expect(baseline.$3, greaterThan(0));
        final contentElement = tester.element(find.byType(_ContentProbe));
        for (var cycle = 0; cycle < 3; cycle++) {
          await tester.dragFrom(const Offset(20, 400), const Offset(100, 0));
          expect(drawerOpen.value, isTrue);
          for (var frame = 0; frame < 40; frame++) {
            await tester.pump(const Duration(milliseconds: 16));
            expect((counts.builds, counts.layouts, counts.paints), baseline);
          }
          await tester.tapAt(const Offset(350, 400));
          expect(drawerOpen.value, isFalse);
          for (var frame = 0; frame < 40; frame++) {
            await tester.pump(const Duration(milliseconds: 16));
            expect((counts.builds, counts.layouts, counts.paints), baseline);
          }
          expect(
            tester.element(find.byType(_ContentProbe)),
            same(contentElement),
          );
        }
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox.shrink());
      },
    );
    testWidgets(
      'preserves drawer state across transitions (effects: $enableNavigationAnimation)',
      (tester) async {
        final drawerOpen = ValueNotifier<bool>(false);
        final conversations = ValueNotifier<DrawerConversationState>(
          const DrawerConversationState(loading: false),
        );
        addTearDown(drawerOpen.dispose);
        addTearDown(conversations.dispose);

        await tester.pumpWidget(
          OperitTheme(
            initialThemePreferenceSnapshot:
                UserPreferencesManager.defaultThemePreferenceSnapshot,
            initialThemeIsReady: false,
            unconfiguredChildEnabled: true,
            hostInteractionHostsEnabled: false,
            child: Scaffold(
              body: PhoneLayout(
                content: const SizedBox.expand(),
                navigationEntries: const [],
                pluginSidebarEntries: const [],
                selectedRouteId: 'ai_chat',
                drawerConversationState: conversations,
                drawerWidth: 300,
                drawerOpenState: drawerOpen,
                enableNavigationAnimation: enableNavigationAnimation,
                onOpenDrawer: () => drawerOpen.value = true,
                onCloseDrawer: () => drawerOpen.value = false,
                onNavigationEntrySelected: (_) {},
                onConversationActivated: () {},
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        final drawerState = tester.state(find.byType(DrawerContent));

        drawerOpen.value = true;
        await tester.pumpAndSettle();
        expect(tester.state(find.byType(DrawerContent)), same(drawerState));

        await tester.tap(find.byTooltip('搜索对话'));
        await tester.pumpAndSettle();
        await tester.enterText(find.byType(TextField), 'retained query');
        await tester.pumpAndSettle();

        for (var cycle = 0; cycle < 3; cycle++) {
          drawerOpen.value = false;
          await tester.pumpAndSettle();
          expect(tester.state(find.byType(DrawerContent)), same(drawerState));

          drawerOpen.value = true;
          await tester.pumpAndSettle();
          expect(tester.state(find.byType(DrawerContent)), same(drawerState));
          expect(find.text('retained query'), findsOneWidget);
        }
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox.shrink());
      },
    );
  }
}

class _ContentCounts {
  int builds = 0;
  int layouts = 0;
  int paints = 0;
}

class _ContentProbe extends StatelessWidget {
  /// Creates a content probe with independently counted rendering phases.
  const _ContentProbe({required this.counts});

  final _ContentCounts counts;

  /// Counts content builds independently of the drawer animation builder.
  @override
  Widget build(BuildContext context) {
    counts.builds++;
    return _ContentRenderProbe(counts: counts);
  }
}

class _ContentRenderProbe extends LeafRenderObjectWidget {
  /// Creates a render probe for the retained content layer.
  const _ContentRenderProbe({required this.counts});

  final _ContentCounts counts;

  /// Creates the render box that records layout and paint calls.
  @override
  RenderObject createRenderObject(BuildContext context) => _ContentBox(counts);
}

class _ContentBox extends RenderBox {
  /// Creates a static content surface backed by shared counters.
  _ContentBox(this.counts);

  final _ContentCounts counts;

  /// Records each content layout under the phone viewport constraints.
  @override
  void performLayout() {
    counts.layouts++;
    size = constraints.biggest;
  }

  /// Records each content repaint and draws an opaque surface.
  @override
  void paint(PaintingContext context, Offset offset) {
    counts.paints++;
    context.canvas.drawRect(offset & size, Paint()..color = Colors.white);
  }
}
