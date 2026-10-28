/** Returns the problem in plain words, or "" when the address looks right. */
export function emailProblem(raw: string, mine: string): string {
  const email = raw.trim().toLowerCase();
  if (!email) return "Enter an email address.";
  if (!/^[^@\s]+@[^@\s]+\.[^@\s]{2,}$/.test(email)) return "That does not look like an email address. Example: name@example.com";
  if (email === mine) return "That is your own email. Enter the email of the person you want to share with.";
  return "";
}
