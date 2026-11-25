import assert from "node:assert/strict";
import test from "node:test";
import { createDiskHarness } from "../../../buildin/character_cards/tests/disk-files.mjs";
import { openPlugin } from "../../../buildin/character_cards/tests/plugin-entry-harness.mjs";
import { loadModule, plain } from "../../../buildin/character_cards/tests/runtime.mjs";

/** Exercises the consumer against actual existing provider registrations, without adding a provider API. */
test("call roster reads saved and active groups through the unchanged provider", async t => {
  const disk = await createDiskHarness(t, { legacyIdentity: "default" }), provider = openPlugin(disk);
  const alice = await provider.api("character.create", { values: { name: "Alice", avatarUri: "alice.png" } });
  const bob = await provider.api("character.create", { values: { name: "Bob", avatarUri: "bob.png" } });
  const group = await provider.api("group.create", { values: { name: "Friends", avatarUri: "friends.png", members: [
    { characterCardId: alice.id, orderIndex: 2 }, { characterCardId: "default", orderIndex: 1 },
    { characterCardId: bob.id, orderIndex: 0 },
  ] } });
  await provider.main.beforeChatCreate({ eventName: "before_create", eventPayload: {
    eventName: "before_create", creationKind: "new", chat: { id: "bound-call", title: "Call", workspaceId: null, parentChatId: null },
    sourceChatId: null, sourceMessageTimestamp: null, sourceExtension: null,
    input: { "com.operit.character_cards": { version: 1, selection: "group:" + group.id } },
  } });
  await provider.api("chat.configuration.draft.commit", { chatId: "bound-call" });
  const methods = [];
  const consumer = loadModule("../../external/voice_call/src/presentation.ts", { ToolPkg: {
    async callDependency(packageId, method, payload) {
      assert.equal(packageId, "com.operit.character_cards");
      methods.push(method);
      const registration = provider.apis.get(method);
      assert.ok(registration, "Expected existing provider API: " + method);
      return registration.function({ callerPackage: "com.operit.voice_call", payload: plain(payload) });
    },
  } });
  const result = plain(await consumer.readCallPresentation("bound-call"));
  assert.deepEqual(result, {
    identity: { title: "Friends", avatarUri: "friends.png" },
    participants: [
      { id: bob.id, name: "Bob", avatarUri: "bob.png" },
      { id: "default", name: "Operit", avatarUri: null },
      { id: alice.id, name: "Alice", avatarUri: "alice.png" },
    ],
  });
  await provider.api("activePrompt.setGroup", { id: group.id });
  assert.deepEqual(plain(await consumer.readCallPresentation(null)), result);
  await provider.api("activePrompt.setCard", { id: alice.id });
  assert.deepEqual(plain(await consumer.readCallPresentation(null)), {
    identity: { title: "Alice", avatarUri: "alice.png" }, participants: [],
  });
  assert.deepEqual(plain(await consumer.readCallPresentation("bound-call")), result);
  assert.equal(provider.apis.has("chat.presentation"), false);
  assert.equal(methods.includes("chat.presentation"), false);
});
