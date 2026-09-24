// GENERATED FILE. Source: operit-proxy-scan.

import { OperitPluginSdkClient, OperitPluginSdkEvent, OperitPluginSdkPushSink } from './client.js';
import * as models from './models.js';

/** Generated client for Core object `application`. */
export class OperitApplicationClient {
  public constructor(private readonly client: OperitPluginSdkClient) {}
  /** Watches `pluginLoadingProgressFlow` through the Core Link route. */
  public pluginLoadingProgressFlow(): AsyncIterable<models.PluginLoadingProgress> { return this.client.watchTyped<models.PluginLoadingProgress>(0, 'pluginLoadingProgressFlow', {}, (value) => models.decodePluginLoadingProgress(value)); }
}

/** Generated client for Core object `application.packageManager`. */
export class OperitApplicationPackageManagerClient {
  public constructor(private readonly client: OperitPluginSdkClient) {}
  /** Calls `activatePackage` through the Core Link route. */
  public activatePackage(packageName: string): Promise<boolean> { return this.client.callTyped<boolean>(5, 'activatePackage', {'packageName': packageName}, (value) => value as boolean); }
  /** Calls `isPackageActivated` through the Core Link route. */
  public isPackageActivated(packageName: string): Promise<boolean> { return this.client.callTyped<boolean>(5, 'isPackageActivated', {'packageName': packageName}, (value) => value as boolean); }
  /** Calls `usePackage` through the Core Link route. */
  public usePackage(packageName: string): Promise<string> { return this.client.callTyped<string>(5, 'usePackage', {'packageName': packageName}, (value) => value as string); }
  /** Calls `executeUsePackageTool` through the Core Link route. */
  public executeUsePackageTool(toolName: string, packageName: string): Promise<models.ToolResult> { return this.client.callTyped<models.ToolResult>(5, 'executeUsePackageTool', {'toolName': toolName, 'packageName': packageName}, (value) => models.decodeToolResult(value)); }
  /** Calls `getEnabledPackageNames` through the Core Link route. */
  public getEnabledPackageNames(): Promise<Array<string>> { return this.client.callTyped<Array<string>>(5, 'getEnabledPackageNames', {}, (value) => (value as unknown[]).map((item) => item as string)); }
  /** Calls `isPackageEnabled` through the Core Link route. */
  public isPackageEnabled(packageName: string): Promise<boolean> { return this.client.callTyped<boolean>(5, 'isPackageEnabled', {'packageName': packageName}, (value) => value as boolean); }
  /** Calls `getActivePackageNames` through the Core Link route. */
  public getActivePackageNames(): Promise<Array<string>> { return this.client.callTyped<Array<string>>(5, 'getActivePackageNames', {}, (value) => (value as unknown[]).map((item) => item as string)); }
  /** Calls `enablePackage` through the Core Link route. */
  public enablePackage(packageName: string): Promise<string> { return this.client.callTyped<string>(5, 'enablePackage', {'packageName': packageName}, (value) => value as string); }
  /** Calls `disablePackage` through the Core Link route. */
  public disablePackage(packageName: string): Promise<string> { return this.client.callTyped<string>(5, 'disablePackage', {'packageName': packageName}, (value) => value as string); }
  /** Calls `getToolPkgPluginContainerDetails` through the Core Link route. */
  public getToolPkgPluginContainerDetails(useEnglish: boolean): Promise<Array<models.ToolPkgContainerDetails>> { return this.client.callTyped<Array<models.ToolPkgContainerDetails>>(5, 'getToolPkgPluginContainerDetails', {'useEnglish': useEnglish}, (value) => (value as unknown[]).map((item) => models.decodeToolPkgContainerDetails(item))); }
  /** Calls `getToolPkgContainerRuntimes` through the Core Link route. */
  public getToolPkgContainerRuntimes(): Promise<Array<models.ToolPkgContainerRuntime>> { return this.client.callTyped<Array<models.ToolPkgContainerRuntime>>(5, 'getToolPkgContainerRuntimes', {}, (value) => (value as unknown[]).map((item) => models.decodeToolPkgContainerRuntime(item))); }
  /** Calls `getToolPkgContainerOrder` through the Core Link route. */
  public getToolPkgContainerOrder(): Promise<Array<string>> { return this.client.callTyped<Array<string>>(5, 'getToolPkgContainerOrder', {}, (value) => (value as unknown[]).map((item) => item as string)); }
  /** Calls `setToolPkgContainerOrder` through the Core Link route. */
  public setToolPkgContainerOrder(packageNames: Array<string>): Promise<void> { return this.client.call(5, 'setToolPkgContainerOrder', {'packageNames': packageNames.map(item => item)}).then(() => undefined); }
  /** Calls `getToolPkgContainerDetails` through the Core Link route. */
  public getToolPkgContainerDetails(packageName: string, useEnglish: boolean): Promise<models.ToolPkgContainerDetails | null> { return this.client.callTyped<models.ToolPkgContainerDetails | null>(5, 'getToolPkgContainerDetails', {'packageName': packageName, 'useEnglish': useEnglish}, (value) => value == null ? null : models.decodeToolPkgContainerDetails(value)); }
  /** Calls `readToolPkgLogoBytes` through the Core Link route. */
  public readToolPkgLogoBytes(packageName: string): Promise<models.ToolPkgLogoBytes | null> { return this.client.callTyped<models.ToolPkgLogoBytes | null>(5, 'readToolPkgLogoBytes', {'packageName': packageName}, (value) => value == null ? null : models.decodeToolPkgLogoBytes(value)); }
  /** Calls `getEffectivePackageTools` through the Core Link route. */
  public getEffectivePackageTools(packageName: string): Promise<models.ToolPackage | null> { return this.client.callTyped<models.ToolPackage | null>(5, 'getEffectivePackageTools', {'packageName': packageName}, (value) => value == null ? null : models.decodeToolPackage(value)); }
  /** Calls `getPackageTools` through the Core Link route. */
  public getPackageTools(packageName: string): Promise<models.ToolPackage | null> { return this.client.callTyped<models.ToolPackage | null>(5, 'getPackageTools', {'packageName': packageName}, (value) => value == null ? null : models.decodeToolPackage(value)); }
  /** Calls `getAvailablePackages` through the Core Link route. */
  public getAvailablePackages(): Promise<Record<string, models.ToolPackage>> { return this.client.callTyped<Record<string, models.ToolPackage>>(5, 'getAvailablePackages', {}, (value) => Object.fromEntries(Object.entries(value as Record<string, unknown>).map(([key, item]) => [key, models.decodeToolPackage(item)]))); }
  /** Calls `getToolPkgLoadIssues` through the Core Link route. */
  public getToolPkgLoadIssues(): Promise<Array<models.ToolPkgLoadIssue>> { return this.client.callTyped<Array<models.ToolPkgLoadIssue>>(5, 'getToolPkgLoadIssues', {}, (value) => (value as unknown[]).map((item) => models.decodeToolPkgLoadIssue(item))); }
}

