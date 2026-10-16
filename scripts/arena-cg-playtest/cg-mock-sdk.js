// The CrazyGames SDK v3 as the portal runs it, faithfully mocked for the
// playtest frame (scripts/arena-cg-playtest.sh, #1219). The cg build loads
// this file instead of https://sdk.crazygames.com/crazygames-sdk-v3.js: the
// real SDK stays "disabled" off crazygames.com, so a login couldn't be tried.
// Same surface the game uses: init, game.{loadingStart,loadingStop,gameplayStart,
// gameplayStop,settings,addSettingsChangeListener}, ad.requestAd, user.{getUser,
// getUserToken,addAuthListener}. Every call is logged to the console and to the
// frame's page (postMessage), which lists them in order. The frame's buttons
// reach it by message: login, ad is asked by the game, mute, new-guest.
// The login's token is signed RS256 with a test key (cg-mock-key.js) the
// workshop server trusts (ARENA_CG_PUBLIC_KEY), claims as CrazyGames' own. The
// user is always the same account (userId "playtest-cg-user-1", so a second
// browser is the same player); `?cguser=<id>` on the frame's page picks another.
(() => {
  const q = new URLSearchParams(location.search);
  const store = (k, v) => { try { return v === undefined ? localStorage.getItem(k) : v === null ? localStorage.removeItem(k) : localStorage.setItem(k, v); } catch { return null; } };
  const log = (call, detail) => {
    console.log("[CrazyGames SDK mock]", call, detail ?? "");
    try { parent.postMessage({ type: "cg-log", call, detail: detail ?? "", at: Date.now() }, "*"); } catch {}
  };
  const settings = { muteAudio: store("cgmock.mute") === "1" || q.get("muteAudio") === "true", disableChat: q.get("disableChat") === "true" };
  const settingListeners = [];
  const authListeners = [];
  let user = store("cgmock.user") ? { username: "PlaytestUser", profilePictureUrl: "" } : null;

  const b64 = (bytes) => btoa(String.fromCharCode(...new Uint8Array(bytes))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  const b64json = (o) => b64(new TextEncoder().encode(JSON.stringify(o)));
  async function token() {
    const jwk = window.__CG_MOCK_KEY;
    if (!jwk) throw new Error("no test key");
    const key = await crypto.subtle.importKey("jwk", jwk, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, false, ["sign"]);
    const now = Math.floor(Date.now() / 1000);
    const body = `${b64json({ alg: "RS256", typ: "JWT" })}.${b64json({ userId: q.get("cguser") || "playtest-cg-user-1", username: "PlaytestUser", gameId: "playtest", profilePictureUrl: "", iat: now, exp: now + 3600 })}`;
    return `${body}.${b64(await crypto.subtle.sign("RSASSA-PKCS1-v1_5", key, new TextEncoder().encode(body)))}`;
  }

  window.CrazyGames = { SDK: {
    environment: "crazygames",
    init: async () => log("init"),
    game: {
      settings,
      loadingStart: () => log("game.loadingStart"),
      loadingStop: () => log("game.loadingStop"),
      gameplayStart: () => log("game.gameplayStart"),
      gameplayStop: () => log("game.gameplayStop"),
      addSettingsChangeListener: (fn) => { settingListeners.push(fn); log("game.addSettingsChangeListener"); },
    },
    ad: {
      requestAd: (type, cb) => {
        log("ad.requestAd", type);
        cb?.adStarted?.();
        log("ad: adStarted", "game mutes");
        setTimeout(() => { cb?.adFinished?.(); log("ad: adFinished", "game unmutes"); }, 3000);
      },
    },
    user: {
      isUserAccountAvailable: true,
      getUser: async () => { log("user.getUser", user ? user.username : "null (guest)"); return user; },
      getUserToken: async () => { if (!user) throw new Error("userNotAuthenticated"); log("user.getUserToken"); return token(); },
      addAuthListener: (fn) => { authListeners.push(fn); log("user.addAuthListener"); },
    },
  } };

  addEventListener("message", (e) => {
    if (e.source !== parent || e.data?.type !== "cg-mock") return;
    switch (e.data.action) {
      case "login":
        user = { username: "PlaytestUser", profilePictureUrl: "" };
        store("cgmock.user", "1");
        log("portal: user logged in", user.username);
        authListeners.forEach((f) => f(user));
        break;
      case "mute":
        settings.muteAudio = !!e.data.on;
        store("cgmock.mute", settings.muteAudio ? "1" : "0");
        log("portal: settings change", `muteAudio=${settings.muteAudio}`);
        settingListeners.forEach((f) => f({ ...settings }));
        break;
      case "new-guest":
        try { localStorage.clear(); sessionStorage.clear(); } catch {}
        log("portal: new guest", "storage cleared, reloading");
        location.reload();
        break;
    }
  });
  parent.postMessage({ type: "cg-ready" }, "*");
})();
