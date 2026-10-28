import { supabase } from "../supabase-client";

export type Share = {
  id: string;
  grantee_email: string;
  grantee_user_id: string | null;
  owner_display_name: string | null;
  owner_email?: string | null;
  created_at: string;
};
export type Shares = { byMe: Share[]; withMe: Share[] };

const COLUMNS = "id,grantee_email,grantee_user_id,owner_display_name,created_at,owner_user_id,owner_email";
const TABLE = "location_shares";

async function me() {
  const { data } = await supabase.auth.getSession();
  if (!data.session) throw new Error("Not signed in");
  return data.session.user;
}

/** Active shares: the ones I give (owner = me) and the ones given to me. RLS limits rows to mine. */
export async function listShares(): Promise<Shares> {
  const user = await me();
  const res = await supabase.from(TABLE).select(COLUMNS).is("revoked_at", null).order("created_at");
  if (res.error) throw new Error(res.error.message);
  const all = res.data as (Share & { owner_user_id: string })[];
  return { byMe: all.filter((s) => s.owner_user_id === user.id), withMe: all.filter((s) => s.owner_user_id !== user.id) };
}

/** Returns "duplicate" when the email already has an active share. */
export async function shareWith(email: string): Promise<"ok" | "duplicate"> {
  const user = await me();
  const meta = user.user_metadata ?? {};
  const owner_display_name = meta.full_name ?? meta.name ?? user.email ?? null;
  const { error } = await supabase.from(TABLE).insert({ grantee_email: email.toLowerCase(), owner_display_name });
  if (error?.code === "23505") return "duplicate";
  if (error) throw new Error(error.message);
  return "ok";
}

export async function stopSharing(id: string) {
  const { error } = await supabase.from(TABLE).update({ revoked_at: new Date().toISOString() }).eq("id", id);
  if (error) throw new Error(error.message);
}

export async function myEmail() {
  return (await me()).email?.toLowerCase() ?? "";
}
