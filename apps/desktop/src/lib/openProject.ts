import type { AgentId, MessagePart, Project } from '@listune/shared';
import { SKIP_FRAME_CHECK } from '@listune/skills';
import { createChat, listChats } from '@/lib/ipc';
import { useApp } from '@/state/app';
import { BUSY_NOTE, useChat, type SendOpts } from '@/state/chat';

/** The project's latest chat, or a fresh one when it has none yet. */
async function projectChat(p: Project) {
  const chats = await listChats(p.id);
  return chats[0] ?? (await createChat(p.id, useApp.getState().settings?.defaultAgent ?? 'claude', p.title));
}

/** A project's chat: the latest one, or a fresh one when it has none yet. */
export async function openProjectChat(p: Project): Promise<void> {
  const chat = await projectChat(p);
  useApp.getState().navigate({ name: 'chat', chatId: chat.id });
}

/** What every send in a project chat carries: its agent, its build job, and the skip line when the frame check is off. */
export function projectSendOptions(project: Pick<Project, 'id' | 'title'> | null, agentId: AgentId, prompt: MessagePart[]): SendOpts {
  // The brief's frame check applies to projects made while it was on; this covers turning it off later.
  const skip = useApp.getState().settings?.frameCheck === false;
  return {
    job: project ? { projectId: project.id, title: project.title } : undefined,
    agentId,
    ...(skip ? { promptParts: [{ type: 'text', text: SKIP_FRAME_CHECK }, ...prompt] } : {}),
  };
}

/**
 * Sends `text` as the user's message in a project chat, through the normal send path with that chat's
 * options. While a turn is running there it only adds the busy note.
 */
export async function sendInProjectChat(
  chatId: string,
  project: Pick<Project, 'id' | 'title'>,
  agentId: AgentId,
  text: string,
): Promise<void> {
  const store = useChat.getState();
  store.listen();
  // The conversation must be in memory before anything is added to it, or the chat would open without its history.
  await store.load(chatId);
  if (useChat.getState().streaming[chatId]) {
    useChat.getState().notice(chatId, BUSY_NOTE);
    return;
  }
  const ask: MessagePart[] = [{ type: 'text', text }];
  // What the user left in the composer stays there: this message is Listune's, sent for them.
  await useChat.getState().send(chatId, ask, { ...projectSendOptions(project, agentId, ask), keepAttachments: true });
}
