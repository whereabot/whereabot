/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_SUPABASE_URL: string;
  readonly VITE_SUPABASE_ANON_KEY: string;
  readonly VITE_RELAY_URL: string;
  readonly VITE_DEV_LOGIN?: string;
  /** Browser build: VAPID public key (base64url). Unset: no Web Push. */
  readonly VITE_VAPID_PUBLIC_KEY?: string;
}
