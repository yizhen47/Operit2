# Operit integration

Based on webview_flutter_wkwebview 3.25.1, retaining its upstream license.

The common `operit/webview_theme` channel updates native WKWebView appearance.
The weak browser registry applies the preference to existing and newly created
views on iOS and macOS. It does not override application/window appearance, so
Flutter continues observing system theme changes independently.

`WebKitWebViewController.setZoomFactor` is backed by the native
`WKWebView.pageZoom` property through the `operit/webview_zoom` channel, which
keeps page zoom available before navigation on macOS (with the legacy
`magnification` fallback on macOS 10.15) and iOS 14+.

On macOS, scroll position and scrollbar visibility use WebKit page JavaScript
and user scripts because `WKWebView` has no iOS-style `UIScrollView` bridge.

The application mounts macOS `AppKitView` browser surfaces directly in their
final workspace location. It does not transfer them from the offscreen owner
host or animate their enclosing workspace panel, since either operation can
produce a transient blank frame in AppKit.
