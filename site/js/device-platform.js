// Which device the visitor is on, from the user agent. Shared by the Get started tabs and install help.
const ua = navigator.userAgent;
// iPadOS 13+ reports a Mac user agent; touch points tell it apart.
export const ios = /iPhone|iPad|iPod/.test(ua) || (/Macintosh/.test(ua) && navigator.maxTouchPoints > 1);
export const iosSafari = ios && !/CriOS|FxiOS|EdgiOS|OPiOS|GSA\//.test(ua);
export const android = /Android/.test(ua);
/** "ios", "android" or "desktop": picks the install path to show first. */
export const platform = ios ? "ios" : android ? "android" : "desktop";
// A touch screen at phone width counts as a phone even when the user agent says otherwise.
export const phoneLike = ios || android || (navigator.maxTouchPoints > 0 && matchMedia("(max-width: 50rem)").matches);
