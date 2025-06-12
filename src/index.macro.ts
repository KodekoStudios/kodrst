// Gateway
export function gateway_bot(): "/api/v10/gateway/bot" {
  return "/api/v10/gateway/bot";
}

export function gateway(): "/api/v10/gateway" {
  return "/api/v10/gateway";
}

// User
export function me(): "/api/v10/users/@me" {
  return "/api/v10/users/@me";
}

export function user<T extends string>(user_id: T): `/api/v10/users/${T}` {
  return `/api/v10/users/${user_id}`;
}

// Channel
export function channel<T extends string>(channel_id: T): `/api/v10/channels/${T}` {
  return `/api/v10/channels/${channel_id}`;
}

export function channel_messages<T extends string>(channel_id: T): `/api/v10/channels/${T}/messages` {
  return `/api/v10/channels/${channel_id}/messages`;
}

export function channel_message<C extends string, M extends string>(channel_id: C, message_id: M): `/api/v10/channels/${C}/messages/${M}` {
  return `/api/v10/channels/${channel_id}/messages/${message_id}`;
}

// Reactions
export function message_reaction<C extends string, M extends string, E extends string>(channel_id: C, message_id: M, emoji: E): `/api/v10/channels/${C}/messages/${M}/reactions/${E}/@me` {
  return `/api/v10/channels/${channel_id}/messages/${message_id}/reactions/${emoji}/@me`;
}

// Guilds
export function guild<T extends string>(guild_id: T): `/api/v10/guilds/${T}` {
  return `/api/v10/guilds/${guild_id}`;
}

export function guild_channels<T extends string>(guild_id: T): `/api/v10/guilds/${T}/channels` {
  return `/api/v10/guilds/${guild_id}/channels`;
}

export function guild_member<G extends string, U extends string>(guild_id: G, user_id: U): `/api/v10/guilds/${G}/members/${U}` {
  return `/api/v10/guilds/${guild_id}/members/${user_id}`;
}
