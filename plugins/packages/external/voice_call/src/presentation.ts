import { characterCards, type ChatDisplayIdentity, type ChatDisplayParticipant } from "./dependencies/character_cards/api.js";

/** Reads the roster with the provider's existing public binding, group and character APIs. */
async function groupParticipants(chatId: string | null): Promise<ChatDisplayParticipant[]> {
  let groupId: string;
  if (chatId === null) {
    const active = await characterCards.activePrompt.get();
    if (!("CharacterGroup" in active)) return [];
    groupId = active.CharacterGroup.id;
  } else {
    const binding = await characterCards.chat.configuration.binding.read({ chatId });
    if (binding.chatId !== chatId) throw new Error("Call binding returned a different chat");
    if (!binding.selection.startsWith("group:")) return [];
    groupId = binding.selection.slice("group:".length);
  }
  if (!groupId || /\s/u.test(groupId)) throw new Error("Call group ID is invalid");
  const group = await characterCards.groups.get({ id: groupId });
  if (group.id !== groupId || group.members.length === 0) throw new Error("Call group has no valid members");
  const ids = new Set<string>();
  const members = [...group.members].sort((left, right) => left.orderIndex - right.orderIndex);
  for (const member of members) {
    if (!member.characterCardId || ids.has(member.characterCardId)) throw new Error("Call group member is empty or duplicated");
    ids.add(member.characterCardId);
  }
  return Promise.all(members.map(async member => {
    const card = await characterCards.characters.get({ id: member.characterCardId });
    if (card.id !== member.characterCardId || !card.name.trim()) throw new Error("Call member returned an invalid identity");
    return { id: card.id, name: card.name, avatarUri: card.avatarUri };
  }));
}

/** Keeps the group portrait and complete roster separate from the active speaker. */
export async function readCallPresentation(chatId: string | null): Promise<{
  identity: ChatDisplayIdentity; participants: ChatDisplayParticipant[];
}> {
  const [identity, participants] = await Promise.all([
    characterCards.chat.identity({ chatId }), groupParticipants(chatId),
  ]);
  return {
    identity: { title: identity?.title ?? "Operit", avatarUri: identity?.avatarUri || await characterCards.defaultAvatar() },
    participants,
  };
}
