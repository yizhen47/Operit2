// GENERATED FILE. Source: operit-proxy-scan.

import 'client.dart';
import 'models.dart';

/// Generated client for Core object `application`.
final class OperitApplicationClient {
  const OperitApplicationClient(this._client);
  final OperitPluginSdkClient _client;
  /// Watches `pluginLoadingProgressFlow` through the Core Link route.
  Stream<PluginLoadingProgress> pluginLoadingProgressFlow() => _client.watchTyped<PluginLoadingProgress>(0, 'pluginLoadingProgressFlow', <String, Object?>{}, (value) => PluginLoadingProgress.fromMessagePackValue(value as Map<String, Object?>));
}

/// Generated client for Core object `application.packageManager`.
final class OperitApplicationPackageManagerClient {
  const OperitApplicationPackageManagerClient(this._client);
  final OperitPluginSdkClient _client;
  /// Calls `activatePackage` through the Core Link route.
  Future<bool> activatePackage({required String packageName}) async { return await _client.callTyped<bool>(5, 'activatePackage', <String, Object?>{'packageName': packageName}, (value) => value as bool); }
  /// Calls `isPackageActivated` through the Core Link route.
  Future<bool> isPackageActivated({required String packageName}) async { return await _client.callTyped<bool>(5, 'isPackageActivated', <String, Object?>{'packageName': packageName}, (value) => value as bool); }
  /// Calls `usePackage` through the Core Link route.
  Future<String> usePackage({required String packageName}) async { return await _client.callTyped<String>(5, 'usePackage', <String, Object?>{'packageName': packageName}, (value) => value as String); }
  /// Calls `executeUsePackageTool` through the Core Link route.
  Future<ToolResult> executeUsePackageTool({required String toolName, required String packageName}) async { return await _client.callTyped<ToolResult>(5, 'executeUsePackageTool', <String, Object?>{'toolName': toolName, 'packageName': packageName}, (value) => ToolResult.fromMessagePackValue(value as Map<String, Object?>)); }
  /// Calls `getEnabledPackageNames` through the Core Link route.
  Future<List<String>> getEnabledPackageNames() async { return await _client.callTyped<List<String>>(5, 'getEnabledPackageNames', <String, Object?>{}, (value) => (value as List<Object?>).map((item) => item as String).toList(growable: false)); }
  /// Calls `isPackageEnabled` through the Core Link route.
  Future<bool> isPackageEnabled({required String packageName}) async { return await _client.callTyped<bool>(5, 'isPackageEnabled', <String, Object?>{'packageName': packageName}, (value) => value as bool); }
  /// Calls `getActivePackageNames` through the Core Link route.
  Future<List<String>> getActivePackageNames() async { return await _client.callTyped<List<String>>(5, 'getActivePackageNames', <String, Object?>{}, (value) => (value as List<Object?>).map((item) => item as String).toList(growable: false)); }
  /// Calls `enablePackage` through the Core Link route.
  Future<String> enablePackage({required String packageName}) async { return await _client.callTyped<String>(5, 'enablePackage', <String, Object?>{'packageName': packageName}, (value) => value as String); }
  /// Calls `disablePackage` through the Core Link route.
  Future<String> disablePackage({required String packageName}) async { return await _client.callTyped<String>(5, 'disablePackage', <String, Object?>{'packageName': packageName}, (value) => value as String); }
  /// Calls `getToolPkgPluginContainerDetails` through the Core Link route.
  Future<List<ToolPkgContainerDetails>> getToolPkgPluginContainerDetails({required bool useEnglish}) async { return await _client.callTyped<List<ToolPkgContainerDetails>>(5, 'getToolPkgPluginContainerDetails', <String, Object?>{'useEnglish': useEnglish}, (value) => (value as List<Object?>).map((item) => ToolPkgContainerDetails.fromMessagePackValue(item as Map<String, Object?>)).toList(growable: false)); }
  /// Calls `getToolPkgContainerRuntimes` through the Core Link route.
  Future<List<ToolPkgContainerRuntime>> getToolPkgContainerRuntimes() async { return await _client.callTyped<List<ToolPkgContainerRuntime>>(5, 'getToolPkgContainerRuntimes', <String, Object?>{}, (value) => (value as List<Object?>).map((item) => ToolPkgContainerRuntime.fromMessagePackValue(item as Map<String, Object?>)).toList(growable: false)); }
  /// Calls `getToolPkgContainerOrder` through the Core Link route.
  Future<List<String>> getToolPkgContainerOrder() async { return await _client.callTyped<List<String>>(5, 'getToolPkgContainerOrder', <String, Object?>{}, (value) => (value as List<Object?>).map((item) => item as String).toList(growable: false)); }
  /// Calls `setToolPkgContainerOrder` through the Core Link route.
  Future<void> setToolPkgContainerOrder({required List<String> packageNames}) async { await _client.call(5, 'setToolPkgContainerOrder', <String, Object?>{'packageNames': packageNames.map((item) => item).toList(growable: false)}); }
  /// Calls `getToolPkgContainerDetails` through the Core Link route.
  Future<ToolPkgContainerDetails?> getToolPkgContainerDetails({required String packageName, required bool useEnglish}) async { return await _client.callTyped<ToolPkgContainerDetails?>(5, 'getToolPkgContainerDetails', <String, Object?>{'packageName': packageName, 'useEnglish': useEnglish}, (value) => value == null ? null : ToolPkgContainerDetails.fromMessagePackValue(value as Map<String, Object?>)); }
  /// Calls `readToolPkgLogoBytes` through the Core Link route.
  Future<ToolPkgLogoBytes?> readToolPkgLogoBytes({required String packageName}) async { return await _client.callTyped<ToolPkgLogoBytes?>(5, 'readToolPkgLogoBytes', <String, Object?>{'packageName': packageName}, (value) => value == null ? null : ToolPkgLogoBytes.fromMessagePackValue(value as Map<String, Object?>)); }
  /// Calls `getEffectivePackageTools` through the Core Link route.
  Future<ToolPackage?> getEffectivePackageTools({required String packageName}) async { return await _client.callTyped<ToolPackage?>(5, 'getEffectivePackageTools', <String, Object?>{'packageName': packageName}, (value) => value == null ? null : ToolPackage.fromMessagePackValue(value as Map<String, Object?>)); }
  /// Calls `getPackageTools` through the Core Link route.
  Future<ToolPackage?> getPackageTools({required String packageName}) async { return await _client.callTyped<ToolPackage?>(5, 'getPackageTools', <String, Object?>{'packageName': packageName}, (value) => value == null ? null : ToolPackage.fromMessagePackValue(value as Map<String, Object?>)); }
  /// Calls `getAvailablePackages` through the Core Link route.
  Future<Map<String, ToolPackage>> getAvailablePackages() async { return await _client.callTyped<Map<String, ToolPackage>>(5, 'getAvailablePackages', <String, Object?>{}, (value) => (value as Map).map((key, item) => MapEntry(key as String, ToolPackage.fromMessagePackValue(item as Map<String, Object?>)))); }
  /// Calls `getToolPkgLoadIssues` through the Core Link route.
  Future<List<ToolPkgLoadIssue>> getToolPkgLoadIssues() async { return await _client.callTyped<List<ToolPkgLoadIssue>>(5, 'getToolPkgLoadIssues', <String, Object?>{}, (value) => (value as List<Object?>).map((item) => ToolPkgLoadIssue.fromMessagePackValue(item as Map<String, Object?>)).toList(growable: false)); }
}

