const comments = document.querySelector(".comments");
const container = comments.querySelector(".giscus");
const button = comments.querySelector(".comments-toggle");
const storageKey = "inkcairn:giscus";

try {
  if (localStorage.getItem(storageKey)) openGiscus();
} catch {}

button.addEventListener("click", () => {
  if (comments.classList.contains("comments-live")) {
    showStaticComments();
  } else {
    openGiscus();
  }
});

document.addEventListener("click", event => {
  if (!event.target.closest("#theme-toggle")) return;

  const theme = document.documentElement.dataset.theme;
  container.querySelector(".giscus-frame")?.contentWindow.postMessage(
    { giscus: { setConfig: { theme } } },
    "https://giscus.app",
  );
});

function openGiscus() {
  const script = document.createElement("script");

  Object.assign(script.dataset, button.dataset, {
    mapping: "specific",
    strict: "1",
    reactionsEnabled: "0",
    emitMetadata: "0",
    inputPosition: "top",
    theme: document.documentElement.dataset.theme,
  });
  script.src = "https://giscus.app/client.js";
  script.crossOrigin = "anonymous";
  script.async = true;
  script.onload = () => {
    try {
      localStorage.setItem(storageKey, "1");
    } catch {}
  };
  script.onerror = showStaticComments;

  comments.classList.add("comments-live");
  button.textContent = "View static comments";
  button.setAttribute("aria-pressed", "true");
  container.append(script);
}

function showStaticComments() {
  comments.classList.remove("comments-live");
  container.replaceChildren();
  button.textContent = "Use Giscus";
  button.setAttribute("aria-pressed", "false");
  try {
    localStorage.removeItem(storageKey);
  } catch {}
}
